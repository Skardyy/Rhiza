use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Select;
use similar_string::compare_similarity;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread;

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

fn search_fuzzy(search_term: &str, exts: Vec<String>) -> Receiver<FileInfo> {
    let (sender, receiver) = channel::<FileInfo>();

    let dir = Path::new("C:\\");
    let excluded_dirs = vec![
        "C:\\Windows",
        "C:\\Windows.old",
        "C:\\ProgramData",
        "C:\\Program Files\\Microsoft",
        "C:\\Program Files\\Windows",
        "C:\\Program Files (x86)\\Microsoft",
        "C:\\Program Files (x86)\\Windows",
    ];

    // Setup progress bar
    let progress_bar = ProgressBar::new_spinner();
    let progress_style = ProgressStyle::default_spinner()
        .template("{spinner:.green} [{elapsed_precise}] {msg}")
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);
    progress_bar.set_style(progress_style);
    progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));
    progress_bar.set_message("Collecting files...");
    let file_count = Arc::new(AtomicUsize::new(0));

    fn is_excluded(entry: &ignore::DirEntry, excluded: &[&str]) -> bool {
        let path = entry.path();
        !excluded.iter().any(|ex| path.starts_with(Path::new(ex)))
    }

    // Build the walker with parallel processing
    let walker = WalkBuilder::new(dir)
        .max_depth(Some(10))
        .same_file_system(true)
        .git_ignore(true)
        .hidden(true)
        .threads(num_cpus::get() * 2)
        .filter_entry(move |entry| is_excluded(entry, &excluded_dirs))
        .build_parallel();

    // Clone variables needed inside the spawned thread
    let search_term = search_term.to_string();
    let exts = exts.clone();
    let file_count_clone = Arc::clone(&file_count);
    let progress_bar_clone = progress_bar.clone();

    // Spawn a thread to run the walker concurrently
    thread::spawn(move || {
        walker.run(|| {
            // Clone for each worker thread
            let sender = sender.clone();
            let file_count = Arc::clone(&file_count_clone);
            let search_term = search_term.clone();
            let exts = exts.clone();
            let progress_bar = progress_bar_clone.clone();

            Box::new(move |result| {
                let files_so_far = file_count.load(Ordering::Relaxed);
                if files_so_far % 1000 == 0 {
                    progress_bar.set_message(format!("Collecting files - Found {}", files_so_far));
                }

                if let Ok(entry) = result {
                    // Only process files (not directories)
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        file_count.fetch_add(1, Ordering::Relaxed);
                        let path: PathBuf = entry.path().to_path_buf();
                        if let Some(ext) = path.extension() {
                            let ext = ext.to_string_lossy().to_lowercase();
                            if exts.contains(&ext) {
                                let path_str = path.to_string_lossy().to_string();
                                let similarity = calculate_similarity(&search_term, &path_str);
                                if similarity > 0.3 {
                                    // Send matching file info; ignore send errors if the receiver is dropped
                                    let _ = sender.send(FileInfo {
                                        path: path_str,
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
        progress_bar_clone.finish_with_message(format!(
            "Finished collecting {} files",
            file_count_clone.load(Ordering::Relaxed)
        ));
    });

    receiver
}

pub fn prompt_fzf(
    search_term: &str,
    max_results: usize,
    prompt: &str,
    exts: Vec<String>,
) -> String {
    // Collect the results from the channel iterator.
    let mut opts: Vec<FileInfo> = search_fuzzy(search_term, exts).into_iter().collect();
    opts.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    opts.truncate(max_results);

    let opts = opts.into_iter().map(|f| f.path).collect();
    Select::new(prompt, opts)
        .without_filtering()
        .with_vim_mode(true)
        .prompt()
        .unwrap()
}
