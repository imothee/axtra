//! Error notification handlers for Slack and Discord

#[cfg(any(feature = "notify-error-slack", feature = "notify-error-discord"))]
use crate::notifier::Notifier;

#[cfg(any(feature = "notify-error-slack", feature = "notify-error-discord"))]
use std::sync::OnceLock;

// Notification Clients
#[cfg(feature = "notify-error-slack")]
static SLACK_NOTIFIER: OnceLock<Option<Notifier>> = OnceLock::new();

#[cfg(feature = "notify-error-slack")]
pub fn slack_notifier() -> Option<&'static Notifier> {
    SLACK_NOTIFIER
        .get_or_init(|| {
            std::env::var("SLACK_ERROR_WEBHOOK_URL")
                .ok()
                .map(Notifier::with_slack)
        })
        .as_ref()
}

#[cfg(feature = "notify-error-discord")]
static DISCORD_NOTIFIER: OnceLock<Option<Notifier>> = OnceLock::new();

#[cfg(feature = "notify-error-discord")]
pub fn discord_notifier() -> Option<&'static Notifier> {
    DISCORD_NOTIFIER
        .get_or_init(|| {
            std::env::var("DISCORD_ERROR_WEBHOOK_URL")
                .ok()
                .map(Notifier::with_discord)
        })
        .as_ref()
}
