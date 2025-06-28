mod installer;
mod searcher;
mod worker;

use std::{path::Path, sync::mpsc::channel};

use clap::{
    builder::{styling::AnsiColor, Styles},
    Arg, ColorChoice, Command,
};
use colored::*;
use inquire::Text;
use searcher::prompt_fzf;

fn main() {
    installer::setup_panic_logging();
    let matches = Command::new("Rhiza")
        .version("1.0")
        .about("A blazingly fast app linker for Windows 🚀")
        .color(ColorChoice::Always)
        .styles(
            Styles::styled()
                .header(AnsiColor::Green.on_default().bold())
                .literal(AnsiColor::Blue.on_default()),
        )
        .subcommand(Command::new("crawl").about("Find potential apps to link"))
        .subcommand(Command::new("add").about("Search for a single app to add"))
        .subcommand(Command::new("path").about("Search for a single app to add to path"))
        .subcommand(Command::new("rm").about("Removed an key added by rhiza"))
        .subcommand(Command::new("run").about("Create the lnk files"))
        .subcommand(Command::new("view").about("View all linked apps and their config"))
        .subcommand(
            Command::new("clear-skipped")
                .about("Clear the skipped config created the the crawl command"),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("crawl", sub_matches)) => {
            let dirs = match sub_matches.get_one::<String>("path") {
                None => vec![
                    "~\\Desktop",
                    "~\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu",
                    "C:\\ProgramData\\Microsoft\\Windows\\Start Menu",
                ],
                Some(p) => {
                    let dir: &str = &p;
                    vec![dir]
                }
            };
            let _ = worker::crawl_directory(dirs);
            println!("{}", "Do 'rhz run' to apply the changes".purple().bold())
        }
        Some(("add", _)) => {
            let mut config = installer::check().unwrap();

            let res = searcher::search_prompt_fzf(
                "Select app to add:\n",
                vec!["exe".to_string(), "lnk".to_string(), "url".to_string()],
            );

            if let Some(path) = res {
                if let Ok(name) = Text::new("what to call that?").prompt() {
                    config.commands.insert(name, path);
                    config.write().unwrap();
                    println!("{}", "Do 'rhz run' to apply the changes".purple().bold())
                }
            }
        }
        Some(("path", _)) => {
            let res = searcher::search_prompt_fzf(
                "Select path to add:\n",
                vec!["ps1".to_string(), "exe".to_string()],
            );

            if let Some(path) = res {
                let base_dir = Path::new(&path).parent();
                if let Some(dir) = base_dir {
                    let dir = &dir.to_string_lossy().to_string();
                    installer::add_to_path_permanently(dir).unwrap();
                }
            }
        }
        Some(("view", _)) => {
            let config = installer::check().unwrap();
            let content = serde_json::to_string_pretty(&config.commands).unwrap();
            println!("{}", content)
        }
        Some(("rm", _)) => {
            let mut config = installer::check().unwrap();
            let items: Vec<String> = config.commands.keys().cloned().collect();
            let (tx, rx) = channel::<String>();
            for item in items {
                tx.send(item).unwrap();
            }
            drop(tx);
            let key = prompt_fzf(rx, "Select key to remove").expect("Failed to get key for rm");

            worker::remove_key(&key).unwrap();
            config.commands.remove(&key);
            config.write().unwrap();
        }
        Some(("run", _)) => {
            worker::run().unwrap();
        }
        Some(("clear-skipped", _)) => {
            let mut config = installer::check().unwrap();
            config.skipped.clear();
            config.write().unwrap();
            println!("Cleared!")
        }
        _ => {
            println!("No subcommand was used. Use --help for more information.");
        }
    }
}
