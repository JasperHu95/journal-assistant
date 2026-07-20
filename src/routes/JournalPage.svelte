<script lang="ts">
  import { getFeedsWithArticleCount, getArticles, markRead, formatDate, type FeedWithCount, type Article } from "../lib/db";
  import { useI18n } from "../lib/useI18n.svelte";
  import ArticleDetail from "../components/ArticleDetail.svelte";

  let feeds = $state<FeedWithCount[]>([]);
  let selectedFeed = $state<FeedWithCount | null>(null);
  let articles = $state<Article[]>([]);
  let selected = $state<Article | null>(null);

  const localized = useI18n();

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

  // 点击文章：切换为详情视图并标记已读
  async function handleSelect(article: Article) {
    selected = article;
    if (!article.is_read) {
      await markRead(article.id!);
      article.is_read = true;
      // selectedFeed 与 feeds 列表中的对象指向同一 state proxy，直接同步未读数
      if (selectedFeed && selectedFeed.unread_count > 0) selectedFeed.unread_count -= 1;
    }
  }

  // 返回文章列表
  function handleBack() {
    selected = null;
  }
</script>

<div class="flex h-full">
  <!-- Journal list -->
  <div class="w-64 border-r border-[#D4C8B0] overflow-auto bg-white">
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
        <p class="text-xs mt-1 {feed.unread_count > 0 ? 'text-[#8B1A2B] font-medium' : 'text-[#6B5E4A]'}">
          {feed.unread_count} {localized("journals.unread")}
        </p>
      </button>
    {:else}
      <p class="text-sm text-[#6B5E4A] p-6 text-center italic">{localized("feeds.empty")}</p>
    {/each}
  </div>

  <!-- Reading pane: article list, swapped for detail on selection -->
  <div class="flex-1 overflow-auto bg-white">
    {#if selectedFeed}
      {#if selected}
        <div class="p-8">
          <button
            onclick={handleBack}
            class="mb-6 text-sm text-[#8B1A2B] hover:underline"
          >
            &larr; {localized("common.back")}
          </button>
          <ArticleDetail bind:article={selected} />
        </div>
      {:else}
        <div class="p-4 border-b border-[#D4C8B0] sticky top-0 bg-white z-10">
          <h2 class="font-serif text-lg text-[#2C2416] tracking-wide truncate">{selectedFeed.title}</h2>
        </div>
        {#each articles as article}
          <button
            onclick={() => handleSelect(article)}
            class="w-full text-left p-4 border-b border-[#D4C8B0]/40 hover:bg-[#FAF7F2] transition-colors"
          >
            <div class="flex items-start gap-2">
              {#if !article.is_read}
                <span class="w-1.5 h-1.5 rounded-full bg-[#8B1A2B] mt-2 shrink-0"></span>
              {:else}
                <span class="w-1.5 h-1.5 mt-2 shrink-0"></span>
              {/if}
              <div class="min-w-0">
                <p class="text-sm text-[#2C2416] {!article.is_read ? 'font-semibold' : 'font-normal'} leading-snug">{article.title}</p>
                <p class="text-xs text-[#6B5E4A] mt-1">
                  {article.author || localized("articles.unknown_author")}
                  {#if formatDate(article.published_at)}
                    <span class="text-[#D4C8B0]"> | </span>{formatDate(article.published_at)}
                  {/if}
                </p>
              </div>
            </div>
          </button>
        {:else}
          <p class="text-sm text-[#6B5E4A] p-6 text-center italic">{localized("articles.empty")}</p>
        {/each}
      {/if}
    {:else}
      <div class="flex items-center justify-center h-full">
        <p class="text-sm text-[#6B5E4A] italic">{localized("journals.select")}</p>
      </div>
    {/if}
  </div>
</div>
