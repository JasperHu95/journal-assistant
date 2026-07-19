use crate::models::{Article, DiscoveredFeed, Feed};
use crate::rss::{discovery, extract, fetcher};

/// Fetch and parse an RSS feed from a URL. Returns feed metadata.
/// The frontend is responsible for storing the result in SQLite.
#[tauri::command]
pub async fn add_feed(url: String) -> Result<Feed, String> {
    fetcher::fetch_feed_metadata(&url)
        .await
        .map_err(|e| format!("Failed to fetch feed: {}", e))
}

/// Discover RSS/Atom feeds from a website URL.
#[tauri::command]
pub async fn discover_feeds(url: String) -> Result<Vec<DiscoveredFeed>, String> {
    discovery::discover_feeds_from_url(&url)
        .await
        .map_err(|e| format!("Failed to discover feeds: {}", e))
}

/// Fetch new articles from a feed URL. Returns a list of articles with
/// feed_id set to 0 as placeholder. The frontend assigns the real feed_id
/// before inserting into SQLite.
#[tauri::command]
pub async fn refresh_feed(feed_url: String) -> Result<Vec<Article>, String> {
    fetcher::fetch_articles(&feed_url)
        .await
        .map_err(|e| format!("Failed to refresh feed: {}", e))
}

/// 从论文 URL 抓取摘要/描述。
/// 优先取 <meta name="description"> 或 <meta property="og:description">，
/// 两者都没有时退回正文中第一个有实质内容的 <p>。
#[tauri::command]
pub async fn extract_abstract(url: String) -> Result<String, String> {
    extract::extract_abstract_from_url(&url)
        .await
        .map_err(|e| format!("Failed to extract abstract: {}", e))
}
