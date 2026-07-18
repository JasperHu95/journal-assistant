use crate::models::DiscoveredFeed;
use scraper::{Html, Selector};

/// Discover RSS/Atom feed URLs from an HTML page at the given URL.
pub async fn discover_feeds_from_url(url: &str) -> Result<Vec<DiscoveredFeed>, String> {
    let client = reqwest::Client::builder()
        .user_agent("JournalAssistant/0.1")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let document = Html::parse_document(&body);

    let mut feeds = Vec::new();

    // Look for <link rel="alternate" type="application/rss+xml" ...>
    let link_selector =
        Selector::parse("link[rel='alternate']").map_err(|e| format!("Selector error: {:?}", e))?;

    for element in document.select(&link_selector) {
        let link_type = element.value().attr("type").unwrap_or("");

        if link_type.contains("rss") || link_type.contains("atom") || link_type.contains("xml") {
            if let Some(href) = element.value().attr("href") {
                let title = element.value().attr("title").map(|t| t.to_string());
                feeds.push(DiscoveredFeed {
                    url: resolve_url(url, href),
                    title,
                });
            }
        }
    }

    Ok(feeds)
}

/// Resolve a potentially relative URL against a base URL.
fn resolve_url(base: &str, href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else if href.starts_with("//") {
        format!("https:{}", href)
    } else if href.starts_with('/') {
        // Extract origin from base
        if let Some(pos) = base.find("://") {
            let after_proto = &base[pos + 3..];
            if let Some(slash_pos) = after_proto.find('/') {
                format!("{}{}", &base[..pos + 3 + slash_pos], href)
            } else {
                format!("{}{}", base.trim_end_matches('/'), href)
            }
        } else {
            href.to_string()
        }
    } else {
        // Relative path
        let base_dir = base.rsplit_once('/').map(|(d, _)| d).unwrap_or(base);
        format!("{}/{}", base_dir, href)
    }
}
