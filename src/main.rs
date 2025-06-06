mod installer;
mod searcher;
mod worker;

use std::path::Path;

use clap::{
    builder::{styling::AnsiColor, Styles},
    Arg, ColorChoice, Command,
};
use colored::*;
use inquire::Text;

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
                        .index(1)
                        .value_name("PATH")
                        .required(false)
                        .help("Specify a directory to crawl"),
                ),
        )
        .subcommand(
            Command::new("add")
                .about("Search for a single app to add")
                .arg(
                    Arg::new("name")
                        .index(1)
                        .value_name("NAME")
                        .help("name of the app to search for")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("path")
                .about("Search for a single app to add to path")
                .arg(
                    Arg::new("name")
                        .index(1)
                        .value_name("NAME")
                        .help("name of the app to search for")
                        .required(false),
                ),
        )
        .subcommand(Command::new("view").about("View all linked apps and their config"))
        .subcommand(Command::new("edit").about("Edit the config"))
        .subcommand(Command::new("run").about("Create the lnk files"))
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
        Some(("add", subcommand)) => {
            let mut config = installer::check().unwrap();
            let name = subcommand.get_one::<String>("name");

            let res = searcher::prompt_fzf(
                name,
                7,
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
        Some(("path", subcommand)) => {
            let name = subcommand.get_one::<String>("name");

            let res = searcher::prompt_fzf(
                name,
                7,
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
        Some(("edit", _)) => {
            installer::check().unwrap();
            let path = shellexpand::tilde("~\\.rhiza").to_string();
            std::process::Command::new("explorer")
                .arg(path)
                .spawn()
                .unwrap();
        }
        Some(("run", _)) => {
            worker::run().unwrap();
        }
        _ => {
            println!("No subcommand was used. Use --help for more information.");
        }
    }
}
