use crate::models::{Article, Feed};
use crate::rss::parser;
use crate::rss::ssrf;
use encoding_rs::Encoding;
use reqwest::{Client, StatusCode};
use std::sync::OnceLock;
use std::time::Duration;

/// 浏览器风格 UA：不少期刊站点（Elsevier、Springer 等）会拦截脚本特征的默认 UA
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36 \
    JournalAssistant/0.1";

/// 响应体大小上限：10MB。恶意 feed 可返回超大响应耗尽内存（OOM），
/// 读取前查 Content-Length、读取后校验实际字节数，双重拦截。
const MAX_RESPONSE_BYTES: u64 = 10 * 1024 * 1024;

/// 抓取失败的原因分类，供调用方区分对待（如 SSRF 拦截 vs 普通网络错误）。
#[derive(Debug)]
pub(crate) enum FetchError {
    /// SSRF 校验拦截（内网地址、非法 scheme 等）
    Blocked(String),
    /// 服务器返回非 200 状态码
    Http(StatusCode),
    /// 网络层错误（连接失败、超时、读取响应体失败等）
    Network(String),
    /// 编码转换或 feed 解析错误
    Decode(String),
    /// 响应体超过 MAX_RESPONSE_BYTES
    TooLarge,
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blocked(msg) => write!(f, "Blocked: {}", msg),
            Self::Http(status) => write!(f, "HTTP error: {}", status),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Decode(msg) => write!(f, "Decode error: {}", msg),
            Self::TooLarge => write!(f, "Response too large"),
        }
    }
}

impl std::error::Error for FetchError {}

/// 进程级共享的 HTTP client：懒初始化一次，后续所有请求复用同一连接池，
/// 避免每次 fetch 都新建 Client（TCP/TLS 连接无法复用，开销大）。
static CLIENT: OnceLock<Client> = OnceLock::new();

/// 获取共享 HTTP client。初始化失败（如 TLS 后端不可用）属于不可恢复错误，直接 panic。
pub(crate) fn get_client() -> &'static Client {
    CLIENT.get_or_init(|| new_client().expect("Failed to build HTTP client"))
}

/// 兼容既有调用方的入口：返回共享 client 的 clone。
/// reqwest::Client 内部是 Arc，clone 廉价且共享连接池与配置；新代码应直接用 get_client()。
pub(crate) fn build_client() -> Result<Client, String> {
    Ok(get_client().clone())
}

/// 构建 HTTP client：超时、重定向策略、UA 统一在此配置。
/// 只应经由 get_client() 的 OnceLock 调用一次。
fn new_client() -> Result<Client, String> {
    // 自定义重定向策略：每次跳转的目标 URL 都重新过 validate_url 校验，
    // 防止攻击者用 302 跳转到 169.254.x.x / 127.0.0.1 等内网地址绕过 SSRF 防护。
    // 跳转上限保持 10：部分站点会多次 301/302（http->https、www 跳转等），给足余量
    let redirect_policy = reqwest::redirect::Policy::custom(|attempt| {
        if ssrf::validate_url(attempt.url().as_str()).is_err() {
            return attempt.stop();
        }
        if attempt.previous().len() >= 10 {
            return attempt.error("too many redirects");
        }
        attempt.follow()
    });
    Client::builder()
        .user_agent(USER_AGENT)
        .redirect(redirect_policy)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// 请求 URL 并返回解码后的 UTF-8 文本。
/// 负责状态码检查、大小限制与编码转换，是 fetcher/discovery 的统一入口。
pub(crate) async fn fetch_text(client: &Client, url: &str) -> Result<String, FetchError> {
    // 已知限制（DNS rebinding）：validate_url 只校验 URL 字面 host，攻击者可通过
    // 控制 DNS 把公网域名解析到内网 IP 来绕过校验。完整防护需要自定义 resolver
    // 在连接建立时校验解析结果，留待后续版本实现，本次暂不做完整防护。
    ssrf::validate_url(url).map_err(FetchError::Blocked)?;
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
        .map_err(|e| FetchError::Network(e.to_string()))?;

    // 先检查状态码，避免把 404/500 的错误页当成 feed 去解析
    let status = response.status();
    if status != StatusCode::OK {
        return Err(FetchError::Http(status));
    }

    // 读取前先看 Content-Length：超限直接拒绝，避免把超大响应拉进内存
    if let Some(len) = response.content_length() {
        if len > MAX_RESPONSE_BYTES {
            return Err(FetchError::TooLarge);
        }
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
        .map_err(|e| FetchError::Network(e.to_string()))?;

    // Content-Length 可能缺失或被伪造，读完后再校验一次实际大小
    if bytes.len() as u64 > MAX_RESPONSE_BYTES {
        return Err(FetchError::TooLarge);
    }

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
    // 定位 encoding 后的 '='（"encoding=" 中必然含 '='，unwrap 不会 panic）
    let eq_pos = decl_lower[enc_pos..].find('=').unwrap() + enc_pos;
    // 用 find 直接定位 '=' 之后第一个引号的位置（引号前可能有空白）
    let after_eq = &decl[eq_pos + 1..];
    let Some(quote_off) = after_eq.find(|c: char| c == '"' || c == '\'') else {
        return text;
    };
    let quote_pos = eq_pos + 1 + quote_off;
    let quote = decl.as_bytes()[quote_pos];
    // 从引号后找配对的闭合引号，确定 value 范围
    let Some(val_len) = decl[quote_pos + 1..].find(quote as char) else {
        return text;
    };

    // 将引号对（含 value）整体替换为 "UTF-8"
    format!(
        "{}\"UTF-8\"{}",
        &text[..start + quote_pos],
        &text[start + quote_pos + 1 + val_len + 1..]
    )
}

/// Fetch the feed metadata (title, description, link) from a feed URL.
pub async fn fetch_feed_metadata(url: &str) -> Result<Feed, FetchError> {
    let text = fetch_text(get_client(), url).await?;

    let (title, description, link) =
        parser::parse_feed_metadata(text.as_bytes()).map_err(FetchError::Decode)?;

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
pub async fn fetch_articles(feed_url: &str) -> Result<Vec<Article>, FetchError> {
    let text = fetch_text(get_client(), feed_url).await?;

    // feed_id placeholder: frontend sets the correct ID after lookup
    parser::parse_feed_bytes(0, text.as_bytes()).map_err(FetchError::Decode)
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

    #[test]
    fn test_normalize_xml_declaration() {
        let cases: &[(&str, &str)] = &[
            // 无 encoding 声明：原样返回
            (
                r#"<?xml version="1.0"?><rss version="2.0"/>"#,
                r#"<?xml version="1.0"?><rss version="2.0"/>"#,
            ),
            // 双引号 encoding="GB2312"：替换为 UTF-8
            (
                r#"<?xml version="1.0" encoding="GB2312"?><rss version="2.0"/>"#,
                r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0"/>"#,
            ),
            // 单引号 encoding='GBK'：替换为 UTF-8（统一输出双引号）
            (
                "<?xml version='1.0' encoding='GBK'?><rss version=\"2.0\"/>",
                r#"<?xml version='1.0' encoding="UTF-8"?><rss version="2.0"/>"#,
            ),
            // 声明不全（无 ?>）：原样返回
            (
                r#"<?xml version="1.0" encoding="GB2312"#,
                r#"<?xml version="1.0" encoding="GB2312"#,
            ),
            // 空字符串：返回空
            ("", ""),
        ];
        for (input, expected) in cases {
            assert_eq!(
                normalize_xml_declaration(input.to_string()),
                *expected,
                "input: {}",
                input
            );
        }
    }
}
