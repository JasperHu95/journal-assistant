<script lang="ts">
  import Sidebar from "./components/Sidebar.svelte";
  import Dashboard from "./routes/Dashboard.svelte";
  import FeedsPage from "./routes/FeedsPage.svelte";
  import JournalPage from "./routes/JournalPage.svelte";
  import CatalogPage from "./routes/CatalogPage.svelte";
  import TagsPage from "./routes/TagsPage.svelte";
  import { initDb, getFeedsWithArticleCount, insertArticles, updateFeedTitle, type Article, type FeedWithCount } from "./lib/db";
  import { catalogNameForUrl } from "./lib/journal-catalog";
  import { invoke } from "@tauri-apps/api/core";
  import { useI18n } from "./lib/useI18n.svelte";

  let currentRoute = $state("/");
  let dbReady = $state(false);
  let dbError = $state("");
  // 期刊列表与当前选中期刊提升到 App 层，供 Sidebar 与 JournalPage 共享
  let feeds = $state<FeedWithCount[]>([]);
  let selectedFeedId = $state<number | null>(null);

  const localized = useI18n();

  // 刷新 App 层的期刊列表（供 CatalogPage/FeedsPage 订阅变更后调用）
  async function refreshFeeds() {
    try {
      feeds = await getFeedsWithArticleCount();
      // 内置期刊的显示名统一为期刊名；顺带修正历史订阅中带出版社前缀的旧标题
      for (const f of feeds) {
        const name = catalogNameForUrl(f.url);
        if (name != null && f.title !== name && f.id != null) {
          await updateFeedTitle(f.id, name);
          f.title = name;
        }
      }
    } catch (e) {
      console.error("Failed to load feeds:", e);
    }
  }

  // 一键刷新所有源：逐个调用 refresh_feed，单个失败不中断整体
  let refreshingAll = $state(false);
  let refreshProgress = $state<{ current: number; total: number } | null>(null);

  async function refreshAll() {
    if (refreshingAll || feeds.length === 0) return;
    refreshingAll = true;
    try {
      const total = feeds.length;
      for (const [i, feed] of feeds.entries()) {
        refreshProgress = { current: i + 1, total };
        try {
          const articles = await invoke<Article[]>("refresh_feed", { feedUrl: feed.url });
          await insertArticles(articles.map((a) => ({ ...a, feed_id: feed.id! })));
        } catch (e) {
          console.error(`Failed to refresh feed ${feed.url}:`, e);
        }
      }
      // 全部完成后刷新 App 层 feeds，更新文章数/未读数
      await refreshFeeds();
    } finally {
      refreshingAll = false;
      refreshProgress = null;
    }
  }

  $effect(() => {
    initDb()
      .then(async () => {
        dbReady = true;
        await refreshFeeds();
        // 开启"自动刷新"开关时，启动后自动执行一次一键刷新
        if (localStorage.getItem("auto_refresh") === "1") refreshAll();
      })
      .catch((e) => {
        console.error("Database init failed:", e);
        dbError = String(e);
      });
  });

  function navigate(route: string) {
    currentRoute = route;
  }

  // 点击 Sidebar 中的期刊子项：记录选中期刊并切换到期刊页
  function selectFeed(id: number | null) {
    selectedFeedId = id;
    navigate("/journals");
  }
</script>

<div class="flex h-screen bg-[#FAF7F2] font-sans">
  <Sidebar {currentRoute} {navigate} {feeds} {selectedFeedId} {selectFeed} {refreshFeeds} {refreshAll} {refreshingAll} {refreshProgress} />
  <main class="flex-1 overflow-auto">
    {#if dbError}
      <div class="flex flex-col items-center justify-center h-full gap-4">
        <p class="text-[#8B1A2B] text-sm font-medium">{localized("common.db_error")}</p>
        <p class="text-[#6B5E4A] text-xs max-w-md text-center">{dbError}</p>
      </div>
    {:else if !dbReady}
      <div class="flex items-center justify-center h-full">
        <p class="text-[#6B5E4A] text-sm">{localized("common.loading")}</p>
      </div>
    {:else if currentRoute === "/"}
      <Dashboard />
    {:else if currentRoute === "/feeds"}
      <FeedsPage {refreshFeeds} />
    {:else if currentRoute === "/catalog"}
      <CatalogPage {refreshFeeds} />
    {:else if currentRoute === "/journals"}
      <JournalPage {feeds} {selectedFeedId} />
    {:else if currentRoute === "/tags"}
      <TagsPage />
    {/if}
  </main>
</div>
