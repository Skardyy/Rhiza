use std::io;

use inquire::{Confirm, InquireError, Text};
use walkdir::{DirEntry, WalkDir};

pub fn crawl_directory(dir: &str) -> Result<Vec<String>, InquireError> {
    let mut executables = Vec::new();

    let expanded_dir = shellexpand::full(dir)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
    let expanded_dir = expanded_dir.as_ref();
    let target_extensions = ["exe", "lnk", "url", "bat", "ps1"];

    for entry in WalkDir::new(expanded_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if is_executable(&entry, &target_extensions) && is_user_friendly(&entry) {
            if let Some(path) = entry.path().to_str() {
                let item = path.to_string();
                let wanted = Confirm::new(&format!("Add {} ?", item))
                    .with_default(true)
                    .prompt()?;
                let place_holder = get_name(&entry)?;
                if wanted {
                    let _name = Text::new("how to call it?")
                        .with_default(&place_holder)
                        .prompt()?;
                    executables.push(path.to_string());
                }
            }
        }
    }

    Ok(executables)
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
