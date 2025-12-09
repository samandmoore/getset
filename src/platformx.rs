use crate::config::PlatformXConfig;
use chrono::Utc;
use serde_json::json;
use std::{collections::HashMap, time::Duration};

const PLATFORMX_API_URL: &str = "https://api.getdx.com/events.track";

pub struct PlatformXClient {
    config: PlatformXConfig,
    client: reqwest::Client,
    namespace: String,
    globals: Globals,
}

impl PlatformXClient {
    pub fn new(config: PlatformXConfig, globals: Globals) -> Self {
        let namespace = config
            .event_namespace
            .clone()
            .unwrap_or_else(|| "getset".to_string());

        Self {
            config,
            client: reqwest::Client::new(),
            namespace,
            globals,
        }
    }

    /// Send an event to PlatformX
    pub async fn send_event(
        &self,
        event_name: &str,
        metadata: &mut HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        let timestamp = Utc::now().timestamp();

        metadata.insert("user_shell".to_string(), json!(self.globals.user_shell));

        let payload = json!({
            "name": event_name,
            "metadata": metadata,
            "timestamp": timestamp.to_string(),
            "email": self.globals.git_email,
            "github_username": self.globals.github_username,
        });

        log::info!("Sending event to PlatformX: {}", payload);

        let result = self
            .client
            .post(PLATFORMX_API_URL)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.secret_key),
            )
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send PlatformX event: {}", e))?;

        log::debug!("PlatformX HTTP Result: {}", result.status());

        if result.status().is_success() {
            Ok(())
        } else {
            Err(format!(
                "PlatformX API call failed with status: {}",
                result.status()
            ))
        }
    }

    /// Send start event
    pub async fn send_start(&self) -> Result<(), String> {
        let mut metadata = HashMap::new();
        let event_name = format!("{}.start", self.namespace);
        self.send_event(&event_name, &mut metadata).await
    }

    /// Send complete event
    pub async fn send_complete(&self, duration: Duration) -> Result<(), String> {
        let mut metadata = HashMap::new();
        metadata.insert("duration".to_string(), json!(duration.as_secs()));

        let event_name = format!("{}.complete", self.namespace);
        self.send_event(&event_name, &mut metadata).await
    }

    /// Send error event
    pub async fn send_error(
        &self,
        duration: Duration,
        error_message: String,
    ) -> Result<(), String> {
        let mut metadata = HashMap::new();
        metadata.insert("duration".to_string(), json!(duration.as_secs()));
        metadata.insert("error_message".to_string(), json!(error_message));

        let event_name = format!("{}.error", self.namespace);
        self.send_event(&event_name, &mut metadata).await
    }
}

/// Default metadata collected from the user's environment
#[derive(Clone, Debug)]
pub struct Globals {
    pub user_shell: String,
    pub github_username: String,
    pub git_email: String,
}

/// Get default metadata from the user's environment and git config
pub fn get_globals() -> Globals {
    let user_shell = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());

    let github_username = std::process::Command::new("git")
        .args(["config", "get", "github.user"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let git_email = std::process::Command::new("git")
        .args(["config", "get", "user.email"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    Globals {
        user_shell,
        github_username,
        git_email,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_globals() -> Globals {
        Globals {
            user_shell: "/bin/bash".to_string(),
            github_username: "testuser".to_string(),
            git_email: "test@example.com".to_string(),
        }
    }

    fn create_test_config(namespace: Option<String>) -> PlatformXConfig {
        PlatformXConfig {
            secret_key: "test_secret_key".to_string(),
            event_namespace: namespace,
        }
    }

    #[test]
    fn test_get_globals_returns_globals() {
        // This test verifies that get_globals() returns a Globals struct
        // The actual values will depend on the environment
        let globals = get_globals();

        // Just verify the fields exist and are not empty
        // (they may be "unknown" if environment variables are not set)
        assert!(!globals.user_shell.is_empty());
        assert!(!globals.github_username.is_empty());
        assert!(!globals.git_email.is_empty());
    }

    #[test]
    fn test_platformx_client_new_with_default_namespace() {
        let config = create_test_config(None);
        let globals = create_test_globals();

        let client = PlatformXClient::new(config.clone(), globals.clone());

        assert_eq!(client.namespace, "getset");
        assert_eq!(client.config.secret_key, config.secret_key);
        assert_eq!(client.globals.user_shell, globals.user_shell);
    }

    #[test]
    fn test_platformx_client_new_with_custom_namespace() {
        let config = create_test_config(Some("custom_namespace".to_string()));
        let globals = create_test_globals();

        let client = PlatformXClient::new(config.clone(), globals.clone());

        assert_eq!(client.namespace, "custom_namespace");
        assert_eq!(client.config.secret_key, config.secret_key);
    }

    #[test]
    fn test_platformx_client_with_empty_namespace() {
        let config = create_test_config(Some("".to_string()));
        let globals = create_test_globals();

        let client = PlatformXClient::new(config, globals);

        // Empty string should be used as-is, not replaced with default
        assert_eq!(client.namespace, "");
    }

    #[test]
    fn test_api_url_constant() {
        assert_eq!(PLATFORMX_API_URL, "https://api.getdx.com/events.track");
    }
}
