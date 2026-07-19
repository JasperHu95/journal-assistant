use crate::rss::fetcher;
use crate::rss::ssrf;
use scraper::{Html, Selector};

/// 视为有效摘要段落的最小字符数：过短的 <p> 多为导航、版权等噪声
const MIN_PARAGRAPH_LEN: usize = 50;

/// 从文章 URL 抓取页面并提取摘要文本。
/// 复用 fetcher 的 client 与解码逻辑：UA/超时/重定向/中文编码处理一致。
pub async fn extract_abstract_from_url(url: &str) -> Result<String, String> {
    // SSRF 防护：仅允许 http/https，拒绝 localhost 与私有/内网地址
    ssrf::validate_url(url)?;
    let client = fetcher::build_client()?;
    let body = fetcher::fetch_text(&client, url)
        .await
        .map_err(|e| e.to_string())?;
    extract_from_html(&body)
}

/// 从 HTML 中按优先级提取摘要：
/// 1. <meta name="description">
/// 2. <meta property="og:description">
/// 3. 正文中第一个有实质内容的 <p>
fn extract_from_html(html: &str) -> Result<String, String> {
    let document = Html::parse_document(html);

    // meta 属性值的大小写和书写位置（name 还是 property）各站不一，
    // 遍历所有 meta 手动匹配，比 CSS 选择器精确匹配更稳
    let meta_selector =
        Selector::parse("meta").map_err(|e| format!("Selector error: {:?}", e))?;

    let mut description: Option<String> = None;
    let mut og_description: Option<String> = None;

    for element in document.select(&meta_selector) {
        let attrs = element.value();
        let Some(key) = attrs
            .attr("name")
            .or_else(|| attrs.attr("property"))
            .map(|k| k.to_ascii_lowercase())
        else {
            continue;
        };
        let content = attrs.attr("content").map(str::trim).unwrap_or("");
        if content.is_empty() {
            continue;
        }
        match key.as_str() {
            "description" if description.is_none() => description = Some(content.to_string()),
            "og:description" if og_description.is_none() => {
                og_description = Some(content.to_string())
            }
            _ => {}
        }
    }

    // name="description" 优先于 og:description
    if let Some(text) = description.or(og_description) {
        return Ok(collapse_whitespace(&text));
    }

    // 回退：第一个达到最小长度的 <p>
    let p_selector = Selector::parse("p").map_err(|e| format!("Selector error: {:?}", e))?;
    for element in document.select(&p_selector) {
        let text = collapse_whitespace(&element.text().collect::<String>());
        if text.chars().count() >= MIN_PARAGRAPH_LEN {
            return Ok(text);
        }
    }

    Err("No abstract found on this page".to_string())
}

/// 压缩 HTML 文本中的连续空白（换行、缩进等）为单个空格
fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_description() {
        let html = r#"<html><head>
            <meta name="description" content="  这是摘要。  ">
            <meta property="og:description" content="OG 摘要">
            </head><body></body></html>"#;
        assert_eq!(extract_from_html(html).unwrap(), "这是摘要。");
    }

    #[test]
    fn test_og_description_fallback() {
        let html = r#"<html><head>
            <meta property="og:description" content="OG abstract">
            </head><body></body></html>"#;
        assert_eq!(extract_from_html(html).unwrap(), "OG abstract");
    }

    #[test]
    fn test_paragraph_fallback() {
        let html = "<html><body>\
            <p>短</p>\
            <p>这一段足够长，是一段真正有意义的正文内容，应当被选作文章的摘要文本。\
            为了通过最小长度阈值的校验，这里再补充一些内容使其总长度超过五十个字符。</p>\
            </body></html>";
        let result = extract_from_html(html).unwrap();
        assert!(result.starts_with("这一段足够长"));
    }

    #[test]
    fn test_no_abstract() {
        let html = "<html><body><p>short</p></body></html>";
        assert!(extract_from_html(html).is_err());
    }
}
