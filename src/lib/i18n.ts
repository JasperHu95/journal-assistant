type Translations = Record<string, string>;

const zh: Translations = {
  "app.title": "Journal Assistant",
  "nav.dashboard": "仪表盘",
  "nav.feeds": "RSS 订阅源",
  "nav.articles": "文章",
  "nav.journals": "期刊",
  "nav.tags": "标签",
  "nav.settings": "设置",
  "dashboard.total_feeds": "订阅源总数",
  "dashboard.total_articles": "文章总数",
  "dashboard.unread": "未读",
  "dashboard.tags": "标签数",
  "feeds.title": "RSS 订阅源",
  "feeds.add": "添加订阅源",
  "feeds.cancel": "取消",
  "feeds.mode_url": "RSS 地址",
  "feeds.mode_discover": "自动发现",
  "feeds.url_placeholder": "https://example.com/rss.xml",
  "feeds.discover_placeholder": "https://example.com",
  "feeds.adding": "添加中...",
  "feeds.remove": "移除",
  "feeds.empty": "暂无订阅源",
  "feeds.refresh": "刷新",
  "feeds.refreshing": "刷新中...",
  "feeds.no_found": "未发现 RSS 源",
  "feeds.refresh_ok": "文章已更新",
  "articles.title": "文章",
  "articles.select": "选择一篇文章阅读",
  "articles.read_full": "阅读全文",
  "articles.unknown_author": "未知作者",
  "articles.empty": "暂无文章，请先添加订阅源",
  "articles.extract": "提取摘要",
  "articles.extracting": "提取中...",
  "articles.translate": "翻译",
  "articles.translating": "翻译中...",
  "articles.no_api_key": "请先在设置中配置 API Key",
  "articles.translated": "翻译结果",
  "journals.title": "期刊",
  "journals.select": "选择一个期刊",
  "journals.articles_count": "篇文章",
  "tags.title": "标签",
  "tags.name": "标签名称",
  "tags.color": "颜色",
  "tags.create": "创建",
  "tags.placeholder": "例如：机器学习",
  "tags.remove": "移除",
  "tags.empty": "暂无标签",
  "settings.title": "设置",
  "settings.api_key": "DeepSeek API Key",
  "settings.api_key_placeholder": "输入你的 DeepSeek API Key",
  "settings.save": "保存",
  "settings.saved": "已保存",
  "lang.switch": "Switch to English",
  "common.loading": "加载中...",
  "common.db_error": "数据库错误",
  "version": "v0.1.0",
};

const en: Translations = {
  "app.title": "Journal Assistant",
  "nav.dashboard": "Dashboard",
  "nav.feeds": "RSS Feeds",
  "nav.articles": "Articles",
  "nav.journals": "Journals",
  "nav.tags": "Tags",
  "nav.settings": "Settings",
  "dashboard.total_feeds": "Total Feeds",
  "dashboard.total_articles": "Total Articles",
  "dashboard.unread": "Unread",
  "dashboard.tags": "Tags",
  "feeds.title": "RSS Feeds",
  "feeds.add": "Add Feed",
  "feeds.cancel": "Cancel",
  "feeds.mode_url": "RSS URL",
  "feeds.mode_discover": "Auto Discover",
  "feeds.url_placeholder": "https://example.com/rss.xml",
  "feeds.discover_placeholder": "https://example.com",
  "feeds.adding": "Adding...",
  "feeds.remove": "Remove",
  "feeds.empty": "No feeds subscribed yet.",
  "feeds.refresh": "Refresh",
  "feeds.refreshing": "Refreshing...",
  "feeds.no_found": "No feeds found at this URL",
  "feeds.refresh_ok": "Articles updated",
  "articles.title": "Articles",
  "articles.select": "Select an article to read",
  "articles.read_full": "Read Full Article",
  "articles.unknown_author": "Unknown author",
  "articles.empty": "No articles yet. Add some feeds first.",
  "articles.extract": "Extract Abstract",
  "articles.extracting": "Extracting...",
  "articles.translate": "Translate",
  "articles.translating": "Translating...",
  "articles.no_api_key": "Please configure API Key in Settings first",
  "articles.translated": "Translation",
  "journals.title": "Journals",
  "journals.select": "Select a journal",
  "journals.articles_count": "articles",
  "tags.title": "Tags",
  "tags.name": "Tag Name",
  "tags.color": "Color",
  "tags.create": "Create",
  "tags.placeholder": "e.g. Machine Learning",
  "tags.remove": "Remove",
  "tags.empty": "No tags created yet.",
  "settings.title": "Settings",
  "settings.api_key": "DeepSeek API Key",
  "settings.api_key_placeholder": "Enter your DeepSeek API Key",
  "settings.save": "Save",
  "settings.saved": "Saved",
  "lang.switch": "切换到中文",
  "common.loading": "Loading...",
  "common.db_error": "Database Error",
  "version": "v0.1.0",
};

const dicts: Record<string, Translations> = { zh, en };

// Use a plain variable + callback pattern instead of $state (which only works in .svelte files)
let currentLang = "zh";

// Restore saved language preference
if (typeof window !== "undefined") {
  const saved = localStorage.getItem("ja-lang");
  if (saved && dicts[saved]) currentLang = saved;
}

// Listeners for reactive updates
type Listener = () => void;
const listeners: Set<Listener> = new Set();

function notify() {
  listeners.forEach((fn) => fn());
}

export function t(key: string): string {
  return dicts[currentLang]?.[key] ?? dicts["zh"]?.[key] ?? key;
}

export function getLang(): string {
  return currentLang;
}

export function setLang(lang: string) {
  if (dicts[lang]) {
    currentLang = lang;
    if (typeof window !== "undefined") {
      localStorage.setItem("ja-lang", lang);
    }
    notify();
  }
}

export function toggleLang() {
  setLang(currentLang === "zh" ? "en" : "zh");
}

// Subscribe to language changes (for Svelte reactivity)
export function onLangChange(fn: Listener): () => void {
  listeners.add(fn);
  return () => listeners.delete(fn);
}
