<script lang="ts">
  import Sidebar from "./components/Sidebar.svelte";
  import Dashboard from "./routes/Dashboard.svelte";
  import FeedsPage from "./routes/FeedsPage.svelte";
  import ArticlesPage from "./routes/ArticlesPage.svelte";
  import TagsPage from "./routes/TagsPage.svelte";
  import { initDb } from "./lib/db";
  import { t } from "./lib/i18n";
  import { onMount } from "svelte";

  let currentRoute = $state("/");
  let dbReady = $state(false);

  onMount(async () => {
    await initDb();
    dbReady = true;
  });

  function navigate(route: string) {
    currentRoute = route;
  }
</script>

<div class="flex h-screen bg-[#FAF7F2] font-sans">
  <Sidebar {currentRoute} {navigate} />
  <main class="flex-1 overflow-auto">
    {#if !dbReady}
      <div class="flex items-center justify-center h-full">
        <p class="text-[#6B5E4A] text-sm">{t("common.loading")}</p>
      </div>
    {:else if currentRoute === "/"}
      <Dashboard />
    {:else if currentRoute === "/feeds"}
      <FeedsPage />
    {:else if currentRoute === "/articles"}
      <ArticlesPage />
    {:else if currentRoute === "/tags"}
      <TagsPage />
    {/if}
  </main>
</div>
