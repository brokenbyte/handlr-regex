mod apps;
mod cli;
mod common;
mod config;
mod error;

use std::io::IsTerminal;

use cli::{Cli, Cmd};
use common::mime_table;
use config::Config;
use error::Result;

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;

#[mutants::skip] // Cannot test directly at the moment
fn main() -> Result<()> {
    CompleteEnv::with_factory(|| Cli::command().name("handlr"))
        .completer("handlr")
        .complete();

    let terminal_output = std::io::stdout().is_terminal();
    let Cli {
        command,
        notifications,
    } = Cli::parse();

    let show_notifications = !terminal_output && notifications;

    let mut config = notify_on_err(
        Config::new(terminal_output),
        "handlr config error",
        show_notifications,
    )?;

    let mut stdout = std::io::stdout().lock();

    let res = match command {
        Cmd::Set { mime, handler } => config.set_handler(&mime, &handler),
        Cmd::Add { mime, handler } => config.add_handler(&mime, &handler),
        Cmd::Launch {
            mime,
            args,
            selector_args,
        } => {
            config.override_selector(selector_args);
            config.launch_handler(&mime, args)
        }
        Cmd::Get {
            mime,
            json,
            selector_args,
        } => {
            config.override_selector(selector_args);
            config.show_handler(&mut stdout, &mime, json)
        }
        Cmd::Open {
            paths,
            selector_args,
        } => {
            config.override_selector(selector_args);
            config.open_paths(&paths)
        }
        Cmd::Mime { paths, json } => {
            mime_table(&mut stdout, &paths, json, config.terminal_output)
        }
        Cmd::List { all, json } => config.print(&mut stdout, all, json),
        Cmd::Unset { mime } => config.unset_handler(&mime),
        Cmd::Remove { mime, handler } => config.remove_handler(&mime, &handler),
    };

    notify_on_err(res, "handlr error", show_notifications)
}

/// Issue a notification if given an error and not running in a terminal
#[mutants::skip] // Cannot test directly, runs external command
pub fn notify_on_err<T>(
    res: Result<T>,
    title: &str,
    show_notifications: bool,
) -> Result<T> {
    if show_notifications {
        if let Err(ref e) = res {
            std::process::Command::new("notify-send")
                .args([
                    "--expire-time=10000",
                    "--icon=dialog-error",
                    title,
                    &e.to_string(),
                ])
                .spawn()?;
        }
    }

    res
}
