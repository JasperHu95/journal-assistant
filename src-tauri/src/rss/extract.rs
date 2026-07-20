use crate::rss::fetcher;
use crate::rss::parser;
use crate::rss::ssrf;
use scraper::{Html, Selector};

/// 视为有效摘要段落的最小字符数：过短的 <p> 多为导航、版权等噪声
const MIN_PARAGRAPH_LEN: usize = 50;

/// 从文章 URL 提取摘要文本。
/// 多数学术期刊站点有反爬机制，直接抓页面常失败，因此优先走 CrossRef API：
/// 1. URL 本身含 DOI（doi.org 链接或出版社 /doi/ 路径）-> 直接查 CrossRef
/// 2. 抓取页面，从 meta 标签（citation_doi 等）提取 DOI -> 查 CrossRef
/// 3. 以上均失败 -> 从 HTML 提取 meta description 或首个实质段落
pub async fn extract_abstract_from_url(url: &str) -> Result<String, String> {
    // SSRF 防护：仅允许 http/https，拒绝 localhost 与私有/内网地址
    ssrf::validate_url(url)?;
    let client = fetcher::get_client();

    // 策略1：URL 中可直接提取 DOI 时，免去抓页面，直接查 CrossRef
    if let Some(doi) = extract_doi(url) {
        if let Ok(abstract_text) = fetch_abstract_from_doi(client, &doi).await {
            return Ok(abstract_text);
        }
    }

    // 策略2：抓取页面（复用 fetcher 的解码逻辑：UA/超时/重定向/中文编码处理一致）
    let body = fetcher::fetch_text(client, url)
        .await
        .map_err(|e| e.to_string())?;

    // 策略2a：页面 meta 标签中声明了 DOI，查 CrossRef 拿结构化摘要
    if let Some(doi) = extract_doi_from_html(&body) {
        if let Ok(abstract_text) = fetch_abstract_from_doi(client, &doi).await {
            return Ok(abstract_text);
        }
    }

    // 策略2b：从 HTML 直接提取 meta description 或首段
    extract_from_html(&body)
}

/// 从 URL 中提取 DOI。
/// 覆盖 doi.org 链接与出版社页面路径中内嵌的 DOI（如 /doi/full/10.xxxx/...），
/// DOI 模式本身（10. + 4-9 位注册号 + / 后缀）在 URL 中足够特异，直接正则匹配即可。
fn extract_doi(url: &str) -> Option<String> {
    use regex::Regex;
    use std::sync::OnceLock;

    static DOI_RE: OnceLock<Regex> = OnceLock::new();
    let re = DOI_RE.get_or_init(|| Regex::new(r#"10\.\d{4,9}/[^\s?#"'<>]+"#).unwrap());

    let m = re.find(url)?;
    // 匹配可能带入 URL 的尾部标点（句点、逗号等），DOI 本身不会以这些字符结尾
    let doi = m.as_str().trim_end_matches(['.', ',', ';', ')']);
    if doi.is_empty() {
        None
    } else {
        Some(doi.to_string())
    }
}

/// 从 HTML meta 标签中提取 DOI：
/// <meta name="citation_doi" content="10.xxxx/..."> 或 <meta name="DOI" content="...">
fn extract_doi_from_html(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let meta_selector = Selector::parse("meta").ok()?;
    for element in document.select(&meta_selector) {
        let attrs = element.value();
        let name = attrs.attr("name").map(|n| n.to_ascii_lowercase());
        let content = attrs.attr("content").map(str::trim);
        if let (Some(name), Some(content)) = (name, content) {
            if (name == "citation_doi" || name == "doi") && content.starts_with("10.") {
                return Some(content.to_string());
            }
        }
    }
    None
}

/// 通过 CrossRef API 按 DOI 查询摘要。CrossRef 为开放 API，无需认证。
async fn fetch_abstract_from_doi(client: &reqwest::Client, doi: &str) -> Result<String, String> {
    let url = format!("https://api.crossref.org/works/{}", doi);
    let response = client
        .get(&url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|e| format!("CrossRef request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("CrossRef returned status {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse CrossRef response: {}", e))?;

    parse_crossref_abstract(&json).ok_or_else(|| "No abstract in CrossRef response".to_string())
}

/// 从 CrossRef 响应 JSON 中取 message.abstract 并清理为纯文本。
/// CrossRef 的 abstract 常带 JATS XML 标签（<jats:p>...</jats:p>），
/// strip_html 的标签匹配是通用的，可直接复用清理。
fn parse_crossref_abstract(json: &serde_json::Value) -> Option<String> {
    let raw = json["message"]["abstract"].as_str()?;
    let text = parser::strip_html(raw);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
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

    #[test]
    fn test_extract_doi_from_doi_org_url() {
        assert_eq!(
            extract_doi("https://doi.org/10.1038/s41586-023-06001-2").as_deref(),
            Some("10.1038/s41586-023-06001-2")
        );
    }

    #[test]
    fn test_extract_doi_from_publisher_url() {
        // 出版社页面路径内嵌 DOI（/doi/full/、/doi/abs/ 等形式）
        assert_eq!(
            extract_doi("https://www.tandfonline.com/doi/full/10.1080/12345678.2023.0001")
                .as_deref(),
            Some("10.1080/12345678.2023.0001")
        );
    }

    #[test]
    fn test_extract_doi_strips_query_and_trailing_punctuation() {
        assert_eq!(
            extract_doi("https://doi.org/10.1016/j.xxx.2023.01.001?via%3Dihub").as_deref(),
            Some("10.1016/j.xxx.2023.01.001")
        );
        assert_eq!(
            extract_doi("https://example.com/articles/10.1000/xyz123.").as_deref(),
            Some("10.1000/xyz123")
        );
    }

    #[test]
    fn test_extract_doi_none() {
        assert_eq!(extract_doi("https://example.com/blog/post-1"), None);
    }

    #[test]
    fn test_extract_doi_from_html() {
        let html = r#"<html><head>
            <meta name="citation_doi" content="10.1038/s41586-023-06001-2">
            </head><body></body></html>"#;
        assert_eq!(
            extract_doi_from_html(html).as_deref(),
            Some("10.1038/s41586-023-06001-2")
        );
    }

    #[test]
    fn test_extract_doi_from_html_doi_meta_name() {
        // 部分站点用 name="DOI"（大小写不敏感），且 content 含空白
        let html = r#"<html><head>
            <meta name="DOI" content=" 10.1016/j.xxx.2023.01.001 ">
            </head><body></body></html>"#;
        assert_eq!(
            extract_doi_from_html(html).as_deref(),
            Some("10.1016/j.xxx.2023.01.001")
        );
    }

    #[test]
    fn test_extract_doi_from_html_none() {
        // 无 DOI meta；content 不以 10. 开头的同名 meta 也不算
        let html = r#"<html><head>
            <meta name="description" content="some page">
            <meta name="citation_doi" content="not-a-doi">
            </head><body></body></html>"#;
        assert_eq!(extract_doi_from_html(html), None);
    }

    #[test]
    fn test_parse_crossref_abstract_with_jats() {
        let json = serde_json::json!({
            "message": {
                "abstract": "<jats:p>We study <jats:italic>RNA</jats:italic> folding.</jats:p>"
            }
        });
        assert_eq!(
            parse_crossref_abstract(&json).as_deref(),
            Some("We study RNA folding.")
        );
    }

    #[test]
    fn test_parse_crossref_abstract_missing_or_empty() {
        // 无 abstract 字段
        let json = serde_json::json!({"message": {"title": "x"}});
        assert_eq!(parse_crossref_abstract(&json), None);
        // abstract 清理后为空
        let json = serde_json::json!({"message": {"abstract": "<jats:p>  </jats:p>"}});
        assert_eq!(parse_crossref_abstract(&json), None);
    }
}
