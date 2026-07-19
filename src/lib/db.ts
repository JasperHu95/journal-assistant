import Database from "@tauri-apps/plugin-sql";

let db: InstanceType<typeof Database> | null = null;

export async function initDb() {
  db = await Database.load("sqlite:journal_assistant.db");
  await db.execute(`
    CREATE TABLE IF NOT EXISTS feeds (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      url TEXT NOT NULL UNIQUE,
      title TEXT NOT NULL,
      description TEXT,
      link TEXT,
      last_fetched_at TEXT,
      created_at TEXT DEFAULT (datetime('now'))
    );
    CREATE TABLE IF NOT EXISTS articles (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      feed_id INTEGER NOT NULL,
      title TEXT NOT NULL,
      url TEXT,
      author TEXT,
      content TEXT,
      summary TEXT,
      categories TEXT,
      published_at TEXT,
      is_read INTEGER NOT NULL DEFAULT 0,
      is_starred INTEGER NOT NULL DEFAULT 0,
      created_at TEXT DEFAULT (datetime('now')),
      FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE
    );
    CREATE TABLE IF NOT EXISTS tags (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL UNIQUE,
      color TEXT
    );
    CREATE TABLE IF NOT EXISTS article_tags (
      article_id INTEGER NOT NULL,
      tag_id INTEGER NOT NULL,
      PRIMARY KEY (article_id, tag_id),
      FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE,
      FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
    );
  `);
  // 旧版本 articles 表没有 categories 列，幂等迁移：列已存在时 ALTER 报错，忽略即可
  try {
    await db.execute("ALTER TABLE articles ADD COLUMN categories TEXT");
  } catch {
    /* 列已存在 */
  }
}

function getDb() {
  if (!db) throw new Error("Database not initialized");
  return db;
}

export interface Feed {
  id: number | null;
  url: string;
  title: string;
  description: string | null;
  link: string | null;
  last_fetched_at: string | null;
  created_at: string | null;
}

export interface Article {
  id: number | null;
  feed_id: number;
  title: string;
  url: string | null;
  author: string | null;
  content: string | null;
  summary: string | null;
  /** DB 中为逗号分隔字符串；后端 invoke 返回的是 string[] */
  categories: string | null;
  published_at: string | null;
  is_read: boolean;
  is_starred: boolean;
  created_at: string | null;
}

export interface Tag {
  id: number | null;
  name: string;
  color: string | null;
}

// Feed CRUD
export async function addFeed(feed: Feed): Promise<void> {
  const d = getDb();
  await d.execute(
    "INSERT OR IGNORE INTO feeds (id, url, title, description, link) VALUES (?, ?, ?, ?, ?)",
    [feed.id, feed.url, feed.title, feed.description, feed.link]
  );
}

export async function getFeeds(): Promise<Feed[]> {
  const d = getDb();
  return await d.select<Feed[]>("SELECT * FROM feeds ORDER BY title");
}

export async function deleteFeed(id: number): Promise<void> {
  const d = getDb();
  await d.execute("DELETE FROM feeds WHERE id = ?", [id]);
}

// Article CRUD
export async function insertArticles(articles: Article[]): Promise<void> {
  const d = getDb();
  for (const a of articles) {
    await d.execute(
      "INSERT OR IGNORE INTO articles (feed_id, title, url, author, content, summary, categories, published_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
      [a.feed_id, a.title, a.url, a.author, a.content, a.summary, typeof a.categories === "string" ? a.categories : Array.isArray(a.categories) ? a.categories.join(", ") : null, a.published_at]
    );
  }
}

export async function getArticles(feedId?: number): Promise<Article[]> {
  const d = getDb();
  if (feedId) {
    return await d.select<Article[]>("SELECT * FROM articles WHERE feed_id = ? ORDER BY published_at DESC", [feedId]);
  }
  return await d.select<Article[]>("SELECT * FROM articles ORDER BY published_at DESC");
}

export async function markRead(id: number): Promise<void> {
  const d = getDb();
  await d.execute("UPDATE articles SET is_read = 1 WHERE id = ?", [id]);
}

export async function markAllRead(): Promise<void> {
  const d = getDb();
  await d.execute("UPDATE articles SET is_read = 1");
}

// Tag CRUD
export async function createTag(name: string, color: string): Promise<void> {
  const d = getDb();
  await d.execute("INSERT INTO tags (name, color) VALUES (?, ?)", [name, color]);
}

export async function getTags(): Promise<Tag[]> {
  const d = getDb();
  return await d.select<Tag[]>("SELECT * FROM tags ORDER BY name");
}

export async function deleteTag(id: number): Promise<void> {
  const d = getDb();
  await d.execute("DELETE FROM tags WHERE id = ?", [id]);
}

export async function addTagToArticle(articleId: number, tagId: number): Promise<void> {
  const d = getDb();
  await d.execute("INSERT OR IGNORE INTO article_tags (article_id, tag_id) VALUES (?, ?)", [articleId, tagId]);
}

export async function removeTagFromArticle(articleId: number, tagId: number): Promise<void> {
  const d = getDb();
  await d.execute("DELETE FROM article_tags WHERE article_id = ? AND tag_id = ?", [articleId, tagId]);
}

export async function getArticleTags(articleId: number): Promise<Tag[]> {
  const d = getDb();
  return await d.select<Tag[]>(
    "SELECT t.* FROM tags t JOIN article_tags at ON t.id = at.tag_id WHERE at.article_id = ?",
    [articleId]
  );
}
