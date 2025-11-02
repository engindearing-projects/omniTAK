//! REST API client for OmniTAK GUI
//!
//! Handles all communication with the main OmniTAK server via REST API

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
    client: Client,
    auth_token: Option<String>,
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    access_token: String,
    expires_at: String,
    role: String,
}

#[derive(Debug, Deserialize)]
pub struct SystemStatus {
    pub uptime_seconds: u64,
    pub active_connections: usize,
    pub messages_processed: u64,
    pub messages_per_second: f64,
    pub memory_usage_bytes: u64,
    pub active_filters: usize,
    pub version: String,
}

#[derive(Debug, Deserialize)]
struct ConnectionListResponse {
    connections: Vec<ConnectionInfo>,
    total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub protocol: String,
    pub status: String,
    pub priority: Option<u8>,
    pub messages_received: Option<u64>,
    pub messages_sent: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CreateConnectionRequest {
    pub name: String,
    pub address: String,
    pub protocol: String,
    pub priority: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct CreateConnectionResponse {
    id: String,
    message: String,
}

// ============================================================================
// API Client Implementation
// ============================================================================

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: base_url.into(),
            client,
            auth_token: None,
        })
    }

    /// Login and get authentication token
    pub async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let url = format!("{}/api/v1/auth/login", self.base_url);

        let request = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send login request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Login failed ({}): {}", status, error_text);
        }

        let login_response: LoginResponse = response
            .json()
            .await
            .context("Failed to parse login response")?;

        self.auth_token = Some(login_response.access_token);

        Ok(())
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth_token.is_some()
    }

    /// Get system status
    pub async fn get_status(&self) -> Result<SystemStatus> {
        let url = format!("{}/api/v1/status", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .context("Failed to get system status")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Status request failed ({}): {}", status, error_text);
        }

        let status = response
            .json()
            .await
            .context("Failed to parse status response")?;

        Ok(status)
    }

    /// List all connections
    pub async fn list_connections(&self) -> Result<Vec<ConnectionInfo>> {
        let url = format!("{}/api/v1/connections", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .context("Failed to list connections")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("List connections failed ({}): {}", status, error_text);
        }

        let list_response: ConnectionListResponse = response
            .json()
            .await
            .context("Failed to parse connections response")?;

        Ok(list_response.connections)
    }

    /// Create a new connection
    pub async fn create_connection(&self, request: CreateConnectionRequest) -> Result<String> {
        let url = format!("{}/api/v1/connections", self.base_url);

        let mut req = self.client.post(&url).json(&request);

        if let Some(token) = &self.auth_token {
            req = req.bearer_auth(token);
        }

        let response = req
            .send()
            .await
            .context("Failed to create connection")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Create connection failed ({}): {}", status, error_text);
        }

        let create_response: CreateConnectionResponse = response
            .json()
            .await
            .context("Failed to parse create connection response")?;

        Ok(create_response.id)
    }

    /// Delete a connection
    pub async fn delete_connection(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/connections/{}", self.base_url, id);

        let mut request = self.client.delete(&url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .context("Failed to delete connection")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Delete connection failed ({}): {}", status, error_text);
        }

        Ok(())
    }

    /// Health check (no auth required)
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/v1/health", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to check health")?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = ApiClient::new("http://localhost:9443");
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let client = ApiClient::new("http://localhost:9443").unwrap();
        // This will fail if server is not running, which is fine for unit tests
        let _ = client.health_check().await;
    }
}
