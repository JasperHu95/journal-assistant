<script lang="ts">
  import { getArticles, markRead, updateArticleSummary, getSetting, type Article } from "../lib/db";
  import { t, getLang, onLangChange } from "../lib/i18n";
  import { invoke } from "@tauri-apps/api/core";

  let articles = $state<Article[]>([]);
  let selected = $state<Article | null>(null);
  let extracting = $state(false);
  let extractError = $state("");
  let translating = $state(false);
  let translation = $state("");
  let translateError = $state("");

  let langVersion = $state(0);
  onLangChange(() => langVersion++);
  function localized(key: string): string {
    langVersion;
    return t(key);
  }

  $effect(() => {
    (async () => { articles = await getArticles(); })();
  });

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
    extractError = "";
    try {
      const summary = await invoke<string>("extract_abstract", { url: article.url });
      await updateArticleSummary(article.id, summary);
      // articles 列表与 selected 指向同一对象，直接赋值即可同步 UI
      article.summary = summary;
    } catch (e) {
      console.error("Failed to extract abstract:", e);
      extractError = String(e);
    } finally {
      extracting = false;
    }
  }

  // 调用 DeepSeek API 将当前文章摘要翻译为界面语言
  async function handleTranslate() {
    const article = selected;
    if (!article?.summary || translating) return;
    const articleId = article.id;
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
      const result = await invoke<string>("translate_text", {
        text: article.summary,
        apiKey,
        targetLang,
      });
      // 翻译期间若用户切换了文章，丢弃过期结果，避免错位
      if (selected?.id !== articleId) return;
      translation = result;
    } catch (e) {
      console.error("Failed to translate:", e);
      translateError = String(e);
    } finally {
      translating = false;
    }
  }
</script>

<div class="flex h-full">
  <!-- Article list -->
  <div class="w-[380px] border-r border-[#D4C8B0] overflow-auto bg-white">
    <div class="p-4 border-b border-[#D4C8B0] sticky top-0 bg-white z-10">
      <h2 class="font-serif text-xl text-[#2C2416] tracking-wide">{localized("articles.title")}</h2>
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
      {#if extractError}
        <p class="mt-2 text-xs text-red-600">{extractError}</p>
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
