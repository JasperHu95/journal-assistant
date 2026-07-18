<script lang="ts">
  import { getArticles, markRead, type Article } from "../lib/db";
  import { t } from "../lib/i18n";

  let articles = $state<Article[]>([]);
  let selected = $state<Article | null>(null);

  $effect(() => {
    (async () => { articles = await getArticles(); })();
  });

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
      <h2 class="font-serif text-xl text-[#2C2416] tracking-wide">{t("articles.title")}</h2>
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
            <p class="text-xs text-[#6B5E4A] mt-1">{article.author || t("articles.unknown_author")}</p>
          </div>
        </div>
      </button>
    {:else}
      <p class="text-sm text-[#6B5E4A] p-6 text-center italic">{t("articles.empty")}</p>
    {/each}
  </div>

  <!-- Article detail -->
  <div class="flex-1 overflow-auto p-8">
    {#if selected}
      <h3 class="font-serif text-xl text-[#2C2416] mb-3 leading-relaxed">{selected.title}</h3>
      <div class="flex items-center gap-3 text-xs text-[#6B5E4A] mb-6 pb-4 border-b border-[#D4C8B0]">
        {#if selected.author}
          <span>{selected.author}</span>
          <span class="text-[#D4C8B0]">|</span>
        {/if}
        {#if selected.published_at}
          <span>{selected.published_at}</span>
        {/if}
      </div>
      {#if selected.summary}
        <p class="text-sm text-[#2C2416] leading-relaxed whitespace-pre-wrap">{selected.summary}</p>
      {/if}
      {#if selected.url}
        <a href={selected.url} target="_blank" rel="noopener noreferrer"
          class="inline-block mt-6 text-sm text-[#8B1A2B] hover:underline">
          {t("articles.read_full")} &rarr;
        </a>
      {/if}
    {:else}
      <div class="flex items-center justify-center h-full">
        <p class="text-sm text-[#6B5E4A] italic">{t("articles.select")}</p>
      </div>
    {/if}
  </div>
</div>
