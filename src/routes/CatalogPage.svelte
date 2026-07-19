<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { CATEGORIES, JOURNAL_CATALOG, type CatalogJournal } from "../lib/journal-catalog";
  import { getFeeds, addFeed, deleteFeed, type Feed } from "../lib/db";
  import { useI18n } from "../lib/useI18n.svelte";

  // 空字符串表示"全部分类"
  let selectedCategory = $state("");
  let feeds = $state<Feed[]>([]);
  let pendingUrl = $state<string | null>(null);
  let error = $state("");

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
    if (!confirm(localized("common.confirm_delete"))) return;
    pendingUrl = journal.rssUrl;
    error = "";
    try {
      await deleteFeed(feed.id);
      await loadFeeds();
    } catch (e) {
      console.error(String(e));
      error = localized("error.fetch_failed");
    } finally {
      pendingUrl = null;
    }
  }
</script>

<div class="flex h-full">
  <aside class="w-64 shrink-0 border-r border-[#D4C8B0] bg-[#F5F0E8] overflow-y-auto">
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

  <div class="flex-1 overflow-y-auto p-8">
    {#if error}
      <p class="text-sm text-red-700 mb-4">{error}</p>
    {/if}

    <div class="space-y-0">
      {#each visibleJournals as journal}
        {@const subscribed = subscribedUrls.has(journal.rssUrl)}
        <div class="bg-white border border-[#D4C8B0] border-b-0 last:border-b p-4 flex items-center justify-between hover:bg-[#FAF7F2] transition-colors">
          <div class="min-w-0 flex-1">
            <p class="font-medium text-[#2C2416] text-sm">{journal.name}</p>
            <p class="text-xs text-[#6B5E4A] mt-0.5">ISSN {journal.issn}</p>
          </div>
          <button
            onclick={() => subscribed ? handleUnsubscribe(journal) : handleSubscribe(journal)}
            disabled={pendingUrl === journal.rssUrl}
            class="ml-4 shrink-0 px-4 py-1.5 text-xs text-white transition-colors disabled:opacity-50
              {subscribed ? 'bg-[#3D6B4F] hover:bg-[#2f5540]' : 'bg-[#8B1A2B] hover:bg-[#6d1522]'}"
          >
            {subscribed ? localized("catalog.subscribed") : localized("catalog.subscribe")}
          </button>
        </div>
      {/each}
    </div>
  </div>
</div>
