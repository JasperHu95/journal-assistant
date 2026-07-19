import Database from "@tauri-apps/plugin-sql";
import { invoke } from "@tauri-apps/api/core";

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
    CREATE TABLE IF NOT EXISTS settings (
      key TEXT PRIMARY KEY,
      value TEXT
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

/** Feed 及其文章数（期刊视图用） */
export interface FeedWithCount extends Feed {
  article_count: number;
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

// 期刊视图：查询所有 feed 并附带各自的文章数
export async function getFeedsWithArticleCount(): Promise<FeedWithCount[]> {
  const d = getDb();
  return await d.select<FeedWithCount[]>(
    `SELECT f.*, COUNT(a.id) AS article_count
     FROM feeds f
     LEFT JOIN articles a ON a.feed_id = f.id
     GROUP BY f.id
     ORDER BY f.title`
  );
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
  if (feedId != null) {
    return await d.select<Article[]>("SELECT * FROM articles WHERE feed_id = ? ORDER BY published_at DESC", [feedId]);
  }
  return await d.select<Article[]>("SELECT * FROM articles ORDER BY published_at DESC");
}

export async function markRead(id: number): Promise<void> {
  const d = getDb();
  await d.execute("UPDATE articles SET is_read = 1 WHERE id = ?", [id]);
}

// 抓取的摘要写回数据库
export async function updateArticleSummary(id: number, summary: string): Promise<void> {
  const d = getDb();
  await d.execute("UPDATE articles SET summary = ? WHERE id = ?", [summary, id]);
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

// 设置项（key-value）读写，如 DeepSeek API Key
export async function getSetting(key: string): Promise<string | null> {
  const d = getDb();
  const rows = await d.select<{ value: string }[]>(
    "SELECT value FROM settings WHERE key = ?",
    [key]
  );
  return rows.length > 0 ? rows[0].value : null;
}

export async function setSetting(key: string, value: string): Promise<void> {
  const d = getDb();
  await d.execute(
    "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    [key, value]
  );
}

// 敏感设置项（如 DeepSeek API Key）的加密读写，加密/解密在 Rust 侧完成
export async function setEncryptedSetting(key: string, value: string): Promise<void> {
  const encrypted = await invoke<string>("encrypt_value", { plaintext: value });
  await setSetting(key, encrypted);
}

export async function getEncryptedSetting(key: string): Promise<string | null> {
  const raw = await getSetting(key);
  if (!raw) return null;
  try {
    return await invoke<string>("decrypt_value", { ciphertext: raw });
  } catch {
    // 密文损坏（如旧版本明文残留）时视为未设置
    return null;
  }
}

/** 将 ISO 日期字符串格式化为本地可读形式（精确到分钟）；空值返回空串，无法解析时原样返回 */
export function formatDate(iso: string | null | undefined): string {
  if (!iso) return "";
  const date = new Date(iso);
  if (isNaN(date.getTime())) return iso;
  return date.toLocaleString(undefined, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}
