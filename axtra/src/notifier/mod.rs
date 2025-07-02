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
