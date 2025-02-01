use colored::Colorize;
use lnk_parser::LNKParser;
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey; // Add this import

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub commands: std::collections::HashMap<String, String>,
    pub skipped: Vec<String>,
}

impl Config {
    pub fn expand(&self) -> Vec<String> {
        let mut res = Vec::new();

        for (_key, path) in &self.commands {
            if let Some(lnk_result) = read_shortcut(path) {
                res.push(lnk_result);
            }
        }
        for path in self.skipped.iter() {
            if let Some(lnk_result) = read_shortcut(path) {
                res.push(lnk_result);
            }
        }

        res
    }

    pub fn write(&self) -> Result<(), io::Error> {
        let rhiza_dir = tilde("~/.rhiza").to_string();
        let config_file = Path::new(&rhiza_dir).join("config.json");
        let content = serde_json::to_string_pretty(&self)?;
        fs::write(config_file, content)?;

        Ok(())
    }
}

pub fn copy_src() -> std::io::Result<()> {
    let user_dir = shellexpand::tilde("~").to_string();
    let src = format!("{}\\.rhiza\\src", user_dir);
    let src = Path::new(&src);
    let target = format!(
        "{}\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\rhiza",
        user_dir
    );
    let target = Path::new(&target);

    if target.exists() {
        fs::remove_dir_all(target)?;
    }
    fs::create_dir_all(target)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = target.join(entry.file_name());

        fs::copy(&src_path, &dest_path)?;
    }

    Ok(())
}

pub fn check() -> io::Result<Config> {
    // Check if the .rhiza directory exists
    let rhiza_dir = tilde("~\\.rhiza").to_string();
    let config_file = Path::new(&rhiza_dir).join("config.json");

    let mut needs_setup = false;

    // Check if the .rhiza directory exists
    if !Path::new(&rhiza_dir).exists() {
        println!(
            "{}",
            ".rhiza directory does not exist, running setup...".yellow()
        );
        needs_setup = true;
    }

    // Check if the config.json file exists
    if !config_file.exists() {
        println!(
            "{}",
            "config.json does not exist, running setup...".yellow()
        );
        needs_setup = true;
    }

    // Check if the new path is already in the PATH
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment_key = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
    let current_path: String = environment_key.get_value("Path")?;

    let bin_dir = rhiza_dir + "\\bin";
    if !current_path.split(';').any(|path| path == bin_dir) {
        needs_setup = true;
    }

    if needs_setup {
        setup_rhiza_config()?;
        add_to_path_permanently(&bin_dir)?;
        return check();
    } else {
        let config_contents = fs::read_to_string(&config_file)?;
        let config: Config = serde_json::from_str(&config_contents).unwrap_or_default();
        Ok(config)
    }
}

fn add_to_path_permanently(new_path: &str) -> io::Result<()> {
    // Open the environment variables key in the registry for the current user
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment_key = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

    // Get the current PATH value
    let current_path: String = environment_key.get_value("Path")?;

    // Check if the new path is already in the PATH
    if !current_path.split(';').any(|path| path == new_path) {
        // Append the new path to the existing PATH
        let new_path_value = if current_path.ends_with(';') {
            format!("{}{}", current_path, new_path)
        } else {
            format!("{};{}", current_path, new_path)
        };

        // Set the new PATH value in the registry
        environment_key.set_value("Path", &new_path_value)?;

        // Notify the system that the environment variables have changed
        unsafe {
            winapi::um::winuser::SendMessageTimeoutA(
                winapi::um::winuser::HWND_BROADCAST,
                winapi::um::winuser::WM_SETTINGCHANGE,
                0 as winapi::shared::minwindef::WPARAM,
                "Environment\0".as_ptr() as winapi::shared::minwindef::LPARAM,
                winapi::um::winuser::SMTO_ABORTIFHUNG,
                5000,
                std::ptr::null_mut(),
            );
        }

        let msg = format!("Successfully added '{}' to the PATH.", new_path).green();
        println!("{}", msg);
    } else {
        let msg = format!("'{}' is already in the PATH, skipping.", new_path).yellow();
        println!("{}", msg);
    }

    Ok(())
}

fn setup_rhiza_config() -> io::Result<()> {
    // Resolve the home directory using shellexpand
    let rhiza_dir = tilde("~/.rhiza").to_string();
    let config_file = Path::new(&rhiza_dir).join("config.json");

    // Create the .rhiza directory if it doesn't exist
    if !Path::new(&rhiza_dir).exists() {
        fs::create_dir(&rhiza_dir)?;
        let msg = format!("Created directory: {:?}", rhiza_dir).green();
        println!("{}", msg);
    } else {
        let msg = format!("Directory already exists: {:?}, skipping", rhiza_dir).yellow();
        println!("{}", msg);
    }

    // Create the config.json file if it doesn't exist
    if !config_file.exists() {
        let default_config = r#"{}"#;
        fs::write(&config_file, default_config)?;

        let msg = format!("Created file: {:?}", config_file).green();
        println!("{}", msg);
    } else {
        let msg = format!("File already exists: {:?}, skipping", config_file).yellow();
        println!("{}", msg);
    }

    Ok(())
}

pub fn read_shortcut(lnk_path: &str) -> Option<String> {
    if lnk_path.ends_with(".lnk") {
        if let Ok(link) = LNKParser::from_path(lnk_path) {
            return link.get_target_full_path().clone();
        }
    } else if lnk_path.ends_with(".url") {
        if let Ok(content) = fs::read_to_string(lnk_path) {
            let lines = content.lines();
            let mut in_internet_shortcut_section = false;
            for line in lines {
                if line.trim() == "[InternetShortcut]" {
                    in_internet_shortcut_section = true;
                    continue;
                }

                if in_internet_shortcut_section && line.starts_with("URL=") {
                    let url = line.trim_start_matches("URL=").trim().to_string();
                    return Some(url);
                }
            }
        }
    }
    None
}
pub fn setup_panic_logging() {
    let log_dir = std::path::PathBuf::from(shellexpand::tilde("~/.rhiza").into_owned());
    let log_path = log_dir.join("panic.log");

    if !log_dir.exists() {
        fs::create_dir_all(&log_dir).expect("Failed to create .rhiza directory");
    }

    std::panic::set_hook(Box::new(move |panic_info| {
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(file, "\n[{}] Panic occurred:", timestamp);
            let _ = writeln!(file, "{}\n", panic_info);
        }
    }));
}
