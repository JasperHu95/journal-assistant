<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { CATEGORIES, JOURNAL_CATALOG, type CatalogJournal } from "../lib/journal-catalog";
  import { getFeeds, addFeed, deleteFeed, insertArticles, updateArticleSummary, type Feed } from "../lib/db";
  import { useI18n } from "../lib/useI18n.svelte";
  import SplitPane from "../components/SplitPane.svelte";

  // 空字符串表示"全部分类"
  let selectedCategory = $state("");
  let feeds = $state<Feed[]>([]);
  let pendingUrl = $state<string | null>(null);
  let refreshingUrl = $state<string | null>(null);
  let extractProgress = $state<{ current: number; total: number } | null>(null);
  let { refreshFeeds }: { refreshFeeds: () => Promise<void> } = $props();
  let error = $state("");
  let successMsg = $state("");

  const localized = useI18n();

  async function loadFeeds() { feeds = await getFeeds(); }
  $effect(() => { loadFeeds(); });

  const subscribedUrls = $derived(new Set(feeds.map((f) => f.url)));
  const visibleJournals = $derived(
    selectedCategory === ""
      ? JOURNAL_CATALOG
      : JOURNAL_CATALOG.filter((j) => j.category === selectedCategory)
  );

  async function handleSubscribe(journal: CatalogJournal) {
    pendingUrl = journal.rssUrl;
    error = "";
    try {
      const result = await invoke<Feed>("add_feed", { url: journal.rssUrl });
      await addFeed(result);
      await loadFeeds();
      await refreshFeeds();
      // 订阅成功后立即刷新该期刊文章，无需用户再手动点刷新
      await handleRefresh(journal);
    } catch (e) {
      console.error(String(e));
      error = localized("error.fetch_failed");
    } finally {
      pendingUrl = null;
    }
  }

  async function handleUnsubscribe(journal: CatalogJournal) {
    const feed = feeds.find((f) => f.url === journal.rssUrl);
    if (!feed || feed.id == null) return;
    pendingUrl = journal.rssUrl;
    error = "";
    try {
      await deleteFeed(feed.id);
      await loadFeeds();
      await refreshFeeds();
    } catch (e) {
      console.error(String(e));
      error = localized("error.fetch_failed");
    } finally {
      pendingUrl = null;
    }
  }

  // 刷新已订阅期刊并自动提取摘要
  async function handleRefresh(journal: CatalogJournal) {
    const feed = feeds.find((f) => f.url === journal.rssUrl);
    if (!feed || feed.id == null) return;
    refreshingUrl = journal.rssUrl;
    error = "";
    successMsg = "";
    extractProgress = null;
    try {
      const articles = await invoke<{ feed_id: number; title: string; url: string | null; author: string | null; content: string | null; summary: string | null; doi: string | null; published_at: string | null }[]>(
        "refresh_feed",
        { feedUrl: journal.rssUrl }
      );
      const withFeedId = articles.map(a => ({ ...a, feed_id: feed.id! }));
      const inserted = await insertArticles(withFeedId);
      // 对无摘要的新文章自动提取；有 DOI 时后端直接查 OpenAlex/CrossRef
      const needSummary = inserted.filter(a => !a.summary && a.url && a.id != null);
      let extracted = 0;
      for (const [i, a] of needSummary.entries()) {
        extractProgress = { current: i + 1, total: needSummary.length };
        try {
          const summary = await invoke<string>("extract_abstract", { url: a.url, doi: a.doi });
          if (summary) {
            await updateArticleSummary(a.id!, summary);
            extracted++;
          }
        } catch {
          // 单篇失败不影响整体
        }
      }
      successMsg = extracted > 0
        ? `${journal.name}: ${localized("feeds.refresh_ok")} (${extracted} ${localized("feeds.abstracts_extracted")})`
        : `${journal.name}: ${localized("feeds.refresh_ok")}`;
    } catch (e) {
      console.error(String(e));
      error = localized("error.fetch_failed");
    } finally {
      refreshingUrl = null;
      extractProgress = null;
    }
  }
</script>

<SplitPane storageKey="catalog-pane" leftWidth={256}>
  {#snippet left()}
    <aside class="h-full bg-[#F5F0E8] overflow-y-auto">
      <div class="p-4 border-b border-[#D4C8B0]">
        <h2 class="font-serif text-lg text-[#2C2416] tracking-wide">{localized("catalog.title")}</h2>
      </div>
      <nav class="p-3 space-y-0.5">
        <button
          onclick={() => selectedCategory = ""}
          class="w-full text-left px-3 py-2 text-sm transition-colors
            {selectedCategory === ""
              ? 'bg-[#8B1A2B]/10 text-[#8B1A2B] border-l-2 border-[#8B1A2B] font-medium'
              : 'text-[#6B5E4A] hover:bg-[#D4C8B0]/30'}"
        >
          {localized("catalog.all_categories")}
        </button>
        {#each CATEGORIES as category}
          <button
            onclick={() => selectedCategory = category}
            class="w-full text-left px-3 py-2 text-sm transition-colors
              {selectedCategory === category
                ? 'bg-[#8B1A2B]/10 text-[#8B1A2B] border-l-2 border-[#8B1A2B] font-medium'
                : 'text-[#6B5E4A] hover:bg-[#D4C8B0]/30'}"
          >
            {category}
          </button>
        {/each}
      </nav>
    </aside>
  {/snippet}
  {#snippet right()}
    <div class="p-8">
    {#if error}
      <p class="text-sm text-red-700 mb-4">{error}</p>
    {/if}
    {#if successMsg}
      <p class="text-sm text-green-700 mb-4">{successMsg}</p>
    {/if}

    <div class="space-y-0">
      {#each visibleJournals as journal}
        {@const subscribed = subscribedUrls.has(journal.rssUrl)}
        <div class="bg-white border border-[#D4C8B0] border-b-0 last:border-b p-4 flex items-center justify-between hover:bg-[#FAF7F2] transition-colors">
          <div class="min-w-0 flex-1">
            <p class="font-medium text-[#2C2416] text-sm">{journal.name}</p>
            <p class="text-xs text-[#6B5E4A] mt-0.5">ISSN {journal.issn}</p>
          </div>
          <div class="flex items-center gap-2 ml-4 shrink-0">
            {#if subscribed}
              <button
                onclick={() => handleRefresh(journal)}
                disabled={refreshingUrl === journal.rssUrl}
                class="text-xs text-[#8B1A2B] hover:underline disabled:opacity-50"
              >
                {refreshingUrl === journal.rssUrl
                  ? extractProgress
                    ? `${localized("feeds.extracting")} ${extractProgress.current}/${extractProgress.total}`
                    : localized("feeds.refreshing")
                  : localized("catalog.refresh")}
              </button>
            {/if}
            <button
              onclick={() => subscribed ? handleUnsubscribe(journal) : handleSubscribe(journal)}
              disabled={pendingUrl === journal.rssUrl}
              class="px-4 py-1.5 text-xs text-white transition-colors disabled:opacity-50
                {subscribed ? 'bg-[#3D6B4F] hover:bg-[#2f5540]' : 'bg-[#8B1A2B] hover:bg-[#6d1522]'}"
            >
              {subscribed ? localized("catalog.subscribed") : localized("catalog.subscribe")}
            </button>
          </div>
        </div>
      {/each}
    </div>
    </div>
  {/snippet}
</SplitPane>
