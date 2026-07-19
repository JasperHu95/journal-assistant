use crate::models::{Article, Feed};
use crate::rss::parser;
use encoding_rs::Encoding;
use reqwest::{Client, StatusCode};
use std::time::Duration;

/// 浏览器风格 UA：不少期刊站点（Elsevier、Springer 等）会拦截脚本特征的默认 UA
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36 \
    JournalAssistant/0.1";

/// 构建共享 HTTP client：超时、重定向策略、UA 统一在此配置
pub(crate) fn build_client() -> Result<Client, String> {
    Client::builder()
        .user_agent(USER_AGENT)
        // 部分站点会多次 301/302（http->https、www 跳转等），给足跳转余量
        .redirect(reqwest::redirect::Policy::limited(10))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// 请求 URL 并返回解码后的 UTF-8 文本。
/// 负责状态码检查与编码转换，是 fetcher/discovery 的统一入口。
pub(crate) async fn fetch_text(client: &Client, url: &str) -> Result<String, String> {
    let response = client
        .get(url)
        // 声明可接受的 feed 类型，兼顾只认 Accept 头的服务器
        .header(
            reqwest::header::ACCEPT,
            "application/rss+xml, application/atom+xml, application/xml;q=0.9, \
             text/xml;q=0.9, text/html;q=0.8, */*;q=0.5",
        )
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    // 先检查状态码，避免把 404/500 的错误页当成 feed 去解析
    let status = response.status();
    if status != StatusCode::OK {
        return Err(format!("HTTP request returned status {}", status));
    }

    // 从 Content-Type 头提取 charset，作为编码检测的依据之一
    let header_charset = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|ct| {
            ct.split(';')
                .map(str::trim)
                .find(|p| p.to_ascii_lowercase().starts_with("charset="))
                .map(|p| p["charset=".len()..].trim_matches('"').to_string())
        });

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    Ok(decode_to_utf8(&bytes, header_charset.as_deref()))
}

/// 将原始字节流按检测到的编码转成 UTF-8 文本。
/// 检测优先级：BOM > XML 声明中的 encoding > HTTP 头 charset > UTF-8。
/// 覆盖 GB2312 / GBK / GB18030 / Big5 等中文编码（encoding_rs 按 label 识别）。
fn decode_to_utf8(bytes: &[u8], header_charset: Option<&str>) -> String {
    let encoding = detect_encoding(bytes, header_charset);
    let (text, _, _) = encoding.decode(bytes);
    normalize_xml_declaration(text.into_owned())
}

/// 依次尝试 BOM、XML 声明、HTTP 头，确定字节流的实际编码。
fn detect_encoding(bytes: &[u8], header_charset: Option<&str>) -> &'static Encoding {
    // 1. BOM 是最可靠的信号
    if bytes.starts_with(b"\xEF\xBB\xBF") {
        return encoding_rs::UTF_8;
    }
    if bytes.starts_with(b"\xFF\xFE") {
        return encoding_rs::UTF_16LE;
    }
    if bytes.starts_with(b"\xFE\xFF") {
        return encoding_rs::UTF_16BE;
    }

    // 2. XML 声明 / HTML meta 中的编码标记（这些内容本身总是 ASCII，可直接扫描）
    if let Some(label) = sniff_declared_encoding(bytes) {
        if let Some(enc) = Encoding::for_label(label.as_bytes()) {
            return enc;
        }
    }

    // 3. HTTP Content-Type 头的 charset
    if let Some(label) = header_charset {
        if let Some(enc) = Encoding::for_label(label.as_bytes()) {
            return enc;
        }
    }

    // 4. 默认 UTF-8
    encoding_rs::UTF_8
}

/// 在文档头部扫描编码声明，支持两种形式：
/// - XML:  <?xml version="1.0" encoding="GB2312"?>
/// - HTML: <meta charset="gbk"> 或 <meta ... content="...; charset=gbk">
fn sniff_declared_encoding(bytes: &[u8]) -> Option<String> {
    // 编码声明只可能出现在文档开头，扫前 2KB 足够
    let head = &bytes[..bytes.len().min(2048)];
    let head_lower: String = head.iter().map(|b| b.to_ascii_lowercase() as char).collect();

    // XML 声明：限定在 <?xml ... ?> 范围内找 encoding=
    if let Some(start) = head_lower.find("<?xml") {
        let decl_end = head_lower[start..].find("?>").map(|i| start + i).unwrap_or(head_lower.len());
        let decl = &head_lower[start..decl_end];
        if let Some(v) = extract_quoted_value(decl, "encoding=") {
            return Some(v);
        }
    }

    // HTML meta charset
    if let Some(pos) = head_lower.find("charset=") {
        let rest = &head_lower[pos + "charset=".len()..];
        let value: String = rest
            .trim_start_matches(['"', '\''])
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !value.is_empty() {
            return Some(value);
        }
    }

    None
}

/// 从形如 key="value" 或 key='value' 的片段中提取 value。
fn extract_quoted_value(s: &str, key: &str) -> Option<String> {
    let pos = s.find(key)?;
    let rest = s[pos + key.len()..].trim_start();
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inner: String = rest[1..].chars().take_while(|&c| c != quote).collect();
    if inner.is_empty() {
        None
    } else {
        Some(inner)
    }
}

/// 转码为 UTF-8 后，XML 声明里的原始 encoding（如 GB2312）已与实际字节不符。
/// 若解析器严格按声明解码会造成二次乱码，这里统一改写为 UTF-8。
fn normalize_xml_declaration(text: String) -> String {
    let Some(start) = text.find("<?xml") else {
        return text;
    };
    let Some(end_off) = text[start..].find("?>") else {
        return text;
    };
    let decl_end = start + end_off + 2;

    let decl = &text[start..decl_end];
    let decl_lower = decl.to_ascii_lowercase();
    let Some(enc_pos) = decl_lower.find("encoding=") else {
        return text; // 声明中无 encoding，无需改写
    };
    let after_eq = &decl[enc_pos + "encoding=".len()..];
    let Some(quote) = after_eq.trim_start().chars().next() else {
        return text;
    };
    if quote != '"' && quote != '\'' {
        return text;
    }
    // 找到引号对的起止位置，整体替换为 "UTF-8"
    let val_start = enc_pos + "encoding=".len() + (after_eq.len() - after_eq.trim_start().len());
    let val_inner_start = val_start + 1;
    let Some(val_len) = decl[val_inner_start..].find(quote) else {
        return text;
    };

    format!(
        "{}\"UTF-8\"{}",
        &text[..start + val_start],
        &text[start + val_inner_start + val_len + 1..]
    )
}

/// Fetch the feed metadata (title, description, link) from a feed URL.
pub async fn fetch_feed_metadata(url: &str) -> Result<Feed, String> {
    let client = build_client()?;
    let text = fetch_text(&client, url).await?;

    let (title, description, link) = parser::parse_feed_metadata(text.as_bytes())?;

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
    let client = build_client()?;
    let text = fetch_text(&client, feed_url).await?;

    // feed_id placeholder: frontend sets the correct ID after lookup
    parser::parse_feed_bytes(0, text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_gb2312_feed() {
        // GB2312 编码的 feed：声明 encoding="GB2312"，标题含中文
        let title = "计算机学报";
        let xml = format!(
            r#"<?xml version="1.0" encoding="GB2312"?>
<rss version="2.0"><channel><title>{}</title>
<item><title>{}</title></item></channel></rss>"#,
            title, title
        );
        let gbk_bytes = encoding_rs::GBK.encode(&xml).0;

        let text = decode_to_utf8(&gbk_bytes, None);
        let articles = parser::parse_feed_bytes(1, text.as_bytes()).unwrap();
        assert_eq!(articles[0].title, title);
        // 声明已被改写为 UTF-8，避免二次解码乱码
        assert!(text.contains("encoding=\"UTF-8\""));
    }

    #[test]
    fn test_sniff_html_meta_charset() {
        let html = b"<html><head><meta charset=\"gbk\"></head><body></body></html>";
        assert_eq!(sniff_declared_encoding(html).as_deref(), Some("gbk"));
    }
}
