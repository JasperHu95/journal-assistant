<script lang="ts">
  import Sidebar from "./components/Sidebar.svelte";
  import Dashboard from "./routes/Dashboard.svelte";
  import FeedsPage from "./routes/FeedsPage.svelte";
  import JournalPage from "./routes/JournalPage.svelte";
  import CatalogPage from "./routes/CatalogPage.svelte";
  import TagsPage from "./routes/TagsPage.svelte";
  import { initDb, getFeedsWithArticleCount, type FeedWithCount } from "./lib/db";
  import { useI18n } from "./lib/useI18n.svelte";

  let currentRoute = $state("/");
  let dbReady = $state(false);
  let dbError = $state("");
  // 期刊列表与当前选中期刊提升到 App 层，供 Sidebar 与 JournalPage 共享
  let feeds = $state<FeedWithCount[]>([]);
  let selectedFeedId = $state<number | null>(null);

  const localized = useI18n();

  $effect(() => {
    initDb()
      .then(async () => {
        dbReady = true;
        try {
          feeds = await getFeedsWithArticleCount();
        } catch (e) {
          console.error("Failed to load feeds:", e);
          feeds = [];
        }
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
  <Sidebar {currentRoute} {navigate} {feeds} {selectedFeedId} {selectFeed} />
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
      <FeedsPage />
    {:else if currentRoute === "/catalog"}
      <CatalogPage />
    {:else if currentRoute === "/journals"}
      <JournalPage {feeds} {selectedFeedId} />
    {:else if currentRoute === "/tags"}
      <TagsPage />
    {/if}
  </main>
</div>
