use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Select;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread;

fn search_fuzzy(exts: Vec<String>, silent: bool) -> Receiver<String> {
    let (sender, receiver) = channel::<String>();

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
    if !silent {
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
        .git_ignore(false)
        .hidden(true)
        .threads(num_cpus::get() * 2)
        .filter_entry(move |entry| is_excluded(entry, &excluded_dirs))
        .build_parallel();

    // Clone variables needed inside the spawned thread
    let exts = exts.clone();
    let file_count_clone = Arc::clone(&file_count);
    let progress_bar_clone = progress_bar.clone();

    // Spawn a thread to run the walker concurrently
    thread::spawn(move || {
        walker.run(|| {
            // Clone for each worker thread
            let sender = sender.clone();
            let file_count = Arc::clone(&file_count_clone);
            let exts = exts.clone();
            let progress_bar = progress_bar_clone.clone();

            Box::new(move |result| {
                if !silent {
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
                                let _ = sender.send(path_str);
                            }
                        }
                    }
                }
                ignore::WalkState::Continue
            })
        });
        if !silent {
            progress_bar_clone.finish_with_message(format!(
                "Finished collecting {} files",
                file_count_clone.load(Ordering::Relaxed)
            ));
        }
    });

    receiver
}

pub fn prompt_fzf(items: Receiver<String>, prompt: &str) -> Option<String> {
    // attempt fzf first
    if let Some(mut child) = spawn_fzf() {
        if let Some(stdin) = child.stdin.as_mut() {
            for item in items {
                if writeln!(stdin, "{}", item).is_err() {
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

    // Fallback
    let opts: Vec<String> = items.into_iter().collect();

    Some(
        Select::new(prompt, opts)
            .with_vim_mode(true)
            .prompt()
            .unwrap(),
    )
}

pub fn search_prompt_fzf(prompt: &str, exts: Vec<String>) -> Option<String> {
    let items = search_fuzzy(exts, which::which("fzf").is_ok());
    prompt_fzf(items, prompt)
}

fn spawn_fzf() -> Option<Child> {
    Command::new("fzf")
        .arg("--tiebreak=end,length")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()
}
