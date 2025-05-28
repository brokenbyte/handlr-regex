use notify_rust::{Notification, Timeout, Urgency};
use tracing::{field::Visit, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{filter, fmt, layer::SubscriberExt, EnvFilter, Layer};

use crate::{cli::Cli, error::Result};

/// Init global tracing subscriber
pub fn init_tracing(cli: &Cli) -> Result<WorkerGuard> {
    let (file_writer, _guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::never(
            xdg::BaseDirectories::new()?.create_cache_directory("handlr")?,
            "handlr.log",
        ));

    // Have log level for certain layers be determined by cli arguments
    let env_filter = || {
        EnvFilter::builder()
            .with_default_directive(cli.verbosity.tracing_level_filter().into())
            .from_env_lossy()
    };

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            // Send logs to stdout as determined by cli args
            .with(
                fmt::Layer::new()
                    .with_writer(std::io::stderr)
                    .with_filter(env_filter()),
            )
            // Send all logs to a log file
            .with(fmt::Layer::new().with_writer(file_writer).with_ansi(false))
            // Notify for logs as determined by cli args
            .with(
                cli.show_notifications()
                    .then_some(NotificationLayer.with_filter(env_filter())),
            )
            // Filter out all logs from other crates so the user is not overwhelmed looking at the logs
            .with(
                filter::Targets::new()
                    .with_target("handlr", Level::TRACE)
                    .with_target("tracing_unwrap", Level::WARN),
            ),
    )?;

    Ok(_guard)
}

/// Custom tracing layer for running a notification on relevant events
struct NotificationLayer;

impl<S> Layer<S> for NotificationLayer
where
    S: tracing::Subscriber,
{
    #[mutants::skip] // Cannot test, relies on dbus
    fn on_event(
        &self,
        event: &tracing::Event,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut message = String::new();
        event.record(&mut NotificationVisitor(&mut message));

        // Just in case a message has no message, but some other information
        if message.is_empty() {
            message = "No message, see ~/.cache/handlr/handlr.log for details"
                .to_string()
        }

        let (level, icon, urgency) = match *event.metadata().level() {
            tracing::Level::ERROR => {
                ("error".to_string(), "dialog-error", Urgency::Critical)
            }
            tracing::Level::WARN => {
                ("warning".to_string(), "dialog-warning", Urgency::Normal)
            }
            l => (
                l.as_str().to_lowercase(),
                "dialog-information",
                Urgency::Low,
            ),
        };

        Notification::new()
            .summary(&format!("handlr {}", level))
            .body(&message)
            .icon(icon)
            .appname("handlr")
            .timeout(Timeout::Milliseconds(10_000))
            .urgency(urgency)
            .show()
            .expect("handlr error: Could not issue dbus notification");
    }
}

struct NotificationVisitor<'a>(&'a mut String);

impl Visit for NotificationVisitor<'_> {
    #[mutants::skip] // Cannot test independently of NotificationLayer
    fn record_debug(
        &mut self,
        field: &tracing::field::Field,
        value: &dyn std::fmt::Debug,
    ) {
        if field.name() == "message" {
            self.0.push_str(&format!("{:?}", value));
        }
    }
}
