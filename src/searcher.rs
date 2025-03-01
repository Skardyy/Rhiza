use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};
use similar_string::compare_similarity;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
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
    let empty_search = search_term == "";

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
    if !empty_search {
        let progress_style = ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));
        progress_bar.set_message("Collecting files...");
    }
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
                if !empty_search {
                    let files_so_far = file_count.load(Ordering::Relaxed);
                    if files_so_far % 1000 == 0 {
                        progress_bar
                            .set_message(format!("Collecting files - Found {}", files_so_far));
                    }
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
                                let similarity = if !empty_search {
                                    calculate_similarity(&search_term, &path_str)
                                } else {
                                    0.0
                                };
                                if empty_search || similarity > 0.3 {
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
        if !empty_search {
            progress_bar_clone.finish_with_message(format!(
                "Finished collecting {} files",
                file_count_clone.load(Ordering::Relaxed)
            ));
        }
    });

    receiver
}

pub fn prompt_fzf(
    search_term: Option<&String>,
    max_results: usize,
    prompt: &str,
    exts: Vec<String>,
) -> Option<String> {
    if !search_term.is_some() {
        if let Some(mut child) = spawn_fzf() {
            if let Some(stdin) = child.stdin.as_mut() {
                for file_info in search_fuzzy("", exts.clone()) {
                    // The Write trait is in scope so you can use writeln!
                    if writeln!(stdin, "{}", file_info.path).is_err() {
                        eprintln!("Failed to write to fzf stdin");
                        break;
                    }
                }
            }

            // Wait for fzf to complete and capture its output.
            let output = child
                .wait_with_output()
                .expect("Failed to wait on fzf child");
            if output.status.success() {
                return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            } else {
                return None;
            }
        }
    }

    // Fallback
    let search_term = match search_term {
        Some(v) => v,
        None => &Text::new("what to search for?").prompt().unwrap(),
    };
    let mut opts: Vec<FileInfo> = search_fuzzy(&search_term, exts).into_iter().collect();
    opts.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    opts.truncate(max_results);
    let opts: Vec<String> = opts.into_iter().map(|f| f.path).collect();

    Some(
        Select::new(prompt, opts)
            .without_filtering()
            .with_vim_mode(true)
            .prompt()
            .unwrap(),
    )
}

fn spawn_fzf() -> Option<Child> {
    Command::new("fzf")
        .arg("--tiebreak=end,length")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()
}
