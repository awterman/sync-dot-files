use ansi_term::{ANSIString, Color};
use clap::{arg, command, Command};
use log::{Level, LevelFilter};
use std::io::Write;

pub mod app;
pub mod config;
pub mod sh;

fn main() {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let level = record.level();
            let message = format!("{}", record.args());
            let message = match level {
                Level::Error => Color::Red.paint(message),
                Level::Warn => Color::Yellow.paint(message),
                Level::Info => Color::Green.paint(message),
                _ => ANSIString::from(message),
            };

            writeln!(buf, "{message}")
        })
        .filter_level(LevelFilter::Trace)
        .init();

    let app = app::App::new().expect("Failed to create app");

    let cmd = Command::new("sync-dot-files")
        .subcommand(
            command!("init")
                .about("Initialize the repository")
                .arg(arg!(github_account: [String])),
        )
        .subcommand(
            command!("add")
                .about("Add a dotfile to the repository")
                .arg(arg!(dotfile: [String])),
        )
        .subcommand(command!("is-clean").about("Check if the repository is clean"))
        .subcommand(command!("is-synced").about("Check if the repository is synced"))
        .subcommand(command!("repo-path").about("Get the local repository path"))
        .subcommand(command!("sync").about("Sync the repository"));

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            let github_account = sub_matches
                .get_one::<String>("github_account")
                .expect("No GitHub account was provided");
            println!("Initializing the repository for {}", github_account);

            app.init(github_account)
                .expect("Failed to initialize the repository");
        }
        Some(("add", sub_matches)) => {
            let dotfile = sub_matches
                .get_one::<String>("dotfile")
                .expect("No dotfile was provided");
            println!("Adding the dotfile {}", dotfile);

            app.add_dotfile(dotfile).expect("Failed to add the dotfile");
        }
        Some(("is-clean", _)) => {
            println!("Checking if the repository is clean");

            if app
                .is_clean()
                .expect("Failed to check if the repository is clean")
            {
                log::info!("The repository is clean");

                // exit with a successful status code
                std::process::exit(0);
            } else {
                log::error!("The repository is not clean");

                // exit with a failure status code
                std::process::exit(1);
            }
        }
        Some(("is-synced", _)) => {
            println!("Checking if the repository is synced");

            if app
                .is_synced()
                .expect("Failed to check if the repository is synced")
            {
                log::info!("The repository is synced");

                // exit with a successful status code
                std::process::exit(0);
            } else {
                log::error!("The repository is not synced");

                // exit with a failure status code
                std::process::exit(1);
            }
        }
        Some(("repo-path", _)) => {
            let repo_path = app
                .get_local_repo_path()
                .expect("Failed to get the local repository path")
                .trim()
                .to_string();
            println!("{repo_path}");
        }
        Some(("sync", _)) => {
            println!("Syncing the repository");

            app.sync().expect("Failed to sync the repository");
        }
        _ => {
            log::trace!("Checking clean");
            let is_clean = app
                .is_clean()
                .expect("Failed to check if the repository is clean");

            if is_clean {
                log::info!("Clean");
            } else {
                log::error!("Not clean");
            }

            log::trace!("Checking synced");
            let is_synced = app
                .is_synced()
                .expect("Failed to check if the repository is synced");

            if is_synced {
                log::info!("Synced");
            } else {
                log::error!("Not synced");
            }

            if is_clean && is_synced {
                // exit with a successful status code
                std::process::exit(0);
            } else {
                // exit with a failure status code
                std::process::exit(1);
            }
        }
    }
}
