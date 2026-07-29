<script lang="ts">
  import { getFeeds, addFeed, deleteFeed, insertArticles, updateArticleSummary, type Feed } from "../lib/db";
  import { useI18n } from "../lib/useI18n.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let feeds = $state<Feed[]>([]);
  let showAdd = $state(false);
  let feedUrl = $state("");
  let feedMode = $state<"url" | "discover">("url");
  let loading = $state(false);
  let { refreshFeeds }: { refreshFeeds: () => Promise<void> } = $props();
  let refreshing = $state<number | null>(null);
  let extractProgress = $state<{ current: number; total: number } | null>(null);
  let error = $state("");
  let successMsg = $state("");

  const localized = useI18n();

  async function loadFeeds() { feeds = await getFeeds(); }
  $effect(() => { loadFeeds(); });

  async function handleAdd() {
    if (!feedUrl.trim()) return;
    loading = true;
    error = "";
    successMsg = "";
    try {
      if (feedMode === "url") {
        const result = await invoke<Feed>("add_feed", { url: feedUrl });
        await addFeed(result);
      } else {
        const discovered = await invoke<{ url: string; title: string | null }[]>("discover_feeds", { url: feedUrl });
        if (discovered.length === 0) { error = localized("feeds.no_found"); return; }
        for (const d of discovered) {
          const result = await invoke<Feed>("add_feed", { url: d.url });
          await addFeed(result);
        }
      }
      feedUrl = "";
      showAdd = false;
      await loadFeeds();
    await refreshFeeds();
    } catch (e) {
      console.error(String(e));
      error = localized("error.fetch_failed");
    } finally {
      loading = false;
    }
  }

  async function handleRefresh(feed: Feed) {
    refreshing = feed.id;
    error = "";
    successMsg = "";
    extractProgress = null;
    try {
      const articles = await invoke<{ feed_id: number; title: string; url: string | null; author: string | null; content: string | null; summary: string | null; doi: string | null; published_at: string | null }[]>(
        "refresh_feed",
        { feedUrl: feed.url }
      );
      // Assign the correct feed_id before inserting
      const withFeedId = articles.map(a => ({ ...a, feed_id: feed.id! }));
      const inserted = await insertArticles(withFeedId);

      // 学术期刊的 RSS 多不含摘要：对新插入且无摘要的文章逐篇提取
      // 有 DOI 时后端直接查 OpenAlex/CrossRef，不抓期刊页面（有反爬）
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
          // 单篇文章提取失败不影响整体
        }
      }

      successMsg = extracted > 0
        ? `${localized("feeds.refresh_ok")} (${extracted} ${localized("feeds.abstracts_extracted")})`
        : localized("feeds.refresh_ok");
    } catch (e) {
      error = String(e);
    } finally {
      refreshing = null;
      extractProgress = null;
    }
  }

  async function handleDelete(id: number) {
    await deleteFeed(id);
    await loadFeeds();
    await refreshFeeds();
  }
</script>

<div class="p-8">
  <div class="flex items-center justify-between mb-6">
    <h2 class="font-serif text-2xl text-[#2C2416] tracking-wide">{localized("feeds.title")}</h2>
    <button
      onclick={() => showAdd = !showAdd}
      class="px-4 py-2 bg-[#8B1A2B] text-white text-sm hover:bg-[#6d1522] transition-colors"
    >
      {showAdd ? localized("feeds.cancel") : localized("feeds.add")}
    </button>
  </div>

  {#if showAdd}
    <div class="bg-white border border-[#D4C8B0] p-5 mb-6">
      <div class="flex gap-6 mb-3">
        <label class="flex items-center gap-2 text-sm text-[#2C2416] cursor-pointer">
          <input type="radio" bind:group={feedMode} value="url" class="accent-[#8B1A2B]" />
          {localized("feeds.mode_url")}
        </label>
        <label class="flex items-center gap-2 text-sm text-[#2C2416] cursor-pointer">
          <input type="radio" bind:group={feedMode} value="discover" class="accent-[#8B1A2B]" />
          {localized("feeds.mode_discover")}
        </label>
      </div>
      <div class="flex gap-2">
        <input
          bind:value={feedUrl}
          placeholder={feedMode === "url" ? localized("feeds.url_placeholder") : localized("feeds.discover_placeholder")}
          class="flex-1 border border-[#D4C8B0] px-3 py-2 text-sm text-[#2C2416] bg-[#FAF7F2] focus:outline-none focus:border-[#8B1A2B]"
        />
        <button
          onclick={handleAdd}
          disabled={loading}
          class="px-4 py-2 bg-[#8B1A2B] text-white text-sm hover:bg-[#6d1522] disabled:opacity-50 transition-colors"
        >
          {loading ? localized("feeds.adding") : localized("feeds.add")}
        </button>
      </div>
      {#if error}
        <p class="text-sm text-red-700 mt-2">{error}</p>
      {/if}
    </div>
  {/if}

  {#if successMsg}
    <p class="text-sm text-green-700 mb-4">{successMsg}</p>
  {/if}

  <div class="space-y-0">
    {#each feeds as feed}
      <div class="bg-white border border-[#D4C8B0] border-b-0 last:border-b p-4 flex items-center justify-between hover:bg-[#FAF7F2] transition-colors">
        <div class="min-w-0 flex-1">
          <p class="font-medium text-[#2C2416] text-sm">{feed.title}</p>
          <p class="text-xs text-[#6B5E4A] mt-0.5 truncate">{feed.url}</p>
        </div>
        <div class="flex items-center gap-3 ml-4 shrink-0">
          <button
            onclick={() => handleRefresh(feed)}
            disabled={refreshing === feed.id}
            class="text-xs text-[#8B1A2B] hover:underline disabled:opacity-50"
          >
            {refreshing === feed.id
              ? extractProgress
                ? `${localized("feeds.extracting")} ${extractProgress.current}/${extractProgress.total}`
                : localized("feeds.refreshing")
              : localized("feeds.refresh")}
          </button>
          <button
            onclick={() => handleDelete(feed.id!)}
            class="text-xs text-[#6B5E4A] hover:underline"
          >
            {localized("feeds.remove")}
          </button>
        </div>
      </div>
    {:else}
      <p class="text-sm text-[#6B5E4A] py-12 text-center italic">{localized("feeds.empty")}</p>
    {/each}
  </div>
</div>
