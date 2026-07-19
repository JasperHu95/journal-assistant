<script lang="ts">
  import { getArticles, markRead, type Article } from "../lib/db";
  import { useI18n } from "../lib/useI18n.svelte";
  import ArticleDetail from "../components/ArticleDetail.svelte";

  let articles = $state<Article[]>([]);
  let selected = $state<Article | null>(null);

  const localized = useI18n();

  $effect(() => {
    (async () => { articles = await getArticles(); })();
  });

  // 点击文章：展示详情并标记已读
  async function handleSelect(article: Article) {
    selected = article;
    if (!article.is_read) {
      await markRead(article.id!);
      article.is_read = true;
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
    <ArticleDetail bind:article={selected} />
  </div>
</div>
