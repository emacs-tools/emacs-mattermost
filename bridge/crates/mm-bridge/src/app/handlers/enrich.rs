use mm_client::MattermostClient;
use mm_core::Post;
use std::collections::{HashMap, HashSet};

pub async fn enrich_posts_with_display_names(
    client: &MattermostClient,
    token: &str,
    posts: &mut [Post],
) {
    let unique_ids: Vec<String> = posts
        .iter()
        .map(|p| p.user_id.0.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if unique_ids.is_empty() {
        return;
    }

    match client.get_users_by_ids(token, &unique_ids).await {
        Ok(users) => {
            let name_map: HashMap<String, String> = users
                .into_iter()
                .map(|u| (u.id.0.clone(), u.display_name()))
                .collect();

            for post in posts.iter_mut() {
                if let Some(name) = name_map.get(&post.user_id.0) {
                    post.user_display_name = name.clone();
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to resolve user display names, falling back to IDs");
        }
    }
}

pub async fn enrich_posts_with_files(client: &MattermostClient, token: &str, posts: &mut [Post]) {
    for post in posts.iter_mut() {
        if post.file_ids.is_empty() {
            continue;
        }
        match client.get_post_files(token, &post.id.0).await {
            Ok(files) => {
                post.files = files;
            }
            Err(e) => {
                tracing::warn!(
                    post_id = %post.id.0,
                    error = %e,
                    "Failed to fetch file info for post"
                );
            }
        }
    }
}
