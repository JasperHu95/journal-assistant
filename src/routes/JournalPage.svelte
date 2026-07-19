<script lang="ts">
  import { getFeedsWithArticleCount, getArticles, markRead, updateArticleSummary, getSetting, type FeedWithCount, type Article } from "../lib/db";
  import { t, onLangChange, getLang } from "../lib/i18n";
  import { invoke } from "@tauri-apps/api/core";

  let feeds = $state<FeedWithCount[]>([]);
  let selectedFeed = $state<FeedWithCount | null>(null);
  let articles = $state<Article[]>([]);
  let selected = $state<Article | null>(null);
  let extracting = $state(false);
  let extractError = $state("");
  let translating = $state(false);
  let translation = $state("");
  let translateError = $state("");

  // i18n 响应式
  let langVersion = $state(0);
  onLangChange(() => langVersion++);
  function localized(key: string): string {
    langVersion;
    return t(key);
  }

  $effect(() => {
    (async () => { feeds = await getFeedsWithArticleCount(); })();
  });

  // 点击期刊：加载该期刊下的文章，并清空已选文章
  async function handleSelectFeed(feed: FeedWithCount) {
    selectedFeed = feed;
    selected = null;
    if (feed.id == null) { articles = []; return; }
    try {
      articles = await getArticles(feed.id);
    } catch (e) {
      console.error("Failed to load articles:", e);
      articles = [];
    }
  }

  // 点击文章：展示详情并标记已读
  async function handleSelect(article: Article) {
    selected = article;
    extractError = "";
    translation = "";
    translateError = "";
    if (!article.is_read) {
      await markRead(article.id!);
      article.is_read = true;
    }
  }

  // 文章无摘要时，从原文链接抓取摘要并写回数据库
  async function handleExtract() {
    const article = selected;
    if (!article?.url || article.id == null || extracting) return;
    extracting = true;
    try {
      const summary = await invoke<string>("extract_abstract", { url: article.url });
      await updateArticleSummary(article.id, summary);
      // articles 列表与 selected 指向同一对象，直接赋值即可同步 UI
      article.summary = summary;
    } catch (e) {
      console.error("Failed to extract abstract:", e);
    } finally {
      extracting = false;
    }
  }

  // 调用 DeepSeek API 翻译当前文章摘要
  async function handleTranslate() {
    const article = selected;
    if (!article?.summary || translating) return;
    translating = true;
    translateError = "";
    translation = "";
    try {
      const apiKey = await getSetting("deepseek_api_key");
      if (!apiKey) {
        translateError = localized("articles.no_api_key");
        return;
      }
      const targetLang = getLang() === "zh" ? "中文" : "English";
      translation = await invoke<string>("translate_text", {
        text: article.summary,
        apiKey,
        targetLang,
      });
    } catch (e) {
      console.error("Failed to translate:", e);
      translateError = String(e);
    } finally {
      translating = false;
    }
  }
</script>

<div class="flex h-full">
  <!-- Journal list -->
  <div class="w-60 border-r border-[#D4C8B0] overflow-auto bg-white">
    <div class="p-4 border-b border-[#D4C8B0] sticky top-0 bg-white z-10">
      <h2 class="font-serif text-xl text-[#2C2416] tracking-wide">{localized("journals.title")}</h2>
    </div>
    {#each feeds as feed}
      <button
        onclick={() => handleSelectFeed(feed)}
        class="w-full text-left p-4 border-b border-[#D4C8B0]/40 hover:bg-[#FAF7F2] transition-colors
          {selectedFeed?.id === feed.id ? 'bg-[#F5F0E8] border-l-2 border-l-[#8B1A2B]' : ''}"
      >
        <p class="text-sm text-[#2C2416] font-medium leading-snug">{feed.title}</p>
        <p class="text-xs text-[#6B5E4A] mt-1">{feed.article_count} {localized("journals.articles_count")}</p>
      </button>
    {:else}
      <p class="text-sm text-[#6B5E4A] p-6 text-center italic">{localized("feeds.empty")}</p>
    {/each}
  </div>

  <!-- Article list of selected journal -->
  <div class="w-[380px] border-r border-[#D4C8B0] overflow-auto bg-white">
    {#if selectedFeed}
      <div class="p-4 border-b border-[#D4C8B0] sticky top-0 bg-white z-10">
        <h2 class="font-serif text-lg text-[#2C2416] tracking-wide truncate">{selectedFeed.title}</h2>
      </div>
      {#each articles as article}
        <button
          onclick={() => handleSelect(article)}
          class="w-full text-left p-4 border-b border-[#D4C8B0]/40 hover:bg-[#FAF7F2] transition-colors
            {selected?.id === article.id ? 'bg-[#F5F0E8] border-l-2 border-l-[#8B1A2B]' : ''}"
        >
          <div class="flex items-start gap-2">
            {#if !article.is_read}
              <span class="w-1.5 h-1.5 rounded-full bg-[#8B1A2B] mt-2 shrink-0"></span>
            {:else}
              <span class="w-1.5 h-1.5 mt-2 shrink-0"></span>
            {/if}
            <div class="min-w-0">
              <p class="text-sm text-[#2C2416] {!article.is_read ? 'font-semibold' : 'font-normal'} leading-snug">{article.title}</p>
              <p class="text-xs text-[#6B5E4A] mt-1">{article.author || localized("articles.unknown_author")}</p>
            </div>
          </div>
        </button>
      {:else}
        <p class="text-sm text-[#6B5E4A] p-6 text-center italic">{localized("articles.empty")}</p>
      {/each}
    {:else}
      <div class="flex items-center justify-center h-full">
        <p class="text-sm text-[#6B5E4A] italic">{localized("journals.select")}</p>
      </div>
    {/if}
  </div>

  <!-- Article detail -->
  <div class="flex-1 overflow-auto p-8">
    {#if selected}
      {@const meta = [selected.author, selected.published_at, selected.categories ? (typeof selected.categories === "string" ? selected.categories : selected.categories.join(", ")) : null].filter(Boolean)}
      <h3 class="font-serif text-xl text-[#2C2416] mb-3 leading-relaxed">{selected.title}</h3>
      <div class="flex items-center gap-3 text-xs text-[#6B5E4A] mb-6 pb-4 border-b border-[#D4C8B0]">
        {#each meta as item, i}
          {#if i > 0}<span class="text-[#D4C8B0]">|</span>{/if}
          <span>{item}</span>
        {/each}
      </div>
      {#if selected.summary}
        <p class="text-sm text-[#2C2416] leading-relaxed whitespace-pre-wrap">{selected.summary}</p>
        <button
          onclick={handleTranslate}
          disabled={translating}
          class="mt-4 px-4 py-2 bg-[#8B1A2B] text-white text-sm hover:bg-[#6d1522] disabled:opacity-50 transition-colors"
        >
          {translating ? localized("articles.translating") : localized("articles.translate")}
        </button>
        {#if translation}
          <div class="mt-4 p-4 bg-[#F5F0E8] border border-[#D4C8B0]">
            <p class="text-xs font-medium text-[#6B5E4A] mb-2">{localized("articles.translated")}</p>
            <p class="text-sm text-[#2C2416] leading-relaxed whitespace-pre-wrap">{translation}</p>
          </div>
        {/if}
        {#if translateError}
          <p class="mt-2 text-xs text-red-600">{translateError}</p>
        {/if}
      {:else if selected.url}
        <button
          onclick={handleExtract}
          disabled={extracting}
          class="px-4 py-2 bg-[#8B1A2B] text-white text-sm hover:bg-[#6d1522] disabled:opacity-50 transition-colors"
        >
          {extracting ? localized("articles.extracting") : localized("articles.extract")}
        </button>
      {/if}
      {#if selected.url}
        <a href={selected.url} target="_blank" rel="noopener noreferrer"
          class="inline-block mt-6 text-sm text-[#8B1A2B] hover:underline">
          {localized("articles.read_full")} &rarr;
        </a>
      {/if}
    {:else}
      <div class="flex items-center justify-center h-full">
        <p class="text-sm text-[#6B5E4A] italic">{localized("articles.select")}</p>
      </div>
    {/if}
  </div>
</div>
