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

/// 调用 DeepSeek API 将文本翻译为目标语言（如 "中文"、"English"），返回译文。
#[tauri::command]
pub async fn translate_text(
    text: String,
    api_key: String,
    target_lang: String,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let body = serde_json::json!({
        "model": "deepseek-chat",
        "messages": [
            {
                "role": "system",
                "content": format!(
                    "你是学术翻译专家。将以下学术文本翻译为{}。保持学术用语准确，保留专业术语。",
                    target_lang
                )
            },
            { "role": "user", "content": text }
        ],
        "temperature": 0.3
    });

    let response = client
        .post("https://api.deepseek.com/v1/chat/completions")
        .bearer_auth(&api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Translation request failed: {}", e))?;

    // 非 2xx 时把状态码和响应体带上，便于定位（Key 错误、额度不足等）
    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(format!("DeepSeek API error {}: {}", status, detail));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse DeepSeek response: {}", e))?;

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Unexpected response format from DeepSeek API".to_string())
}
