//! # Notifier
//!
//! Send error alerts and notifications to Slack and Discord webhooks.
//!
//! ## Feature Flag
//!
//! This module is only available when the `notifier` feature is enabled.
//!
//! ## Usage
//!
//! ### Instance API
//!
//! ```rust
//! use axtra::notifier::Notifier;
//! use serde_json::json;
//!
//! // Create a notifier for Slack
//! let slack = Notifier::with_slack("https://hooks.slack.com/services/XXX");
//! slack.notify_slack("Hello from Axtra!").await?;
//!
//! // Send rich Slack blocks
//! let blocks = json!([{ "type": "section", "text": { "type": "plain_text", "text": "Critical error!" } }]);
//! slack.notify_slack_rich(blocks).await?;
//!
//! // Create a notifier for Discord
//! let discord = Notifier::with_discord("https://discord.com/api/webhooks/XXX");
//! discord.notify_discord("Hello from Axtra!").await?;
//!
//! // Send rich Discord embeds
//! let embeds = json!([{ "title": "Error", "description": "Something went wrong!" }]);
//! discord.notify_discord_rich(embeds).await?;
//! ```
//!
//! ### Static API (One-off notifications)
//!
//! ```rust
//! use axtra::notifier::Notifier;
//! use serde_json::json;
//!
//! // Send a one-off Slack message
//! Notifier::slack("https://hooks.slack.com/services/XXX", "Hello!").await?;
//!
//! // Send a one-off rich Slack message
//! let blocks = json!([{ "type": "section", "text": { "type": "plain_text", "text": "Critical error occurred!" } }]);
//! Notifier::slack_rich("https://hooks.slack.com/services/XXX", blocks).await?;
//!
//! // Send a one-off Discord message
//! Notifier::discord("https://discord.com/api/webhooks/XXX", "Hello!").await?;
//!
//! // Send a one-off rich Discord message
//! let embeds = json!([{ "title": "Error", "description": "Something went wrong!", "color": 16711680 }]);
//! Notifier::discord_rich("https://discord.com/api/webhooks/XXX", embeds).await?;
//! ```
//!
//! ## Environment Variables
//!
//! You can configure webhook URLs via environment variables for automatic integration:
//!
//! ```text
//! SLACK_ERROR_WEBHOOK_URL=your_slack_webhook_url
//! DISCORD_ERROR_WEBHOOK_URL=your_discord_webhook_url
//! ```
//!
//! ## See Also
//! - [README](https://github.com/imothee/axtra)
//! - [docs.rs/axtra](https://docs.rs/axtra)
//!

#[cfg(feature = "notifier")]
use reqwest::Client;
#[cfg(feature = "notifier")]
use serde_json::Value;

#[cfg(feature = "notifier")]
pub struct Notifier {
    client: Client,
    slack_webhook: Option<String>,
    discord_webhook: Option<String>,
}

#[cfg(feature = "notifier")]
impl Default for Notifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "notifier")]
impl Notifier {
    /// Create a new notifier with specific webhook URLs
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            slack_webhook: None,
            discord_webhook: None,
        }
    }

    /// Create a notifier with Slack webhook
    pub fn with_slack(webhook_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            slack_webhook: Some(webhook_url.into()),
            discord_webhook: None,
        }
    }

    /// Create a notifier with Discord webhook
    pub fn with_discord(webhook_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            slack_webhook: None,
            discord_webhook: Some(webhook_url.into()),
        }
    }

    /// Create a notifier with both webhooks
    pub fn with_both(slack_url: impl Into<String>, discord_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            slack_webhook: Some(slack_url.into()),
            discord_webhook: Some(discord_url.into()),
        }
    }

    // --- Instance methods (reuse the webhook URLs) ---

    /// Send simple text to Slack using stored webhook
    pub async fn notify_slack(
        &self,
        message: impl AsRef<str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let webhook_url = self
            .slack_webhook
            .as_ref()
            .ok_or("No Slack webhook configured")?;

        let payload = serde_json::json!({ "text": message.as_ref() });
        self.send(webhook_url, payload).await.map_err(Into::into)
    }

    /// Send rich blocks to Slack using stored webhook
    pub async fn notify_slack_rich(
        &self,
        blocks: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let webhook_url = self
            .slack_webhook
            .as_ref()
            .ok_or("No Slack webhook configured")?;

        let payload = serde_json::json!({ "blocks": blocks });
        self.send(webhook_url, payload).await.map_err(Into::into)
    }

    /// Send simple text to Discord using stored webhook
    pub async fn notify_discord(
        &self,
        message: impl AsRef<str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let webhook_url = self
            .discord_webhook
            .as_ref()
            .ok_or("No Discord webhook configured")?;

        let payload = serde_json::json!({ "content": message.as_ref() });
        self.send(webhook_url, payload).await.map_err(Into::into)
    }

    /// Send rich embeds to Discord using stored webhook
    pub async fn notify_discord_rich(
        &self,
        embeds: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let webhook_url = self
            .discord_webhook
            .as_ref()
            .ok_or("No Discord webhook configured")?;

        let payload = serde_json::json!({ "embeds": embeds });
        self.send(webhook_url, payload).await.map_err(Into::into)
    }

    // Use internal client to send the payload
    async fn send(&self, webhook_url: &str, payload: Value) -> Result<(), reqwest::Error> {
        self.client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    // --- Static methods (one-off notifications) ---

    /// Send simple text to Slack (static method)
    pub async fn slack(
        webhook_url: impl AsRef<str>,
        message: impl AsRef<str>,
    ) -> Result<(), reqwest::Error> {
        let payload = serde_json::json!({ "text": message.as_ref() });
        Self::send_static(webhook_url.as_ref(), payload).await
    }

    /// Send rich blocks to Slack (static method)
    pub async fn slack_rich(
        webhook_url: impl AsRef<str>,
        blocks: Value,
    ) -> Result<(), reqwest::Error> {
        let payload = serde_json::json!({ "blocks": blocks });
        Self::send_static(webhook_url.as_ref(), payload).await
    }

    /// Send simple text to Discord (static method)
    pub async fn discord(
        webhook_url: impl AsRef<str>,
        message: impl AsRef<str>,
    ) -> Result<(), reqwest::Error> {
        let payload = serde_json::json!({ "content": message.as_ref() });
        Self::send_static(webhook_url.as_ref(), payload).await
    }

    /// Send rich embeds to Discord (static method)
    pub async fn discord_rich(
        webhook_url: impl AsRef<str>,
        embeds: Value,
    ) -> Result<(), reqwest::Error> {
        let payload = serde_json::json!({ "embeds": embeds });
        Self::send_static(webhook_url.as_ref(), payload).await
    }

    // Internal helper
    async fn send_static(webhook_url: &str, payload: Value) -> Result<(), reqwest::Error> {
        Client::new()
            .post(webhook_url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
