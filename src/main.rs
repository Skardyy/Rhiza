mod installer;
mod worker;
use clap::{
    builder::{styling::AnsiColor, Styles},
    Arg, ColorChoice, Command,
};
use colored::*;

fn main() {
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
        .subcommand(
            Command::new("add")
                .about("Add an app to link manually")
                .arg(
                    Arg::new("path")
                        .short('p')
                        .long("path")
                        .value_name("PATH")
                        .required(true)
                        .help("Path to the app to add"),
                ),
        )
        .subcommand(Command::new("view").about("View all linked apps and their config"))
        .subcommand(Command::new("edit").about("Edit the config"))
        .subcommand(Command::new("run").about("Create the lnk files"))
        .subcommand(Command::new("install").about("Creates the config and adds to the path"))
        .get_matches();

    match matches.subcommand() {
        Some(("crawl", sub_matches)) => {
            let path = sub_matches.get_one::<String>("path");

            if let Some(_p) = path {
            } else {
                let _items = worker::crawl_directory(vec![
                    "~\\Desktop",
                    "~\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu",
                    "C:\\ProgramData\\Microsoft\\Windows\\Start Menu",
                ]);
                println!("{}", "Do 'rhz run' to apply the changes".purple().bold())
            }
            // Call your crawl function here
        }
        Some(("add", sub_matches)) => {
            let path = sub_matches.get_one::<String>("path").unwrap();
            println!("Adding app at path: {}", path);
            // Call your add function here
        }
        Some(("view", _)) => {
            println!("Viewing linked apps");
            // Call your view function here
        }
        Some(("edit", _)) => {
            println!("Editing config");
            // Call your edit function here
        }
        Some(("run", _)) => {
            worker::run().unwrap();
        }
        Some(("install", _)) => {
            println!("Installing");
            // Call your run function here
        }
        _ => {
            println!("No subcommand was used. Use --help for more information.");
        }
    }
}
