<script lang="ts">
  import Sidebar from "./components/Sidebar.svelte";
  import Dashboard from "./routes/Dashboard.svelte";
  import FeedsPage from "./routes/FeedsPage.svelte";
  import ArticlesPage from "./routes/ArticlesPage.svelte";
  import JournalPage from "./routes/JournalPage.svelte";
  import TagsPage from "./routes/TagsPage.svelte";
  import SettingsPage from "./routes/SettingsPage.svelte";
  import { initDb } from "./lib/db";
  import { useI18n } from "./lib/useI18n.svelte";

  let currentRoute = $state("/");
  let dbReady = $state(false);
  let dbError = $state("");

  const localized = useI18n();

  $effect(() => {
    initDb()
      .then(() => {
        dbReady = true;
      })
      .catch((e) => {
        console.error("Database init failed:", e);
        dbError = String(e);
      });
  });

  function navigate(route: string) {
    currentRoute = route;
  }
</script>

<div class="flex h-screen bg-[#FAF7F2] font-sans">
  <Sidebar {currentRoute} {navigate} />
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
    {:else if currentRoute === "/articles"}
      <ArticlesPage />
    {:else if currentRoute === "/journals"}
      <JournalPage />
    {:else if currentRoute === "/tags"}
      <TagsPage />
    {:else if currentRoute === "/settings"}
      <SettingsPage />
    {/if}
  </main>
</div>
