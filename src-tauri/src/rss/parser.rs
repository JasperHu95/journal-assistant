use crate::models::Article;
use crate::rss::extract::extract_doi;
use feed_rs::model::Entry;
use feed_rs::parser;

/// 从原始 RSS/Atom 字节流解析出文章列表。
/// 调用方需保证字节流已是 UTF-8（编码转换在 fetcher 层完成）。
pub fn parse_feed_bytes(feed_id: i64, bytes: &[u8]) -> Result<Vec<Article>, String> {
    let feed = parser::parse(bytes).map_err(|e| format!("Feed parse error: {}", e))?;

    let articles = feed.entries.iter().map(parse_entry).collect::<Vec<_>>();

    // feed_id 由调用方指定，此处统一填入
    Ok(articles
        .into_iter()
        .map(|mut a| {
            a.feed_id = feed_id;
            a
        })
        .collect())
}

/// 提取单篇文章的各字段，集中处理不同 feed 格式的字段差异。
fn parse_entry(entry: &Entry) -> Article {
    // 标题：清理空白，截断英法双语拼接中的法语部分（Wiley CAR 源）
    let title = entry
        .title
        .as_ref()
        .map(|t| truncate_french_title(t.content.trim()))
        .unwrap_or_default();

    // 链接：Atom 中常有多个 link（alternate/self/enclosure），
    // 优先取 rel="alternate"（或缺省 rel，按 Atom 规范即 alternate）；
    // 其次取任意 http(s) 链接；最后回退到 guid/id（RSS 的 guid 常是 permalink）
    let url = extract_link(entry);

    // 作者：feed-rs 会把 RSS author、dc:creator、Atom author/name 统一放进 authors；
    // 多位作者用逗号连接，避免只取第一位造成信息丢失
    let author = {
        let names: Vec<&str> = entry
            .authors
            .iter()
            .map(|a| a.name.trim())
            .filter(|n| !n.is_empty())
            .collect();
        if names.is_empty() {
            // ScienceDirect 等源不使用标准 author 字段，作者写在 description 中
            entry
                .summary
                .as_ref()
                .and_then(|s| extract_authors_from_description(&s.content))
        } else {
            Some(names.join(", "))
        }
    };

    // 日期：优先 published（原始发表时间），updated 只是 feed 条目更新时间。
    // feed-rs 已兼容 RFC-822 / RFC-3339 / ISO-8601 等常见格式（含 dc:date）。
    let published_at = entry.published.or(entry.updated);

    // 正文：Atom content 或 RSS content:encoded
    // 清除原始 HTML 标签，避免前端把 <p> 等标签当纯文本显示
    let content = entry
        .content
        .as_ref()
        .and_then(|c| c.body.clone())
        .map(|b| strip_html(&b));

    // 摘要：Atom summary 或 RSS description（feed-rs 统一映射到 summary）。
    // 当 summary 为空或仅含元数据（如 "Journal name, EarlyView."）时，
    // 从 content（content:encoded）中提取摘要——Wiley 等出版社将摘要放在此处。
    let summary_raw = entry.summary.as_ref().map(|s| strip_html(&s.content));
    let summary = pick_summary(summary_raw, content.as_deref());

    // DOI：学术期刊条目常在 guid（Wiley）或链接（AEA）中携带 DOI，
    // 提取后供摘要抓取直接查 OpenAlex/CrossRef，绕过期刊页面的反爬
    let doi = extract_doi_from_entry(entry);

    // 分类/标签：RSS category 与 Atom category 的 term
    let categories = entry
        .categories
        .iter()
        .map(|c| c.term.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();

    Article {
        id: None,
        feed_id: 0,
        title,
        url,
        author,
        content,
        summary,
        doi,
        categories,
        published_at,
        is_read: false,
        is_starred: false,
        created_at: None,
    }
}

/// 从 description/summary 中提取作者：ScienceDirect 等源不用标准 author 字段，
/// 而是在描述正文中写 "Author(s): Name1, Name2"（也可能写作 "Author:" / "Authors:"）。
/// 捕获到 HTML 标签或换行为止，避免把摘要正文误当作作者。
fn extract_authors_from_description(description: &str) -> Option<String> {
    use regex::Regex;
    use std::sync::OnceLock;

    static AUTHOR_RE: OnceLock<Regex> = OnceLock::new();
    let re = AUTHOR_RE.get_or_init(|| {
        Regex::new(r"(?i)authors?(?:\(s\))?\s*:\s*([^<\n]+)").unwrap()
    });

    let captures = re.captures(description)?;
    let raw = captures.get(1)?.as_str();
    // 双重编码的内容里标签以 &lt; 形式存在，同样视为作者名单的终止
    let raw = raw.split("&lt;").next().unwrap_or(raw);
    let authors = decode_entities(raw).trim().to_string();
    if authors.is_empty() { None } else { Some(authors) }
}

/// 清除 HTML 字符串中的标签并解码常见实体，返回可纯文本显示的内容。
/// pub(crate)：extract 模块清理 CrossRef 返回的 JATS XML 摘要时复用。
pub(crate) fn strip_html(input: &str) -> String {
    use regex::Regex;
    use std::sync::OnceLock;

    // 先整体移除 script/style 标签及其内容，避免脚本样式文本混入正文
    static BLOCK_RE: OnceLock<Regex> = OnceLock::new();
    let block_re = BLOCK_RE.get_or_init(|| {
        Regex::new(r"(?is)<(script|style)\b[^>]*>.*?</(script|style)>").unwrap()
    });

    // 匹配所有 HTML 标签
    static TAG_RE: OnceLock<Regex> = OnceLock::new();
    let tag_re = TAG_RE.get_or_init(|| Regex::new(r"<[^>]*>").unwrap());

    // 压缩连续空白
    static WS_RE: OnceLock<Regex> = OnceLock::new();
    let ws_re = WS_RE.get_or_init(|| Regex::new(r"\s+").unwrap());

    let no_blocks = block_re.replace_all(input, " ");
    let no_tags = tag_re.replace_all(&no_blocks, " ");
    let decoded = decode_entities(&no_tags);
    ws_re.replace_all(decoded.trim(), " ").to_string()
}

/// 解码常见的 HTML 命名实体和数字实体。
fn decode_entities(input: &str) -> String {
    let mut out = input.to_string();
    // &amp; 必须最后解码，否则 &amp;lt; 会被二次解码成 <
    for (entity, ch) in [
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&apos;", "'"),
        ("&#39;", "'"),
        ("&nbsp;", " "),
    ] {
        out = out.replace(entity, ch);
    }
    out.replace("&amp;", "&")
}

/// 按优先级从 entry 中提取文章链接。
fn extract_link(entry: &Entry) -> Option<String> {
    // 1. rel="alternate" 或未声明 rel 的链接（Atom 规范：缺省即 alternate）
    let alternate = entry
        .links
        .iter()
        .find(|l| {
            l.rel.as_deref().map(|r| r == "alternate").unwrap_or(true)
                && !l.href.is_empty()
        })
        .map(|l| l.href.clone());
    if alternate.is_some() {
        return alternate;
    }

    // 2. 任意 http(s) 链接（排除 self/enclosure 等非正文链接后的兜底）
    let any = entry
        .links
        .iter()
        .map(|l| l.href.as_str())
        .find(|h| h.starts_with("http://") || h.starts_with("https://"))
        .map(|h| h.to_string());
    if any.is_some() {
        return any;
    }

    // 3. guid/id 回退：很多 RSS 把 permalink 放在 guid（isPermaLink="true"）
    let id = entry.id.trim();
    if id.starts_with("http://") || id.starts_with("https://") {
        return Some(id.to_string());
    }

    None
}

/// 从条目中提取 DOI：
/// 1. guid/id 本身是 DOI（Wiley 的 guid 形如 "10.1111/..."）或 doi.org 链接
/// 2. 任意链接中内嵌 DOI（AEA 的 ?id=10.1257/...、出版社 /doi/ 路径）
/// DOI 模式（10. + 4-9 位注册号 + / 后缀）足够特异，直接复用 extract 模块的 URL 匹配。
fn extract_doi_from_entry(entry: &Entry) -> Option<String> {
    if let Some(doi) = extract_doi(entry.id.trim()) {
        return Some(doi);
    }
    entry.links.iter().find_map(|l| extract_doi(&l.href))
}

/// Parse and extract feed metadata (title, description, link) from the channel.
pub fn parse_feed_metadata(bytes: &[u8]) -> Result<(String, Option<String>, Option<String>), String> {
    let feed = parser::parse(bytes).map_err(|e| format!("Feed parse error: {}", e))?;

    let title = feed
        .title
        .map(|t| t.content.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Untitled Feed".to_string());

    let description = feed.description.map(|d| d.content);

    // 与条目链接同策略：优先 alternate，其次第一个 http(s) 链接
    let link = feed
        .links
        .iter()
        .find(|l| l.rel.as_deref().map(|r| r == "alternate").unwrap_or(true))
        .or_else(|| feed.links.first())
        .map(|l| l.href.clone());

    Ok((title, description, link))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_feed() {
        let result = parse_feed_bytes(1, b"");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_rss2_cdata_guid_author_category() {
        // 覆盖：CDATA 标题、link 缺失时回退 guid、dc:creator 作者、category、description 摘要
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <channel>
    <title>Test Journal</title>
    <item>
      <title><![CDATA[A Study on <RSS> Parsing]]></title>
      <guid isPermaLink="true">https://example.com/articles/123</guid>
      <dc:creator>Zhang, San</dc:creator>
      <pubDate>Tue, 10 Jun 2025 08:00:00 GMT</pubDate>
      <description><![CDATA[<p>Abstract text</p>]]></description>
      <category>Computer Science</category>
      <category>Machine Learning</category>
    </item>
  </channel>
</rss>"#;
        let articles = parse_feed_bytes(7, xml.as_bytes()).unwrap();
        assert_eq!(articles.len(), 1);
        let a = &articles[0];
        assert_eq!(a.feed_id, 7);
        assert_eq!(a.title, "A Study on <RSS> Parsing");
        assert_eq!(a.url.as_deref(), Some("https://example.com/articles/123"));
        assert_eq!(a.author.as_deref(), Some("Zhang, San"));
        assert!(a.published_at.is_some());
        assert!(a.summary.as_deref().unwrap().contains("Abstract text"));
        assert_eq!(a.categories, vec!["Computer Science", "Machine Learning"]);
    }

    #[test]
    fn test_parse_atom_prefers_alternate_link() {
        // 覆盖：Atom 多 link 时应跳过 rel="self"，取 rel="alternate"
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Atom Feed</title>
  <entry>
    <title>Atom Entry</title>
    <link rel="self" href="https://example.com/feed.xml"/>
    <link rel="alternate" href="https://example.com/entry/1"/>
    <author><name>Li, Si</name></author>
    <author><name>Wang, Wu</name></author>
    <published>2025-06-10T08:00:00Z</published>
    <updated>2025-06-11T08:00:00Z</updated>
    <summary>Summary here</summary>
  </entry>
</feed>"#;
        let articles = parse_feed_bytes(1, xml.as_bytes()).unwrap();
        assert_eq!(articles.len(), 1);
        let a = &articles[0];
        assert_eq!(a.url.as_deref(), Some("https://example.com/entry/1"));
        // 多作者逗号连接
        assert_eq!(a.author.as_deref(), Some("Li, Si, Wang, Wu"));
        // published 优先于 updated
        assert_eq!(a.published_at.unwrap().to_rfc3339(), "2025-06-10T08:00:00+00:00");
    }

    #[test]
    fn test_parse_doi_from_guid() {
        // Wiley 风格：guid 本身即 DOI（description 只含期刊名，不含摘要）
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Wiley Journal</title>
    <item>
      <title>Some Paper</title>
      <guid isPermaLink="false">10.1111/1911-3846.70065</guid>
      <link>https://onlinelibrary.wiley.com/doi/10.1111/1911-3846.70065</link>
      <description>Journal of Whatever, EarlyView</description>
    </item>
  </channel>
</rss>"#;
        let articles = parse_feed_bytes(1, xml.as_bytes()).unwrap();
        assert_eq!(articles[0].doi.as_deref(), Some("10.1111/1911-3846.70065"));
    }

    #[test]
    fn test_parse_doi_from_link_query() {
        // AEA 风格：guid 是普通 id，DOI 内嵌在链接的 ?id= 查询参数中
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>AEA Journal</title>
    <item>
      <title>Some Paper</title>
      <guid isPermaLink="false">aea-12345</guid>
      <link>https://www.aeaweb.org/articles?id=10.1257/aer.20240930</link>
      <description>Volume 114, Issue 5, Page 1-30, May 2024</description>
    </item>
  </channel>
</rss>"#;
        let articles = parse_feed_bytes(1, xml.as_bytes()).unwrap();
        assert_eq!(articles[0].doi.as_deref(), Some("10.1257/aer.20240930"));
    }

    #[test]
    fn test_parse_doi_none() {
        // 普通博客条目：无 DOI 模式时为 None
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Blog</title>
    <item>
      <title>Post</title>
      <guid isPermaLink="true">https://example.com/blog/post-1</guid>
      <description>Just a post.</description>
    </item>
  </channel>
</rss>"#;
        let articles = parse_feed_bytes(1, xml.as_bytes()).unwrap();
        assert_eq!(articles[0].doi, None);
    }

    #[test]
    fn test_parse_authors_from_description() {
        // ScienceDirect 风格：无 dc:creator/author 字段，description 中写 "Author(s): ..."
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>ScienceDirect Journal</title>
    <item>
      <title>Some Paper</title>
      <link>https://www.sciencedirect.com/science/article/pii/S0000000001</link>
      <description><![CDATA[<p>Author(s): Jane Doe, John Smith</p><p>Publication date: June 2025</p>]]></description>
    </item>
  </channel>
</rss>"#;
        let articles = parse_feed_bytes(1, xml.as_bytes()).unwrap();
        // 作者名单在 </p> 处截断，不混入后续元数据
        assert_eq!(articles[0].author.as_deref(), Some("Jane Doe, John Smith"));
    }

    #[test]
    fn test_parse_author_singular_from_description() {
        // "Author:" 单数写法也应识别
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Journal</title>
    <item>
      <title>Paper</title>
      <link>https://example.com/paper/1</link>
      <description>Author: Zhang, San
Volume 10, Issue 2</description>
    </item>
  </channel>
</rss>"#;
        let articles = parse_feed_bytes(1, xml.as_bytes()).unwrap();
        // 作者名单在换行处截断
        assert_eq!(articles[0].author.as_deref(), Some("Zhang, San"));
    }

    #[test]
    fn test_parse_no_author_anywhere() {
        // 无标准 author 字段且 description 中也没有作者行时，author 为 None
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Blog</title>
    <item>
      <title>Post</title>
      <link>https://example.com/blog/post-1</link>
      <description>Just an abstract without any author info.</description>
    </item>
  </channel>
</rss>"#;
        let articles = parse_feed_bytes(1, xml.as_bytes()).unwrap();
        assert_eq!(articles[0].author, None);
    }
}

#[cfg(test)]
mod content_encoded_tests {
    use super::*;

    #[test]
    fn test_feed_rs_captures_content_encoded() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss xmlns:content="http://purl.org/rss/1.0/modules/content/" version="2.0">
<channel>
<title>Test</title>
<item>
<title>Article 1</title>
<description>Short desc</description>
<content:encoded>&lt;h2&gt;ABSTRACT&lt;/h2&gt;&lt;p&gt;This is the full abstract text from content:encoded which is quite long and should be preserved.&lt;/p&gt;</content:encoded>
<guid>10.1234/test.001</guid>
</item>
</channel>
</rss>"#;
        let feed = feed_rs::parser::parse(xml.as_bytes()).unwrap();
        let entry = &feed.entries[0];
        
        // Check what feed-rs captures
        let summary_text = entry.summary.as_ref().map(|s| s.content.clone());
        let content_body = entry.content.as_ref().and_then(|c| c.body.clone());
        
        println!("summary: {:?}", summary_text);
        println!("content body: {:?}", content_body);
        
        // Does feed-rs capture content:encoded?
        assert!(content_body.is_some(), "feed-rs should capture content:encoded");
        let body = content_body.unwrap();
        assert!(body.contains("full abstract text"), "content:encoded body should contain the abstract");
    }
}

/// 选择最佳摘要：summary（RSS description）优先，但当它为空或仅为元数据时回退到 content（content:encoded）。
/// Wiley 等出版社将摘要放在 <content:encoded> 而非 <description> 中。
fn pick_summary(summary: Option<String>, content: Option<&str>) -> Option<String> {
    // 如果 summary 包含有意义的摘要内容（长度 > 50 且不只是 "Journal name, EarlyView."），直接使用
    if let Some(ref s) = summary {
        let trimmed = s.trim();
        if trimmed.len() > 50 && !is_metadata_only(trimmed) {
            return summary.map(|s| truncate_french_abstract(&s));
        }
    }
    // 否则尝试从 content 中提取摘要
    if let Some(c) = content {
        let trimmed = c.trim();
        if trimmed.len() > 50 {
            return Some(truncate_french_abstract(trimmed));
        }
    }
    // 两者都没有有意义的内容，返回 summary（可能为空或短元数据）
    summary
}

/// 截断英法双语摘要中的法语部分，只保留英文摘要。
/// CAR（Contemporary Accounting Research）等加拿大期刊的 content:encoded 中
/// 英文摘要之后紧跟法语摘要（以法语标题或重复的 ABSTRACT 标题开头）。
/// 截断规则（取最靠前的截断点）：
/// 1. 出现多个大写 "ABSTRACT" 标题（英法各一个）时，截断到第二个之前；
/// 2. 出现明显的法语起始标记（英文摘要中不会出现的表达）时，截断到该标记之前。
/// 截断英法双语拼接标题中的法语部分，只保留英文标题。
/// Wiley 的 CAR（Contemporary Accounting Research）源把英文标题与法语标题
/// 无分隔符直接拼接（如 "Analyst IntegrityL'int\u{e9}grit\u{e9} des analysts"）。
/// 规则：找到第一个法语重音字符；英法拼接处呈"小写字母紧跟大写字母"的边界
/// （如 "WomenLa"），取重音之前最后一个这种边界截断；
/// 找不到时退化为截断到重音所在单词之前的空格。
fn truncate_french_title(title: &str) -> String {
    const FRENCH_CHARS: &str = "\u{e9}\u{e8}\u{ea}\u{eb}\u{e0}\u{e2}\u{f9}\u{fb}\u{fc}\u{f4}\u{ee}\u{ef}\u{e7}\u{153}\u{e6}\u{c9}\u{c8}\u{ca}\u{cb}\u{c0}\u{c2}\u{d9}\u{db}\u{dc}\u{d4}\u{ce}\u{cf}\u{c7}\u{152}\u{c6}";
    let Some(accent_pos) = title.find(|c: char| FRENCH_CHARS.contains(c)) else {
        return title.to_string();
    };

    // 重音之前最后一个"小写->大写"边界即英法拼接缝
    let mut seam = None;
    let mut prev_is_lower = false;
    for (i, c) in title.char_indices() {
        if i >= accent_pos {
            break;
        }
        if prev_is_lower && c.is_uppercase() {
            seam = Some(i);
        }
        prev_is_lower = c.is_lowercase();
    }
    if let Some(cut) = seam {
        return title[..cut].trim().to_string();
    }

    // 无拼接缝：截断到重音所在单词之前的空格
    let before = &title[..accent_pos];
    if let Some(last_space) = before.rfind(' ') {
        return title[..last_space].trim().to_string();
    }
    before.trim().to_string()
}

fn truncate_french_abstract(text: &str) -> String {
    let mut cut: Option<usize> = None;

    // 大写 ABSTRACT 是标题样式，英文摘要正文中几乎不会出现，可安全作为分隔点
    if let Some(first) = text.find("ABSTRACT") {
        let after_first = first + "ABSTRACT".len();
        if let Some(rel) = text[after_first..].find("ABSTRACT") {
            cut = Some(after_first + rel);
        }
    }

    // 明显的法语起始表达
    for marker in [
        "Sommaire",
        "Résumé",
        "RÉSUMÉ",
        "Cet article",
        "Cette étude",
        "Nous analysons",
        "Dans le cadre",
        "L'auteur",
        "Les résultats",
    ] {
        if let Some(pos) = text.find(marker) {
            if cut.map_or(true, |c| pos < c) {
                cut = Some(pos);
            }
        }
    }

    match cut {
        Some(pos) => text[..pos].trim().to_string(),
        None => text.to_string(),
    }
}

/// 判断文本是否仅为 RSS 元数据（如 "Journal name, EarlyView."、"Volume X, Issue Y"）而非真正摘要。
fn is_metadata_only(text: &str) -> bool {
    let lower = text.to_lowercase();
    // 常见的无摘要元数据模式
    let patterns = ["earlyview", "volume ", "issue ", "page ", "table of contents", "publication date"];
    let match_count = patterns.iter().filter(|p| lower.contains(*p)).count();
    // 如果匹配 2 个以上元数据关键词且文本较短，认为是纯元数据
    match_count >= 2 && text.len() < 200
}


#[cfg(test)]
mod french_abstract_tests {
    use super::*;

    #[test]
    fn test_truncate_at_second_abstract_marker() {
        // CAR 风格：英文 ABSTRACT 段落之后紧跟法语 ABSTRACT 段落
        let text = "ABSTRACT This study examines audit quality and finds robust evidence. ABSTRACT Cette \u{e9}tude examine la qualit\u{e9} de l'audit.";
        let result = truncate_french_abstract(text);
        assert_eq!(
            result,
            "ABSTRACT This study examines audit quality and finds robust evidence."
        );
    }

    #[test]
    fn test_truncate_at_french_marker() {
        // 法语部分无 ABSTRACT 标题，以明显的法语起始表达开头
        let text = "ABSTRACT We analyze disclosure choices in capital markets. Nous analysons les choix de divulgation sur les march\u{e9}s financiers.";
        let result = truncate_french_abstract(text);
        assert_eq!(
            result,
            "ABSTRACT We analyze disclosure choices in capital markets."
        );
    }

    #[test]
    fn test_truncate_at_sommaire() {
        let text = "ABSTRACT English abstract text that is long enough to matter here. Sommaire Cet article pr\u{e9}sente les r\u{e9}sultats.";
        let result = truncate_french_abstract(text);
        assert_eq!(
            result,
            "ABSTRACT English abstract text that is long enough to matter here."
        );
    }

    #[test]
    fn test_no_truncation_for_english_only() {
        // 纯英文摘要（含小写 abstract 一词）不应被截断
        let text = "ABSTRACT This paper studies how markets react to news and the abstract notion of risk.";
        let result = truncate_french_abstract(text);
        assert_eq!(result, text);
    }

    #[test]
    fn test_pick_summary_bilingual_content() {
        // summary 仅为元数据时回退到 content，并截断其中的法语部分
        let content = "ABSTRACT We examine earnings management in family firms and document several findings. ABSTRACT Nous examinons la gestion des r\u{e9}sultats.";
        let summary = pick_summary(
            Some("Contemporary Accounting Research, EarlyView.".to_string()),
            Some(content),
        );
        assert_eq!(
            summary.as_deref(),
            Some("ABSTRACT We examine earnings management in family firms and document several findings.")
        );
    }
}

#[cfg(test)]
mod french_title_tests {
    use super::*;

    #[test]
    fn test_glued_french_single_word() {
        let title = "Analyst IntegrityL'int\u{e9}grit\u{e9} des analysts";
        assert_eq!(truncate_french_title(title), "Analyst Integrity");
    }

    #[test]
    fn test_glued_french_long_title() {
        let title = "The Falling Roe and Relocation of Skilled WomenLa remise en cause de l'arr\u{ea}t Roe";
        assert_eq!(truncate_french_title(title), "The Falling Roe and Relocation of Skilled Women");
    }

    #[test]
    fn test_no_accent_unchanged() {
        let title = "Analyst Integrity in Question";
        assert_eq!(truncate_french_title(title), "Analyst Integrity in Question");
    }
}
