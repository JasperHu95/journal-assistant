<script lang="ts">
  import { getArticles, getFeedsWithArticleCount, getTags, formatRelativeTime, type Article } from "../lib/db";
  import { useI18n } from "../lib/useI18n.svelte";

  let stats = $state({ feeds: 0, articles: 0, unread: 0, tags: 0 });
  let recentArticles = $state<{ article: Article; feedTitle: string }[]>([]);

  const localized = useI18n();

  $effect(() => {
    (async () => {
      const feeds = await getFeedsWithArticleCount();
      const articles = await getArticles();
      const tags = await getTags();
      stats = {
        feeds: feeds.length,
        articles: articles.length,
        unread: articles.filter(a => !a.is_read).length,
        tags: tags.length,
      };
      // getArticles 已按 published_at 降序返回，直接取前 5 篇
      const feedTitleById = new Map(feeds.map((f) => [f.id, f.title]));
      recentArticles = articles.slice(0, 5).map((article) => ({
        article,
        feedTitle: feedTitleById.get(article.feed_id) ?? "",
      }));
    })();
  });
</script>

<div class="p-8">
  <h2 class="font-serif text-2xl text-[#2C2416] mb-6 tracking-wide">{localized("nav.dashboard")}</h2>

  <div class="grid grid-cols-4 gap-4">
    {#each [
      { label: localized("dashboard.total_feeds"), value: stats.feeds },
      { label: localized("dashboard.total_articles"), value: stats.articles },
      { label: localized("dashboard.unread"), value: stats.unread },
      { label: localized("dashboard.tags"), value: stats.tags },
    ] as stat}
      <div class="bg-white border border-[#D4C8B0] p-5">
        <p class="text-xs text-[#6B5E4A] uppercase tracking-widest font-medium">{stat.label}</p>
        <p class="font-serif text-3xl text-[#2C2416] mt-2">{stat.value}</p>
      </div>
    {/each}
  </div>

  <h3 class="font-serif text-lg text-[#2C2416] mt-8 mb-4 tracking-wide">{localized("dashboard.recent_articles")}</h3>
  <div class="bg-white border border-[#D4C8B0]">
    {#each recentArticles as item}
      <div class="p-4 border-b border-[#D4C8B0]/40 last:border-b-0">
        <p class="text-sm text-[#2C2416] leading-snug">{item.article.title}</p>
        <p class="text-xs text-[#6B5E4A] mt-1">
          {item.feedTitle}
          {#if formatRelativeTime(item.article.published_at)}
            <span class="text-[#D4C8B0]"> | </span>{formatRelativeTime(item.article.published_at)}
          {/if}
        </p>
      </div>
    {:else}
      <p class="text-sm text-[#6B5E4A] p-6 text-center italic">{localized("articles.empty")}</p>
    {/each}
  </div>
</div>
