<script lang="ts">
  import {
    updateArticleSummary,
    formatDate,
    getArticleTags,
    getTags,
    addTagToArticle,
    createTag,
    type Article,
    type Tag,
  } from "../lib/db";
  import { useI18n } from "../lib/useI18n.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let { article = $bindable() }: { article: Article | null } = $props();

  const localized = useI18n();

  let extracting = $state(false);
  let extractError = $state("");

  // 标签区域状态
  let articleTags = $state<Tag[]>([]);
  let allTags = $state<Tag[]>([]);
  let showTagInput = $state(false);
  let newTagName = $state("");

  // 可选列表 = 全部标签中尚未贴到当前文章的
  let availableTags = $derived(allTags.filter((t) => !articleTags.some((at) => at.id === t.id)));

  // 切换文章时重置提取状态（pre effect 保证在渲染前执行，避免闪烁）
  $effect.pre(() => {
    article?.id;
    extractError = "";
  });

  // 切换文章时加载该文章已贴的标签与全部已有标签
  $effect(() => {
    const id = article?.id;
    articleTags = [];
    showTagInput = false;
    newTagName = "";
    if (id == null) return;
    (async () => {
      try {
        const [attached, all] = await Promise.all([getArticleTags(id), getTags()]);
        // 异步返回时用户可能已切换文章，避免旧数据覆盖新文章
        if (article?.id === id) {
          articleTags = attached;
          allTags = all;
        }
      } catch (e) {
        console.error("Failed to load tags:", e);
      }
    })();
  });

  // 文章无摘要时提取：有 DOI 则后端直接查 OpenAlex/CrossRef，否则抓原文页面
  async function handleExtract() {
    if (!article?.url || article.id == null || extracting) return;
    extracting = true;
    extractError = "";
    try {
      const summary = await invoke<string>("extract_abstract", { url: article.url, doi: article.doi });
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

  async function attachTag(tag: Tag) {
    if (article?.id == null || tag.id == null) return;
    await addTagToArticle(article.id, tag.id);
    articleTags = await getArticleTags(article.id);
  }

  // 手动输入添加标签：大小写不敏感去重；仅大小写不同时询问是否合并到已有标签
  async function handleAddTag() {
    const name = newTagName.trim();
    if (!name || article?.id == null) return;
    const exact = allTags.find((t) => t.name === name);
    if (exact) {
      await attachTag(exact);
    } else {
      const ciMatch = allTags.find((t) => t.name.toLowerCase() === name.toLowerCase());
      if (ciMatch) {
        if (!confirm(localized("articles.tag_exists_confirm").replace("{name}", ciMatch.name))) return;
        await attachTag(ciMatch);
      } else {
        await createTag(name, "#8B1A2B");
        allTags = await getTags();
        const created = allTags.find((t) => t.name === name);
        if (created) await attachTag(created);
      }
    }
    newTagName = "";
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
    {@const abs = article.summary.trim()}
    {@const heading = abs.match(/^abstract\b[\s:：]*/i)}
    {#if heading}
      <h4 class="font-serif text-base font-bold text-[#2C2416] mb-2">ABSTRACT</h4>
      <p class="text-sm text-[#2C2416] leading-relaxed whitespace-pre-wrap">{abs.slice(heading[0].length).trim()}</p>
    {:else}
      <p class="text-sm text-[#2C2416] leading-relaxed whitespace-pre-wrap">{article.summary}</p>
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
  {#if article.id != null}
    <div class="mt-8 pt-4 border-t border-[#D4C8B0]">
      <div class="flex flex-wrap items-center gap-2">
        {#each articleTags as tag}
          <span class="inline-flex items-center gap-1.5 px-2 py-0.5 text-xs border border-[#D4C8B0] bg-[#FAF7F2] text-[#2C2416]">
            <span class="w-2 h-2 rounded-full shrink-0" style="background-color: {tag.color || '#8B1A2B'}"></span>
            {tag.name}
          </span>
        {/each}
        <button
          onclick={() => (showTagInput = !showTagInput)}
          class="px-2 py-0.5 text-xs text-[#8B1A2B] border border-dashed border-[#8B1A2B]/50 hover:bg-[#8B1A2B]/5 transition-colors"
        >
          {showTagInput ? localized("feeds.cancel") : localized("articles.add_tag")}
        </button>
      </div>
      {#if showTagInput}
        <div class="mt-3">
          <div class="flex gap-2">
            <input
              bind:value={newTagName}
              placeholder={localized("tags.placeholder")}
              onkeydown={(e) => e.key === "Enter" && handleAddTag()}
              class="flex-1 border border-[#D4C8B0] px-3 py-1.5 text-xs text-[#2C2416] bg-[#FAF7F2] focus:outline-none focus:border-[#8B1A2B]"
            />
            <button
              onclick={handleAddTag}
              disabled={!newTagName.trim()}
              class="px-3 py-1.5 bg-[#8B1A2B] text-white text-xs hover:bg-[#6d1522] disabled:opacity-50 transition-colors"
            >
              {localized("tags.create")}
            </button>
          </div>
          {#if availableTags.length > 0}
            <div class="flex flex-wrap gap-1.5 mt-2">
              {#each availableTags as tag}
                <button
                  onclick={() => attachTag(tag)}
                  class="inline-flex items-center gap-1.5 px-2 py-0.5 text-xs border border-[#D4C8B0] text-[#6B5E4A] hover:bg-[#D4C8B0]/30 transition-colors"
                >
                  <span class="w-2 h-2 rounded-full shrink-0" style="background-color: {tag.color || '#8B1A2B'}"></span>
                  {tag.name}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
{:else}
  <div class="flex items-center justify-center h-full">
    <p class="text-sm text-[#6B5E4A] italic">{localized("articles.select")}</p>
  </div>
{/if}
