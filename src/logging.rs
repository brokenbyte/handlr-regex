use std::collections::BTreeMap;

use tracing::{field::Visit, level_filters::LevelFilter};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer};

use crate::error::Result;

/// Init global tracing subscriber
pub fn init_tracing(show_notifications: bool) -> Result<WorkerGuard> {
    let (file_writer, _guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::never(
            xdg::BaseDirectories::new()?.create_cache_directory("handlr")?,
            "handlr.log",
        ));

    let env_filter = || {
        EnvFilter::builder()
            .with_default_directive(LevelFilter::WARN.into())
            .from_env_lossy()
    };

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(
                fmt::Layer::new()
                    .pretty()
                    .with_writer(std::io::stderr)
                    .with_filter(env_filter()),
            )
            .with(fmt::Layer::new().with_writer(file_writer))
            .with(
                show_notifications
                    .then_some(NotificationLayer.with_filter(env_filter())),
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
    fn on_event(
        &self,
        event: &tracing::Event,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut fields = BTreeMap::new();
        let mut visitor = NotificationVisitor(&mut fields);
        event.record(&mut visitor);
        let message = fields
            .get("message")
            .expect("handlr error: Tracing event did not have any message");

        let (level, icon, urgency, timeout) = match *event.metadata().level() {
            l if l == tracing::Level::ERROR => {
                ("error".to_string(), "dialog-error", "critical", 0)
            }
            tracing::Level::WARN => {
                ("warning".to_string(), "dialog-warning", "normal", 10000)
            }
            l => (
                l.as_str().to_lowercase(),
                "dialog-information",
                "low",
                10000,
            ),
        };

        std::process::Command::new("notify-send")
            .arg("--app-name=handlr")
            .arg(format!("--expire-time={}", timeout))
            .arg(format!("--icon={}", icon))
            .arg(format!("handlr {}", level))
            .arg(format!("--urgency={}", urgency))
            .arg(message)
            .spawn()
            .and_then(|mut c| c.wait())
            .expect("handlr error: Could not run `notify-send`");
    }
}

struct NotificationVisitor<'a>(&'a mut BTreeMap<String, String>);

impl Visit for NotificationVisitor<'_> {
    fn record_debug(
        &mut self,
        field: &tracing::field::Field,
        value: &dyn std::fmt::Debug,
    ) {
        self.0
            .insert(field.name().to_string(), format!("{:?}", value));
    }
}
