mod installer;
mod searcher;
mod worker;
use std::path::Path;

use clap::{
    builder::{styling::AnsiColor, Styles},
    Arg, ColorChoice, Command,
};
use colored::*;
use inquire::{Select, Text};
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
        .subcommand(
            Command::new("crawl")
                .about("Find potential apps to link")
                .arg(
                    Arg::new("path")
                        .short('p')
                        .long("path")
                        .value_name("PATH")
                        .help("Specify a directory to crawl"),
                ),
        )
        .subcommand(Command::new("add").about("Search for a single app to add"))
        .subcommand(Command::new("view").about("View all linked apps and their config"))
        .subcommand(Command::new("edit").about("Edit the config"))
        .subcommand(Command::new("run").about("Create the lnk files"))
        .subcommand(Command::new("install").about("Creates the config and adds to the path"))
        .get_matches();

    match matches.subcommand() {
        Some(("crawl", sub_matches)) => {
            let path = sub_matches.get_one::<String>("path");

            let dirs = match path {
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
            let name = Text::new("what to search for?").prompt().unwrap();

            let optimizer = searcher::FileSearchOptimizer::new();
            let matches = optimizer.find_top_matches(&name, 5);

            let options = matches
                .iter()
                .map(|f| format!("{} {}", f.path.display(), f.formatted_last_modified()))
                .collect();

            let ans = Select::new("choose the best match", options)
                .prompt()
                .unwrap();
            let ans = ans.split_once("‌").map(|(name, _)| name).unwrap();
            if let Ok(name) = Text::new("what to call that?").prompt() {
                config.commands.insert(name, ans.to_string());
                config.write().unwrap();
                println!("{}", "Do 'rhz run' to apply the changes".purple().bold())
            }
        }
        Some(("view", _)) => {
            let config = installer::check().unwrap();
            let content = serde_json::to_string_pretty(&config.commands).unwrap();
            println!("{}", content)
        }
        Some(("edit", _)) => {
            installer::check().unwrap();
            let path = shellexpand::tilde("~\\.rhiza").to_string();
            println!("{}", path);
            std::process::Command::new("explorer")
                .arg(path)
                .spawn()
                .unwrap();
        }
        Some(("run", _)) => {
            worker::run().unwrap();
        }
        Some(("install", _)) => {
            installer::check().unwrap();

            let current_exe = std::env::current_exe().unwrap();
            let path = shellexpand::tilde("~\\.rhiza\\bin").to_string();
            let path = Path::new(&path);
            let destination_path = path.join("rhz.exe");
            std::fs::copy(&current_exe, &destination_path).unwrap();

            println!(
                "{}",
                "Finished installing, you can run rhz now".purple().bold()
            )
        }
        _ => {
            println!("No subcommand was used. Use --help for more information.");
        }
    }
}
