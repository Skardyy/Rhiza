use colored::Colorize;
use inquire::{Confirm, InquireError, MultiSelect, Text};
use shellexpand::tilde;
use std::{
    fmt, fs,
    io::{self, Write},
    path::Path,
};
use walkdir::{DirEntry, WalkDir};

use crate::installer;

pub fn crawl_directory(dirs: Vec<&str>) -> Result<Vec<String>, InquireError> {
    let executables = Vec::new();
    let mut config = installer::check()?;
    let mut abs_skips = config.expand();
    let mut candidates = Vec::new();

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
        "rhiza",
    ];
    for dir in dirs {
        let expanded_dir = shellexpand::full(dir)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
        let expanded_dir = expanded_dir.as_ref();

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
                    let path = path.to_string();
                    let path_exists = config.commands.values().any(|cmd| cmd == &path);
                    let skipped_before = config.skipped.contains(&path.to_owned());
                    let expanded_exists = file_exists(&abs_skips, &path)?;
                    if path_exists || skipped_before || expanded_exists {
                        continue;
                    }
                    candidates.push(path);
                }
            }
        }
    }

    if candidates.is_empty() {
        return Ok(vec![]);
    }

    let selected = MultiSelect::new("Select apps to add:\n", candidates.clone())
        .without_filtering()
        .with_vim_mode(true)
        .prompt()?;

    // adds selected
    for path in selected.clone() {
        let entry = Path::new(&path);
        let place_holder = get_name(&entry)?;

        let prompt = format!("for \x1b[35m{}\x1b[0m\nhow to call it?", path);
        let name = Text::new(&prompt).with_default(&place_holder).prompt()?;

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
        config.commands.insert(name.clone(), path.to_string());
        // make sure it won't reappear
        if let Some(abs_path) = installer::read_shortcut(&path) {
            abs_skips.push(abs_path);
        }
    }
    // removes not selected
    let remove = Confirm::new(
        "\x1b[35m\x1b[1mWould you like to hide the unselected apps from future selections?\x1b[0m",
    )
    .with_default(true)
    .prompt()?;
    if remove {
        for path in candidates {
            if !selected.contains(&path) {
                // make sure it won't reappear
                config.skipped.push(path.to_string());
                if let Some(abs_path) = installer::read_shortcut(&path) {
                    abs_skips.push(abs_path);
                }
            }
        }
    }

    match config.write() {
        Ok(_) => return Ok(executables),
        Err(err) => return Err(InquireError::IO(err)),
    }
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
    let rhiza_src = tilde("~\\.rhiza\\src").to_string();
    let rhiza_bin = tilde("~\\.rhiza\\bin").to_string();

    // Ensure bin directory exists
    fs::create_dir_all(&rhiza_src)?;

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
                let target_path = Path::new(&rhiza_src).join(&target_name);

                if !target_path.exists() {
                    fs::copy(source_path, &target_path)?;
                    println!(
                        "{} {} -> {}",
                        "Created".green(),
                        key.bold(),
                        source_path.display()
                    );
                }
            }
            Some("exe") => {
                // For .exe files, we create a shortcut
                let target_name = format!("{}.lnk", key);
                let target_path = Path::new(&rhiza_src).join(&target_name);

                if !target_path.exists() {
                    create_shortcut(source_path, &target_path)?;
                    println!(
                        "{} {} -> {}",
                        "Created shortcut".green(),
                        key,
                        source_path.display()
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

    generate_batch_files(&rhiza_src, &rhiza_bin)?;
    println!("{}", "Done writing bat files".purple());
    installer::copy_src()?;

    Ok(())
}

fn clean_dir(target: &Path) -> io::Result<()> {
    if !target.exists() {
        fs::create_dir_all(target)?;
        return Ok(());
    }

    for entry in fs::read_dir(target)? {
        let entry = entry?;
        let path = entry.path();

        fs::remove_file(&path)?;
    }

    Ok(())
}

fn generate_batch_files(src_dir: &str, dst_dir: &str) -> io::Result<()> {
    let target = Path::new(dst_dir);
    clean_dir(target)?;

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(filename) = path.file_stem() {
                let bat_filename = format!("{}.bat", filename.to_string_lossy());
                let bat_path = Path::new(dst_dir).join(bat_filename);

                // Create .bat file content
                let lnk_path = path.to_string_lossy();
                let bat_content = format!("@echo off\nstart /B \"\" \"{}\" %*", lnk_path);

                // Write .bat file
                let mut bat_file = fs::File::create(bat_path)?;
                bat_file.write_all(bat_content.as_bytes())?;
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

fn get_name(file: &Path) -> Result<String, io::Error> {
    let file_name = file
        .file_name()
        .unwrap()
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
    let sl = mslnk::ShellLink::new(source).unwrap();
    sl.create_lnk(target).unwrap();
    Ok(())
}
