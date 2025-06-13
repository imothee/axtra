#[cfg(feature = "notifier")]
use reqwest::Client;
#[cfg(feature = "notifier")]
use serde_json::Value;

#[cfg(feature = "notifier")]
pub enum NotifierKind {
    Slack,
    Discord,
}

#[cfg(feature = "notifier")]
pub struct Notifier {
    client: Client,
    webhook_url: String,
    kind: NotifierKind,
}

#[cfg(feature = "notifier")]
impl Notifier {
    pub fn new(webhook_url: impl Into<String>, kind: NotifierKind) -> Self {
        Self {
            client: Client::new(),
            webhook_url: webhook_url.into(),
            kind,
        }
    }

    pub async fn notify(&self, message: &str) -> Result<(), reqwest::Error> {
        let payload = match self.kind {
            NotifierKind::Slack => serde_json::json!({ "text": message }),
            NotifierKind::Discord => serde_json::json!({ "content": message }),
        };

        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn notify_slack(
        &self,
        message: &str,
        blocks: Option<Value>,
    ) -> Result<(), reqwest::Error> {
        let mut payload = serde_json::json!({ "text": message });
        if let Some(blocks) = blocks {
            payload["blocks"] = blocks;
        }
        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn notify_discord(
        &self,
        message: &str,
        embeds: Option<Value>,
    ) -> Result<(), reqwest::Error> {
        let mut payload = serde_json::json!({ "content": message });
        if let Some(embeds) = embeds {
            payload["embeds"] = embeds;
        }
        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
