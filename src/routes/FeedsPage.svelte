<script lang="ts">
  import { getFeeds, addFeed, deleteFeed, type Feed } from "../lib/db";
  import { t } from "../lib/i18n";
  import { invoke } from "@tauri-apps/api/core";

  let feeds = $state<Feed[]>([]);
  let showAdd = $state(false);
  let feedUrl = $state("");
  let feedMode = $state<"url" | "discover">("url");
  let loading = $state(false);
  let error = $state("");

  async function loadFeeds() { feeds = await getFeeds(); }
  $effect(() => { loadFeeds(); });

  async function handleAdd() {
    if (!feedUrl.trim()) return;
    loading = true;
    error = "";
    try {
      if (feedMode === "url") {
        const result = await invoke<Feed>("add_feed", { url: feedUrl });
        await addFeed(result);
      } else {
        const discovered = await invoke<{ url: string; title: string | null }[]>("discover_feeds", { url: feedUrl });
        if (discovered.length === 0) { error = t("feeds.no_found"); return; }
        for (const d of discovered) {
          const result = await invoke<Feed>("add_feed", { url: d.url });
          await addFeed(result);
        }
      }
      feedUrl = "";
      showAdd = false;
      await loadFeeds();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleDelete(id: number) {
    await deleteFeed(id);
    await loadFeeds();
  }
</script>

<div class="p-8">
  <div class="flex items-center justify-between mb-6">
    <h2 class="font-serif text-2xl text-[#2C2416] tracking-wide">{t("feeds.title")}</h2>
    <button
      onclick={() => showAdd = !showAdd}
      class="px-4 py-2 bg-[#8B1A2B] text-white text-sm hover:bg-[#6d1522] transition-colors"
    >
      {showAdd ? t("feeds.cancel") : t("feeds.add")}
    </button>
  </div>

  {#if showAdd}
    <div class="bg-white border border-[#D4C8B0] p-5 mb-6">
      <div class="flex gap-6 mb-3">
        <label class="flex items-center gap-2 text-sm text-[#2C2416] cursor-pointer">
          <input type="radio" bind:group={feedMode} value="url" class="accent-[#8B1A2B]" />
          {t("feeds.mode_url")}
        </label>
        <label class="flex items-center gap-2 text-sm text-[#2C2416] cursor-pointer">
          <input type="radio" bind:group={feedMode} value="discover" class="accent-[#8B1A2B]" />
          {t("feeds.mode_discover")}
        </label>
      </div>
      <div class="flex gap-2">
        <input
          bind:value={feedUrl}
          placeholder={feedMode === "url" ? t("feeds.url_placeholder") : t("feeds.discover_placeholder")}
          class="flex-1 border border-[#D4C8B0] px-3 py-2 text-sm text-[#2C2416] bg-[#FAF7F2] focus:outline-none focus:border-[#8B1A2B]"
        />
        <button
          onclick={handleAdd}
          disabled={loading}
          class="px-4 py-2 bg-[#8B1A2B] text-white text-sm hover:bg-[#6d1522] disabled:opacity-50 transition-colors"
        >
          {loading ? t("feeds.adding") : t("feeds.add")}
        </button>
      </div>
      {#if error}
        <p class="text-sm text-red-700 mt-2">{error}</p>
      {/if}
    </div>
  {/if}

  <div class="space-y-0">
    {#each feeds as feed}
      <div class="bg-white border border-[#D4C8B0] border-b-0 last:border-b p-4 flex items-center justify-between hover:bg-[#FAF7F2] transition-colors">
        <div class="min-w-0 flex-1">
          <p class="font-medium text-[#2C2416] text-sm">{feed.title}</p>
          <p class="text-xs text-[#6B5E4A] mt-0.5 truncate">{feed.url}</p>
        </div>
        <button
          onclick={() => handleDelete(feed.id!)}
          class="text-xs text-[#8B1A2B] hover:underline ml-4 shrink-0"
        >
          {t("feeds.remove")}
        </button>
      </div>
    {:else}
      <p class="text-sm text-[#6B5E4A] py-12 text-center italic">{t("feeds.empty")}</p>
    {/each}
  </div>
</div>
