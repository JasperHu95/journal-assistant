use crate::models::Article;
use feed_rs::parser;

/// Parse raw RSS/Atom feed bytes into a list of Articles.
pub fn parse_feed_bytes(feed_id: i64, bytes: &[u8]) -> Result<Vec<Article>, String> {
    let feed = parser::parse(bytes).map_err(|e| format!("Feed parse error: {}", e))?;

    let mut articles = Vec::new();

    for entry in &feed.entries {
        let title = entry
            .title
            .as_ref()
            .map(|t| t.content.clone())
            .unwrap_or_default();

        let url = entry
            .links
            .first()
            .map(|l| l.href.clone());

        let published_at = entry
            .updated
            .or(entry.published);

        let content = entry
            .content
            .as_ref()
            .and_then(|c| c.body.clone());

        let summary = entry
            .summary
            .as_ref()
            .map(|s| s.content.clone());

        let author = entry
            .authors
            .first()
            .map(|a| a.name.clone());

        articles.push(Article {
            id: None,
            feed_id,
            title,
            url,
            author,
            content,
            summary,
            published_at,
            is_read: false,
            is_starred: false,
            created_at: None,
        });
    }

    Ok(articles)
}

/// Parse and extract feed metadata (title, description, link) from the channel.
pub fn parse_feed_metadata(bytes: &[u8]) -> Result<(String, Option<String>, Option<String>), String> {
    let feed = parser::parse(bytes).map_err(|e| format!("Feed parse error: {}", e))?;

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());

    let description = feed
        .description
        .map(|d| d.content);

    let link = feed
        .links
        .first()
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
}
