use crate::models::DiscoveredFeed;
use crate::rss::fetcher;
use crate::rss::ssrf;
use scraper::{Html, Selector};

/// 已知的 feed MIME 类型（统一小写后比较）
const FEED_MIME_TYPES: &[&str] = &[
    "application/rss+xml",   // RSS 2.0
    "application/atom+xml",  // Atom
    "application/rdf+xml",   // RSS 1.0
    "application/xml",       // 泛 XML，部分站点用它声明 feed
    "text/xml",              // 泛 XML 的另一种写法
    "application/feed+json", // JSON Feed（feed-rs 同样支持解析）
];

/// Discover RSS/Atom feed URLs from an HTML page at the given URL.
pub async fn discover_feeds_from_url(url: &str) -> Result<Vec<DiscoveredFeed>, String> {
    // SSRF 防护：仅允许 http/https，拒绝 localhost 与私有/内网地址
    ssrf::validate_url(url)?;
    // 复用 fetcher 的 client 与解码逻辑：UA/超时/重定向/中文编码处理一致
    let client = fetcher::build_client()?;
    let body = fetcher::fetch_text(&client, url)
        .await
        .map_err(|e| e.to_string())?;

    let document = Html::parse_document(&body);

    // 候选 1：<link rel="alternate" type="..."> 标准的 feed 声明方式。
    // rel 可能含多个 token（如 "alternate stylesheet"），且属性值大小写不保证，
    // 因此选中全部 link 后手动判断，比 CSS 选择器精确匹配更稳。
    let link_selector =
        Selector::parse("link").map_err(|e| format!("Selector error: {:?}", e))?;

    let mut feeds: Vec<DiscoveredFeed> = Vec::new();

    for element in document.select(&link_selector) {
        let attrs = element.value();

        let is_alternate = attrs
            .attr("rel")
            .map(|rel| {
                rel.split_whitespace()
                    .any(|t| t.eq_ignore_ascii_case("alternate"))
            })
            .unwrap_or(false);
        if !is_alternate {
            continue;
        }

        let link_type = attrs.attr("type").unwrap_or("").to_ascii_lowercase();
        if !FEED_MIME_TYPES.contains(&link_type.as_str()) {
            continue;
        }

        if let Some(href) = attrs.attr("href") {
            let resolved = resolve_url(url, href);
            // 同一页面常以多种 type 声明同一 feed，按 URL 去重
            if !feeds.iter().any(|f| f.url == resolved) {
                feeds.push(DiscoveredFeed {
                    url: resolved,
                    title: attrs.attr("title").map(|t| t.to_string()),
                });
            }
        }
    }

    Ok(feeds)
}

/// Resolve a potentially relative URL against a base URL.
/// 直接复用 reqwest 内部的 url 库，正确处理 ./、../、//host、绝对路径等情况。
fn resolve_url(base: &str, href: &str) -> String {
    reqwest::Url::parse(base)
        .and_then(|b| b.join(href))
        .map(|u| u.to_string())
        // base 不是合法绝对 URL 时，原样返回，由后续请求阶段报错
        .unwrap_or_else(|_| href.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_url_variants() {
        let base = "https://example.com/blog/page";
        assert_eq!(
            resolve_url(base, "https://other.com/feed.xml"),
            "https://other.com/feed.xml"
        );
        assert_eq!(resolve_url(base, "/feed.xml"), "https://example.com/feed.xml");
        assert_eq!(resolve_url(base, "feed.xml"), "https://example.com/blog/feed.xml");
        assert_eq!(resolve_url(base, "//cdn.com/feed"), "https://cdn.com/feed");
    }
}
