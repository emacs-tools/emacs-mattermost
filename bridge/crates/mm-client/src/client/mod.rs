use crate::error::ClientError;
use crate::tls;
use mm_core::{ChannelId, Post, Team, User}; // Добавим Post и Team в mm-core позже
use reqwest::header::AUTHORIZATION;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MattermostClient {
    base_url: String,
    client: reqwest::Client,
}

impl MattermostClient {
    pub fn new(base_url: String, ca_file: Option<&Path>) -> Result<Self, ClientError> {
        let builder = tls::builder(ca_file)?;
        let client = builder.build().map_err(ClientError::Http)?;
        let base_url = base_url.trim_end_matches('/').to_string();
        Ok(Self { base_url, client })
    }

    pub async fn login(&self, login_id: &str, password: &str) -> Result<String, ClientError> {
        let url = format!("{}/api/v4/users/login", self.base_url);
        let body = serde_json::json!({
            "login_id": login_id,
            "password": password
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(ClientError::Http)?;

        if !response.status().is_success() {
            return handle_api_error_string(response)
                .await
                .map(|_| unreachable!());
        }

        let token = response
            .headers()
            .get("Token")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .ok_or(ClientError::MissingTokenHeader)?;

        Ok(token)
    }

    pub async fn get_me(&self, token: &str) -> Result<User, ClientError> {
        let url = format!("{}/api/v4/users/me", self.base_url);
        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(ClientError::Http)?;

        if !response.status().is_success() {
            return handle_api_error_user(response).await;
        }

        let user: User = response.json().await.map_err(ClientError::Http)?;
        Ok(user)
    }

    pub async fn get_teams(&self, token: &str) -> Result<Vec<Team>, ClientError> {
        let url = format!("{}/api/v4/teams", self.base_url);
        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(ClientError::Http)?;

        if !response.status().is_success() {
            return handle_api_error_vec(response).await;
        }

        let teams: Vec<Team> = response.json().await.map_err(ClientError::Http)?;
        Ok(teams)
    }

    pub async fn create_direct_channel(
        &self,
        token: &str,
        user_ids: &[String],
    ) -> Result<ChannelId, ClientError> {
        let url = format!("{}/api/v4/channels/direct", self.base_url);
        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .json(user_ids)
            .send()
            .await
            .map_err(ClientError::Http)?;

        if !response.status().is_success() {
            return handle_api_error_channel(response).await;
        }

        #[derive(serde::Deserialize)]
        struct ChannelResponse {
            id: String,
        }

        let ch: ChannelResponse = response.json().await.map_err(ClientError::Http)?;
        Ok(ChannelId::new(ch.id))
    }

    pub async fn get_posts(
        &self,
        token: &str,
        channel_id: &str,
        limit: u32,
    ) -> Result<Vec<Post>, ClientError> {
        let url = format!(
            "{}/api/v4/channels/{}/posts?page=0&per_page={}",
            self.base_url, channel_id, limit
        );

        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(ClientError::Http)?;

        if !response.status().is_success() {
            return handle_api_error_vec(response).await;
        }

        // Mattermost returns { order: [], posts: {} }
        #[derive(serde::Deserialize)]
        struct PostsResponse {
            order: Vec<String>,
            posts: std::collections::HashMap<String, Post>,
        }

        let data: PostsResponse = response.json().await.map_err(ClientError::Http)?;

        // Normalize to ordered vector
        let mut posts: Vec<Post> = data
            .order
            .iter()
            .filter_map(|id| data.posts.get(id).cloned())
            .collect();

        // Sort ascending by create_at for display
        posts.sort_by_key(|p| p.create_at);

        Ok(posts)
    }

    pub async fn create_post(
        &self,
        token: &str,
        channel_id: &str,
        message: &str,
        root_id: Option<&str>,
    ) -> Result<Post, ClientError> {
        let url = format!("{}/api/v4/posts", self.base_url);

        let mut body = serde_json::json!({
            "channel_id": channel_id,
            "message": message
        });

        if let Some(rid) = root_id {
            body["root_id"] = serde_json::Value::String(rid.to_string());
        }

        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .map_err(ClientError::Http)?;

        if !response.status().is_success() {
            return handle_api_error_post(response).await;
        }

        let post: Post = response.json().await.map_err(ClientError::Http)?;
        Ok(post)
    }
}

// --- Helpers for error handling ---

async fn handle_api_error_string(response: reqwest::Response) -> Result<String, ClientError> {
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();
    #[derive(serde::Deserialize)]
    struct MmError {
        id: String,
        message: String,
    }
    let err_msg = serde_json::from_str::<MmError>(&body)
        .map(|e| format!("{} ({})", e.message, e.id))
        .unwrap_or_else(|_| body.clone());
    Err(ClientError::Api {
        id: "api_error".into(),
        message: err_msg,
        status_code: status,
    })
}

async fn handle_api_error_user(response: reqwest::Response) -> Result<User, ClientError> {
    handle_api_error_string(response).await?;
    unreachable!()
}
async fn handle_api_error_vec<T>(response: reqwest::Response) -> Result<Vec<T>, ClientError> {
    handle_api_error_string(response).await?;
    unreachable!()
}
async fn handle_api_error_channel(response: reqwest::Response) -> Result<ChannelId, ClientError> {
    handle_api_error_string(response).await?;
    unreachable!()
}
async fn handle_api_error_post(response: reqwest::Response) -> Result<Post, ClientError> {
    handle_api_error_string(response).await?;
    unreachable!()
}
