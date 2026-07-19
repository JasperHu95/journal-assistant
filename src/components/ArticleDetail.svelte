<script lang="ts">
  import { updateArticleSummary, getEncryptedSetting, formatDate, type Article } from "../lib/db";
  import { getLang } from "../lib/i18n";
  import { useI18n } from "../lib/useI18n.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let { article = $bindable() }: { article: Article | null } = $props();

  const localized = useI18n();

  let extracting = $state(false);
  let extractError = $state("");
  let translating = $state(false);
  let translation = $state("");
  let translateError = $state("");

  // 切换文章时重置提取/翻译状态（pre effect 保证在渲染前执行，避免闪烁）
  $effect.pre(() => {
    article?.id;
    extractError = "";
    translation = "";
    translateError = "";
  });

  // 文章无摘要时，从原文链接抓取摘要并写回数据库
  async function handleExtract() {
    if (!article?.url || article.id == null || extracting) return;
    extracting = true;
    extractError = "";
    try {
      const summary = await invoke<string>("extract_abstract", { url: article.url });
      await updateArticleSummary(article.id, summary);
      // article 与父组件列表中的对象指向同一 state proxy，直接赋值即可同步 UI
      article.summary = summary;
    } catch (e) {
      console.error("Failed to extract abstract:", e);
      extractError = localized("error.extract_failed");
    } finally {
      extracting = false;
    }
  }

  // 调用 DeepSeek API 将当前文章摘要翻译为界面语言
  async function handleTranslate() {
    if (!article?.summary || translating) return;
    const articleId = article.id;
    translating = true;
    translateError = "";
    translation = "";
    try {
      const apiKey = await getEncryptedSetting("deepseek_api_key");
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
      if (article?.id !== articleId) return;
      translation = result;
    } catch (e) {
      console.error("Failed to translate:", e);
      translateError = localized("error.translate_failed");
    } finally {
      translating = false;
    }
  }
</script>

{#if article}
  {@const meta = [article.author, formatDate(article.published_at), article.categories ? (typeof article.categories === "string" ? article.categories : article.categories.join(", ")) : null].filter(Boolean)}
  <h3 class="font-serif text-xl text-[#2C2416] mb-3 leading-relaxed">{article.title}</h3>
  <div class="flex items-center gap-3 text-xs text-[#6B5E4A] mb-6 pb-4 border-b border-[#D4C8B0]">
    {#each meta as item, i}
      {#if i > 0}<span class="text-[#D4C8B0]">|</span>{/if}
      <span>{item}</span>
    {/each}
  </div>
  {#if article.summary}
    <p class="text-sm text-[#2C2416] leading-relaxed whitespace-pre-wrap">{article.summary}</p>
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
  {:else if article.url}
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
  {#if article.url}
    <a href={article.url} target="_blank" rel="noopener noreferrer"
      class="inline-block mt-6 text-sm text-[#8B1A2B] hover:underline">
      {localized("articles.read_full")} &rarr;
    </a>
  {/if}
{:else}
  <div class="flex items-center justify-center h-full">
    <p class="text-sm text-[#6B5E4A] italic">{localized("articles.select")}</p>
  </div>
{/if}
