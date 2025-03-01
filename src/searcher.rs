use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Select;
use similar_string::compare_similarity;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct FileInfo {
    path: String,
    score: f64,
}

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

    // Weighted combination
    name_similarity + contains_score
}

fn search_fuzzy(search_term: &str, exts: Vec<String>) -> Vec<FileInfo> {
    // Starting directory
    let dir = Path::new("C:\\");

    // Excluded directories
    let excluded_dirs = vec![
        "C:\\Windows",
        "C:\\Windows.old",
        "C:\\ProgramData",
        "C:\\Program Files\\Microsoft",
        "C:\\Program Files\\Windows",
        "C:\\Program Files (x86)\\Microsoft",
        "C:\\Program Files (x86)\\Windows",
    ];

    // Results collection with pre-allocated capacity for performance
    let results = Arc::new(Mutex::new(Vec::with_capacity(350_000)));

    let file_count = Arc::new(AtomicUsize::new(0));

    // Progress bar setup
    let progress_bar = ProgressBar::new_spinner();
    let progress_style = ProgressStyle::default_spinner()
        .template("{spinner:.green} [{elapsed_precise}] {msg}")
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

    progress_bar.set_style(progress_style);
    progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));
    progress_bar.set_message("Collecting files...");

    // Helper function to check if path is excluded
    fn is_excluded(entry: &ignore::DirEntry, excluded: &[&str]) -> bool {
        let path = entry.path();
        !excluded.iter().any(|ex| path.starts_with(ex))
    }

    let walker = WalkBuilder::new(dir)
        .max_depth(Some(10))
        .same_file_system(true)
        .git_ignore(true)
        .hidden(true)
        .threads(num_cpus::get() * 2) // Use twice the number of CPUs for more parallelism
        .filter_entry(move |entry| is_excluded(entry, &excluded_dirs))
        .build_parallel();

    // Run parallel walker
    walker.run(|| {
        let local_results = Arc::clone(&results);
        let local_file_count = Arc::clone(&file_count);
        let progress_bar = progress_bar.clone();
        let local_search_term = search_term.to_string();
        let local_exts = exts.clone();

        Box::new(move |result| {
            let files_so_far = local_file_count.load(Ordering::Relaxed);
            // Update progress periodically
            if files_so_far % 1000 == 0 {
                progress_bar.set_message(format!("Collecting files - Found {}", files_so_far));
            }

            if let Ok(entry) = result {
                // Only process files, not directories
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    // Increase file counter
                    local_file_count.fetch_add(1, Ordering::Relaxed);

                    let path = entry.path().to_path_buf();
                    if let Some(ext) = path.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        if local_exts.contains(&ext) {
                            let path = path.to_string_lossy().to_string();
                            let similarity = calculate_similarity(&local_search_term, &path);
                            if similarity > 0.3 {
                                // Add to results
                                let mut results = local_results.lock().unwrap();
                                results.push(FileInfo {
                                    path,
                                    score: similarity,
                                });
                            }
                        }
                    }
                }
            }

            ignore::WalkState::Continue
        })
    });

    // Get final counts
    let final_file_count = file_count.load(Ordering::Relaxed);
    progress_bar.finish_with_message(format!("Finished collecting {} files", final_file_count));

    let final_results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    final_results
}

pub fn prompt_fzf(
    search_term: &str,
    max_results: usize,
    prompt: &str,
    exts: Vec<String>,
) -> String {
    let mut opts = search_fuzzy(search_term, exts);
    opts.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    opts.truncate(max_results);

    let opts = opts.iter().map(|f| f.path.clone()).collect();
    let select = Select::new(prompt, opts)
        .without_filtering()
        .with_vim_mode(true)
        .prompt()
        .unwrap();

    select
}
