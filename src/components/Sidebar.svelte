<script lang="ts">
  import { toggleLang } from "../lib/i18n";
  import { useI18n } from "../lib/useI18n.svelte";
  import { insertArticles, type Article, type FeedWithCount } from "../lib/db";
  import { invoke } from "@tauri-apps/api/core";

  let {
    currentRoute,
    navigate,
    feeds,
    selectedFeedId,
    selectFeed,
    refreshFeeds,
  }: {
    currentRoute: string;
    navigate: (route: string) => void;
    feeds: FeedWithCount[];
    selectedFeedId: number | null;
    selectFeed: (id: number | null) => void;
    refreshFeeds: () => Promise<void>;
  } = $props();

  const localized = useI18n();

  // “期刊”父项的展开/折叠状态；当前选中的期刊 ID 由 App 层通过 selectedFeedId 传入
  let journalsExpanded = $state(false);

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

  const navItems = [
    { path: "/", labelKey: "nav.dashboard" },
    { path: "/catalog", labelKey: "nav.builtin_rss" },
    { path: "/feeds", labelKey: "nav.feeds" },
    { path: "/journals", labelKey: "nav.journals" },
    { path: "/tags", labelKey: "nav.tags" },
  ];

  // 点击“期刊”父项：切换子列表展开状态，并进入期刊页
  function toggleJournals() {
    journalsExpanded = !journalsExpanded;
    navigate("/journals");
  }
</script>

<aside class="w-60 bg-[#F5F0E8] border-r border-[#D4C8B0] flex flex-col">
  <div class="p-4 border-b border-[#D4C8B0]">
    <h1 class="font-serif text-lg text-[#2C2416] tracking-wide">{localized("app.title")}</h1>
  </div>

  <nav class="flex-1 p-3 space-y-0.5 overflow-auto">
    {#each navItems as item}
      {#if item.path === "/journals"}
        <button
          onclick={toggleJournals}
          aria-expanded={journalsExpanded}
          title={localized("nav.journals.expand")}
          class="w-full text-left px-3 py-2 text-sm transition-colors flex items-center justify-between
            {currentRoute === item.path
              ? 'bg-[#8B1A2B]/10 text-[#8B1A2B] border-l-2 border-[#8B1A2B] font-medium'
              : 'text-[#6B5E4A] hover:bg-[#D4C8B0]/30'}"
        >
          <span>{localized(item.labelKey)}</span>
          <span class="text-xs">{journalsExpanded ? "▾" : "▸"}</span>
        </button>
        {#if journalsExpanded}
          <div class="ml-3 border-l border-[#D4C8B0] pl-1 space-y-0.5">
            {#each feeds as feed}
              <button
                onclick={() => selectFeed(feed.id)}
                class="w-full text-left px-2 py-1.5 text-xs transition-colors flex items-center justify-between gap-2
                  {currentRoute === '/journals' && selectedFeedId === feed.id
                    ? 'text-[#8B1A2B] font-medium bg-[#8B1A2B]/5'
                    : 'text-[#6B5E4A] hover:bg-[#D4C8B0]/30'}"
              >
                <span class="truncate">{feed.title}</span>
                <span class="shrink-0 {feed.unread_count > 0 ? 'text-[#8B1A2B] font-medium' : 'text-[#9B9B8A]'}">
                  {feed.unread_count}
                </span>
              </button>
            {:else}
              <p class="px-2 py-1.5 text-xs text-[#9B9B8A] italic">{localized("feeds.empty")}</p>
            {/each}
          </div>
        {/if}
      {:else}
        <button
          onclick={() => navigate(item.path)}
          class="w-full text-left px-3 py-2 text-sm transition-colors
            {currentRoute === item.path
              ? 'bg-[#8B1A2B]/10 text-[#8B1A2B] border-l-2 border-[#8B1A2B] font-medium'
              : 'text-[#6B5E4A] hover:bg-[#D4C8B0]/30'}"
        >
          {localized(item.labelKey)}
        </button>
      {/if}
    {/each}
  </nav>

  <div class="p-3 border-t border-[#D4C8B0] space-y-2">
    <button
      onclick={refreshAll}
      disabled={refreshingAll || feeds.length === 0}
      class="w-full text-left px-3 py-1.5 text-xs text-[#6B5E4A] hover:bg-[#D4C8B0]/30 transition-colors border border-[#D4C8B0] disabled:opacity-50"
    >
      {refreshProgress
        ? localized("feeds.refresh_all_progress")
            .replace("{current}", String(refreshProgress.current))
            .replace("{total}", String(refreshProgress.total))
        : localized("feeds.refresh_all")}
    </button>
    <button
      onclick={toggleLang}
      class="w-full text-left px-3 py-1.5 text-xs text-[#6B5E4A] hover:bg-[#D4C8B0]/30 transition-colors border border-[#D4C8B0]"
    >
      {localized("lang.switch")}
    </button>
    <p class="text-xs text-[#9B9B8A]">{localized("version")}</p>
  </div>
</aside>
