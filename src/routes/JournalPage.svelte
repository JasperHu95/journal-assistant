<script lang="ts">
  import { getArticles, markRead, formatRelativeTime, type FeedWithCount, type Article } from "../lib/db";
  import { useI18n } from "../lib/useI18n.svelte";
  import ArticleDetail from "../components/ArticleDetail.svelte";
  import SplitPane from "../components/SplitPane.svelte";

  let {
    feeds,
    selectedFeedId,
  }: {
    feeds: FeedWithCount[];
    selectedFeedId: number | null;
  } = $props();

  let articles = $state<Article[]>([]);
  let selected = $state<Article | null>(null);
  let searchQuery = $state("");

  const localized = useI18n();

  // 当前选中期刊由 Sidebar 经 App 层传入；feeds 与 Sidebar 共享同一 state proxy
  let selectedFeed = $derived(feeds.find((f) => f.id === selectedFeedId) ?? null);

  // 按标题和作者实时过滤；搜索框为空时显示全部
  let filteredArticles = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return articles;
    return articles.filter(
      (a) =>
        a.title.toLowerCase().includes(q) ||
        (a.author ?? "").toLowerCase().includes(q)
    );
  });

  // 选中期刊变化时：加载该期刊下的文章，并清空已选文章与搜索框
  $effect(() => {
    const feedId = selectedFeedId;
    selected = null;
    articles = [];
    searchQuery = "";
    if (feedId == null) return;
    (async () => {
      try {
        const list = await getArticles(feedId);
        // 异步返回时用户可能已切换期刊，避免旧数据覆盖新列表
        if (selectedFeedId === feedId) articles = list;
      } catch (e) {
        console.error("Failed to load articles:", e);
      }
    })();
  });

  // 点击文章：切换为详情视图并标记已读
  async function handleSelect(article: Article) {
    selected = article;
    if (!article.is_read) {
      // 先同步标记再写库：快速双击时第二次调用会被 is_read 检查拦下，
      // 避免 unread_count 递减两次
      article.is_read = true;
      await markRead(article.id!);
      // selectedFeed 与 feeds 列表中的对象指向同一 state proxy，直接同步未读数
      if (selectedFeed && selectedFeed.unread_count > 0) selectedFeed.unread_count -= 1;
    }
  }

  // 返回文章列表
  function handleBack() {
    selected = null;
  }
</script>

<div class="h-full bg-white">
  {#if selectedFeed}
    <SplitPane storageKey="journal-pane" leftWidth={320}>
      {#snippet left()}
        <div class="p-4 border-b border-[#D4C8B0] sticky top-0 bg-white z-10">
          <h2 class="font-serif text-lg text-[#2C2416] tracking-wide truncate">{selectedFeed.title}</h2>
          <input
            type="text"
            bind:value={searchQuery}
            placeholder={localized("articles.search")}
            class="mt-3 w-full border border-[#D4C8B0] px-3 py-1.5 text-sm text-[#2C2416] bg-[#FAF7F2] focus:outline-none focus:border-[#8B1A2B]"
          />
        </div>
        {#each filteredArticles as article}
          <button
            onclick={() => handleSelect(article)}
            class="w-full text-left p-4 border-b border-[#D4C8B0]/40 hover:bg-[#FAF7F2] transition-colors
              {selected?.id === article.id ? 'bg-[#8B1A2B]/5' : ''}"
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
                  {#if formatRelativeTime(article.published_at)}
                    <span class="text-[#D4C8B0]"> | </span>{formatRelativeTime(article.published_at)}
                  {/if}
                </p>
              </div>
            </div>
          </button>
        {:else}
          <p class="text-sm text-[#6B5E4A] p-6 text-center italic">{localized("articles.empty")}</p>
        {/each}
      {/snippet}
      {#snippet right()}
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
          <div class="flex items-center justify-center h-full">
            <p class="text-sm text-[#6B5E4A] italic">{localized("articles.select")}</p>
          </div>
        {/if}
      {/snippet}
    </SplitPane>
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-sm text-[#6B5E4A] italic">{localized("journals.select")}</p>
    </div>
  {/if}
</div>
