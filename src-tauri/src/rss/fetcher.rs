use crate::models::{Article, Feed};
use crate::rss::parser;

/// Fetch the feed metadata (title, description, link) from a feed URL.
pub async fn fetch_feed_metadata(url: &str) -> Result<Feed, String> {
    let client = reqwest::Client::builder()
        .user_agent("JournalAssistant/0.1")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    let (title, description, link) = parser::parse_feed_metadata(&bytes)?;

    Ok(Feed {
        id: None,
        url: url.to_string(),
        title,
        description,
        link,
        last_fetched_at: None,
        created_at: None,
    })
}

/// Fetch the latest articles from a feed URL.
/// feed_id is set to 0 as a placeholder; the frontend assigns the real ID before storing.
pub async fn fetch_articles(feed_url: &str) -> Result<Vec<Article>, String> {
    let client = reqwest::Client::builder()
        .user_agent("JournalAssistant/0.1")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let response = client
        .get(feed_url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    // feed_id placeholder: frontend sets the correct ID after lookup
    parser::parse_feed_bytes(0, &bytes)
}
