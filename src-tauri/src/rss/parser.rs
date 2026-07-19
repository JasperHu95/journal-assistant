use crate::models::Article;
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
    // 标题：feed-rs 已处理 CDATA 和 XML 实体，这里只做空白清理
    let title = entry
        .title
        .as_ref()
        .map(|t| t.content.trim().to_string())
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
            None
        } else {
            Some(names.join(", "))
        }
    };

    // 日期：优先 published（原始发表时间），updated 只是 feed 条目更新时间。
    // feed-rs 已兼容 RFC-822 / RFC-3339 / ISO-8601 等常见格式（含 dc:date）。
    let published_at = entry.published.or(entry.updated);

    // 正文：Atom content 或 RSS content:encoded
    let content = entry.content.as_ref().and_then(|c| c.body.clone());

    // 摘要：Atom summary 或 RSS description（feed-rs 统一映射到 summary）
    let summary = entry.summary.as_ref().map(|s| s.content.clone());

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
        categories,
        published_at,
        is_read: false,
        is_starred: false,
        created_at: None,
    }
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
    <author><name>Li, Si</name><name>Wang, Wu</name></author>
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
}
