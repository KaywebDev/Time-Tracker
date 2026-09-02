//! Initialization logic for the track application

use crate::error::Suggestion;
use clap_verbosity_flag::Verbosity;
use error_stack::{Report, fmt::ColorMode};
use owo_colors::OwoColorize;
use serde::de::value;
use tracing_log::AsTrace;
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};

pub fn error_reporting() {
    Report::set_color_mode(ColorMode::Color);
    Report::install_debug_hook::<Suggestion>(|value, context| {
        let msg: &str = value.0;
        let body: String = format!("suggestion: {}", msg);

        match context.color_mode() {
            ColorMode::Color => context.push_body(body.cyan().to_string()),
            ColorMode::Emphasis => context.push_body(body.italic().to_string()),
            ColorMode::None => context.push_body(body.to_string()),
        }
    });
}

pub fn tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::fmt::Subscriber;
    use tracing_subscriber::layer::SubscriberExt;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(EnvFilter::builder().from_env_lossy())
        .with(tracing_error::ErrorLayer::default())
        .init();
}
