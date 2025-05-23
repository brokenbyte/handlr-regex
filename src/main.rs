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
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer};
use tracing_unwrap::ResultExt;

#[mutants::skip] // Cannot test directly at the moment
fn main() {
    // Shell completions
    CompleteEnv::with_factory(|| Cli::command().name("handlr"))
        .completer("handlr")
        .complete();

    let cli = Cli::parse();

    let _guard = init_tracing()
        .expect("handlr error: Could not initialize global tracing subscriber");

    let terminal_output = cli
        .terminal_output
        .unwrap_or(std::io::stdout().is_terminal());

    let show_notifications = !terminal_output && cli.enable_notifications;

    if let Err(e) = run(cli, terminal_output) {
        if show_notifications {
            std::process::Command::new("notify-send")
                .args([
                    "--expire-time=10000",
                    "--icon=dialog-error",
                    &e.to_string(),
                ])
                .spawn()
                .and_then(|mut c| c.wait())
                .expect_or_log("Could not run `notify-send`");
        }
    }
}

/// Init global tracing subscriber
fn init_tracing() -> Result<WorkerGuard> {
    let (file_writer, _guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::never(
            xdg::BaseDirectories::new()?.create_cache_directory("handlr")?,
            "handlr.log",
        ));

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(
                fmt::Layer::new()
                    .pretty()
                    .with_writer(std::io::stderr)
                    .with_filter(
                        EnvFilter::builder()
                            .with_default_directive(LevelFilter::WARN.into())
                            .from_env_lossy(),
                    ),
            )
            .with(fmt::Layer::new().with_writer(file_writer)),
    )?;

    Ok(_guard)
}

/// Run main program logic
#[mutants::skip] // Cannot test directly at the moment
fn run(cli: Cli, terminal_output: bool) -> Result<()> {
    let mut config = Config::new(terminal_output)?;

    let mut stdout = std::io::stdout().lock();

    match cli.command {
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
    }
}
