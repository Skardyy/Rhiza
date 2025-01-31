use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use chrono::{DateTime, Local};
use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use similar_string::compare_similarity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMatch {
    pub path: PathBuf,
    pub name: String,
    pub similarity_score: f64,
    pub last_modified: std::time::SystemTime,
}

impl FileMatch {
    pub fn formatted_last_modified(&self) -> String {
        let datetime: DateTime<Local> = self.last_modified.clone().into();
        datetime.format("%m/%d/%Y %I:%M %p").to_string()
    }
}

pub struct FileSearchOptimizer {
    extensions: Vec<String>,
    excluded_dirs: Vec<PathBuf>,
}

impl FileSearchOptimizer {
    pub fn new() -> Self {
        Self {
            extensions: vec!["lnk".to_string(), "url".to_string(), "exe".to_string()],
            excluded_dirs: Self::get_system_dirs(),
        }
    }

    /// Get standard Windows system directories to exclude
    fn get_system_dirs() -> Vec<PathBuf> {
        vec![
            // Standard Windows system directories
            PathBuf::from("C:\\Windows"),
            PathBuf::from("C:\\ProgramData"),
            // Common system directories to skip
            PathBuf::from("C:\\Windows\\System32"),
            PathBuf::from("C:\\Windows\\WinSxS"),
        ]
    }

    /// Fuzzy matching with advanced scoring
    fn calculate_similarity(search_term: &str, filename: &str) -> f64 {
        // Normalize by lowercase
        let normalized_search = search_term.to_lowercase();
        let normalized_filename = filename.to_lowercase();

        // Combine different similarity metrics
        let name_similarity = compare_similarity(&normalized_search, &normalized_filename);
        let contains_score = if normalized_filename.contains(&normalized_search) {
            0.7
        } else {
            0.0
        };
        let start_score = if normalized_filename.starts_with(&normalized_search) {
            0.3
        } else {
            0.0
        };

        // Weighted combination
        name_similarity + contains_score + start_score
    }

    /// Search files with parallel processing
    pub fn find_top_matches(&self, search_term: &str, max_results: usize) -> Vec<FileMatch> {
        let dir = Path::new("C:\\");
        let results = Arc::new(Mutex::new(Vec::new()));

        // Progress tracking
        let processed_files = Arc::new(AtomicUsize::new(0));
        let progress_bar = Arc::new(ProgressBar::new_spinner());

        // Configure progress bar style
        let progress_style = ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));
        progress_bar.set_message(format!("Searching for '{}'...", search_term));

        // Parallel directory walk
        let walker = WalkBuilder::new(&dir)
            .max_depth(Some(10)) // Limit depth to prevent excessive searching
            .same_file_system(true) // Avoid network/removable drives
            .git_ignore(false)
            .hidden(false)
            .build_parallel();

        walker.run(|| {
            let local_results = Arc::clone(&results);
            let local_processed_files = Arc::clone(&processed_files);
            let local_progress_bar = Arc::clone(&progress_bar);
            let local_search_term = search_term.to_string();

            Box::new(move |result| {
                // Increment processed files count
                let current_count = local_processed_files.fetch_add(1, Ordering::Relaxed) + 1;

                // Update progress bar periodically to avoid performance overhead
                if current_count % 100 == 0 {
                    local_progress_bar.set_message(format!(
                        "Searching for '{}' - Processed {} files",
                        local_search_term, current_count
                    ));
                }

                if let Ok(entry) = result {
                    let path = entry.path().to_path_buf();

                    // Check file extensions and exclusions
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_str().unwrap_or_default().to_lowercase();
                        let is_excluded_dir = self
                            .excluded_dirs
                            .iter()
                            .any(|excluded| path.starts_with(excluded));

                        if self.extensions.contains(&ext_str) && !is_excluded_dir {
                            if let Some(filename) = path.file_name() {
                                let filename_str = filename.to_str().unwrap_or_default();
                                let similarity =
                                    Self::calculate_similarity(&local_search_term, filename_str);

                                if similarity > 0.3 {
                                    // Threshold to filter weak matches
                                    let metadata = std::fs::metadata(&path).ok();
                                    let last_modified = metadata
                                        .and_then(|m| m.modified().ok())
                                        .unwrap_or(std::time::UNIX_EPOCH);

                                    let path = path.clone();
                                    let file_match = FileMatch {
                                        path,
                                        name: filename_str.to_string(),
                                        similarity_score: similarity,
                                        last_modified,
                                    };

                                    let mut results = local_results.lock().unwrap();
                                    results.push(file_match);
                                }
                            }
                        }
                    }
                }
                ignore::WalkState::Continue
            })
        });

        // Finish progress bar
        progress_bar.finish_with_message(format!("Finished searching for '{}'", search_term));

        // Sort and truncate results
        let mut final_results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();

        final_results.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.last_modified.cmp(&a.last_modified))
        });

        final_results.truncate(max_results);
        final_results
    }
}
