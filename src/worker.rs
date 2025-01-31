use colored::Colorize;
use inquire::{Confirm, InquireError, Text};
use shellexpand::tilde;
use std::{fs, io, path::Path};
use walkdir::{DirEntry, WalkDir};

use crate::installer;

pub fn crawl_directory(dirs: Vec<&str>) -> Result<Vec<String>, InquireError> {
    let executables = Vec::new();
    let mut config = installer::check()?;
    let abs_paths = config.expand();

    for dir in dirs {
        let expanded_dir = shellexpand::full(dir)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
        let expanded_dir = expanded_dir.as_ref();
        let target_extensions = ["exe", "lnk", "url"];

        let skips = vec![
            "Windows Kits",
            "Windows Accessories",
            "PowerShell",
            "Visual Studio",
            "Windows System",
            "Windows Tools",
            "Accessibility",
            "System Tools",
            "Accessories",
            "Git",
            "make",
            "Make",
            "Microsoft Edge", // Lol
            "Node.js",
            "Administrative Tools",
            "Python",
        ];
        for entry in WalkDir::new(expanded_dir)
            .into_iter()
            .filter_entry(|e| {
                !skips
                    .iter()
                    .any(|skip| e.path().to_string_lossy().contains(skip))
            })
            .filter_map(|e| e.ok())
        {
            if is_executable(&entry, &target_extensions) && is_user_friendly(&entry) {
                if let Some(path) = entry.path().to_str() {
                    let item = path.to_string();

                    // Check for path duplicates
                    let path_exists = config.commands.values().any(|cmd| cmd == path);
                    let skipped_before = config.skipped.contains(&path.to_owned());
                    let expanded_exists = file_exists(&abs_paths, path)?;
                    if path_exists || skipped_before || expanded_exists {
                        continue;
                    }

                    let wanted = Confirm::new(&format!("Add {} ?", item))
                        .with_default(false)
                        .prompt()?;
                    if wanted {
                        // give it a name
                        let place_holder = get_name(&entry)?;
                        let name = Text::new("how to call it?")
                            .with_default(&place_holder)
                            .prompt()?;

                        // Check for name conflicts
                        if config.commands.contains_key(&name) {
                            let override_existing = Confirm::new(&format!(
                                "Command name '{}' already exists. Do you want to override it?",
                                name
                            ))
                            .with_default(true)
                            .prompt()?;

                            if !override_existing {
                                continue;
                            }
                        }

                        // Add the new command
                        config.commands.insert(name.clone(), path.to_string());
                    } else {
                        config.skipped.push(path.to_string());
                    }
                }
            }
        }
    }

    // extend and write
    let rhiza_dir = tilde("~/.rhiza").to_string();
    let config_file = Path::new(&rhiza_dir).join("config.json");
    fs::write(
        config_file,
        serde_json::to_string_pretty(&config)
            .map_err(|e| InquireError::IO(io::Error::new(io::ErrorKind::Other, e.to_string())))?,
    )
    .map_err(InquireError::IO)?;

    Ok(executables)
}

fn file_exists(expanded_lnks: &Vec<String>, path: &str) -> Result<bool, io::Error> {
    if path.ends_with(".lnk") || path.ends_with(".url") {
        if let Some(expanded_path) = installer::read_shortcut(path) {
            let flag = expanded_lnks.contains(&expanded_path);
            return Ok(flag);
        }
    }
    if path.ends_with(".exe") {
        let flag = expanded_lnks.contains(&path.to_owned());
        return Ok(flag);
    }
    Ok(false)
}

pub fn run() -> io::Result<()> {
    // Get config
    let config = installer::check()?;
    let rhiza_bin = tilde("~/.rhiza/bin").to_string();

    // Ensure bin directory exists
    fs::create_dir_all(&rhiza_bin)?;

    for (key, path) in config.commands.iter() {
        let source_path = Path::new(path);

        // Skip if source doesn't exist
        if !source_path.exists() {
            println!(
                "{}",
                format!("Source {} doesn't exist, skipping...", path).red(),
            );
            continue;
        }

        match source_path.extension().and_then(|ext| ext.to_str()) {
            Some("url") | Some("lnk") => {
                // For .url and .lnk files, we copy them
                let ext = source_path.extension().unwrap().to_str().unwrap();
                let target_name = format!("{}.{}", key, ext);
                let target_path = Path::new(&rhiza_bin).join(&target_name);

                if !target_path.exists() {
                    fs::copy(source_path, &target_path)?;
                    println!(
                        "{} {} -> {}",
                        "Created".green(),
                        key.bold(),
                        target_path.display()
                    );
                }
            }
            Some("exe") => {
                // For .exe files, we create a shortcut
                let target_name = format!("{}.lnk", key);
                let target_path = Path::new(&rhiza_bin).join(&target_name);

                if !target_path.exists() {
                    create_shortcut(source_path, &target_path)?;
                    println!(
                        "{} {} -> {}",
                        "Created shortcut".green(),
                        key,
                        target_path.display()
                    );
                }
            }
            _ => {
                println!(
                    "{}",
                    format!("Unsupported file type {}, skipping ...", path).yellow()
                );
            }
        }
    }

    Ok(())
}

fn is_executable(entry: &DirEntry, target_extensions: &[&str]) -> bool {
    if entry.file_type().is_dir() {
        return false;
    }

    if let Some(ext) = entry.path().extension() {
        if let Some(ext_str) = ext.to_str() {
            return target_extensions.contains(&ext_str.to_lowercase().as_str());
        }
    }

    false
}

fn get_name(file: &DirEntry) -> Result<String, io::Error> {
    let file_name = file
        .file_name()
        .to_str()
        .ok_or(io::Error::from(io::ErrorKind::InvalidData))?
        .to_lowercase();

    let first_part = file_name.split_whitespace().next().unwrap_or(&file_name);

    if first_part.len() > 7 {
        let first = first_part.chars().next().unwrap_or(' ');
        let last = first_part.chars().last().unwrap_or(' ');
        let combined = format!("{}{}", first, last);
        Ok(combined)
    } else {
        Ok(first_part.to_owned())
    }
}

fn is_user_friendly(entry: &DirEntry) -> bool {
    let path = entry.path();

    // Exclude executables in certain directories
    let excluded_dirs = [
        "debug",
        "release",
        "obj",
        "bin",
        "build",
        "node_modules",
        "temp",
    ];
    if path
        .components()
        .any(|c| excluded_dirs.contains(&c.as_os_str().to_string_lossy().to_lowercase().as_str()))
    {
        return false;
    }

    // Exclude executables with certain names
    let excluded_names = [
        "debug",
        "test",
        "example",
        "sample",
        "setup",
        "uninstall",
        "install",
    ];
    if let Some(file_name) = path.file_stem() {
        let file_name = file_name.to_string_lossy().to_lowercase();
        if excluded_names.iter().any(|&name| file_name.contains(name)) {
            return false;
        }
    }

    true
}
fn create_shortcut(source: &Path, target: &Path) -> io::Result<()> {
    let sl = mslnk::ShellLink::new(target).unwrap();
    sl.create_lnk(source).unwrap();
    Ok(())
}
