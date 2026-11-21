use crate::error::ClientError;
use crate::tls;
use mm_core::{Channel, ChannelId, Post, User};
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
            let err = api_error(response).await;
            return Err(err);
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
            return Err(api_error(response).await);
        }

        let user: User = response.json().await.map_err(ClientError::Http)?;
        Ok(user)
    }

    pub async fn get_user(&self, token: &str, user_id: &str) -> Result<User, ClientError> {
        let url = format!("{}/api/v4/users/{}", self.base_url, user_id);
        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(ClientError::Http)?;

        if !response.status().is_success() {
            return Err(api_error(response).await);
        }

        let user: User = response.json().await.map_err(ClientError::Http)?;
        Ok(user)
    }

    /// Batch-fetch users by their IDs. Mattermost limit is 1000 per request.
    pub async fn get_users_by_ids(
        &self,
        token: &str,
        user_ids: &[String],
    ) -> Result<Vec<User>, ClientError> {
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }
        let url = format!("{}/api/v4/users/ids", self.base_url);
        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .json(user_ids)
            .send()
            .await
            .map_err(ClientError::Http)?;

        if !response.status().is_success() {
            return Err(api_error(response).await);
        }

        let users: Vec<User> = response.json().await.map_err(ClientError::Http)?;
        Ok(users)
    }

    pub async fn get_teams(&self, token: &str) -> Result<Vec<mm_core::Team>, ClientError> {
        let url = format!("{}/api/v4/users/me/teams", self.base_url);
        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(ClientError::Http)?;

        if !response.status().is_success() {
            return Err(api_error(response).await);
        }

        let teams: Vec<mm_core::Team> = response.json().await.map_err(ClientError::Http)?;
        Ok(teams)
    }

    pub async fn get_my_channels(&self, token: &str) -> Result<Vec<Channel>, ClientError> {
        let url = format!("{}/api/v4/users/me/channels", self.base_url);
        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(ClientError::Http)?;

        if !response.status().is_success() {
            return Err(api_error(response).await);
        }

        let channels: Vec<Channel> = response.json().await.map_err(ClientError::Http)?;
        Ok(channels)
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
            return Err(api_error(response).await);
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
        before: Option<&str>,
    ) -> Result<Vec<Post>, ClientError> {
        let mut url = format!(
            "{}/api/v4/channels/{}/posts?per_page={}",
            self.base_url, channel_id, limit
        );
        if let Some(b) = before {
            url.push_str(&format!("&before={}", b));
        }

        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(ClientError::Http)?;

        if !response.status().is_success() {
            return Err(api_error(response).await);
        }

        #[derive(serde::Deserialize)]
        struct PostsResponse {
            order: Vec<String>,
            posts: std::collections::HashMap<String, Post>,
        }

        let data: PostsResponse = response.json().await.map_err(ClientError::Http)?;

        let mut posts: Vec<Post> = data
            .order
            .iter()
            .filter_map(|id| data.posts.get(id).cloned())
            .collect();

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
            return Err(api_error(response).await);
        }

        let post: Post = response.json().await.map_err(ClientError::Http)?;
        Ok(post)
    }
}

async fn api_error(response: reqwest::Response) -> ClientError {
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();

    #[derive(serde::Deserialize)]
    struct MmError {
        id: String,
        message: String,
    }

    let (id, message) = serde_json::from_str::<MmError>(&body)
        .map(|e| (e.id, e.message))
        .unwrap_or_else(|_| ("api_error".to_string(), body));

    ClientError::Api {
        id,
        message,
        status_code: status,
    }
}
