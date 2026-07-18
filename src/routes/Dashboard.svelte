<script lang="ts">
  import { getArticles, getFeeds, getTags } from "../lib/db";
  import { t } from "../lib/i18n";

  let stats = $state({ feeds: 0, articles: 0, unread: 0, tags: 0 });

  $effect(() => {
    (async () => {
      const feeds = await getFeeds();
      const articles = await getArticles();
      const tags = await getTags();
      stats = {
        feeds: feeds.length,
        articles: articles.length,
        unread: articles.filter(a => !a.is_read).length,
        tags: tags.length,
      };
    })();
  });
</script>

<div class="p-8">
  <h2 class="font-serif text-2xl text-[#2C2416] mb-6 tracking-wide">{t("nav.dashboard")}</h2>

  <div class="grid grid-cols-4 gap-4">
    {#each [
      { label: t("dashboard.total_feeds"), value: stats.feeds },
      { label: t("dashboard.total_articles"), value: stats.articles },
      { label: t("dashboard.unread"), value: stats.unread },
      { label: t("dashboard.tags"), value: stats.tags },
    ] as stat}
      <div class="bg-white border border-[#D4C8B0] p-5">
        <p class="text-xs text-[#6B5E4A] uppercase tracking-widest font-medium">{stat.label}</p>
        <p class="font-serif text-3xl text-[#2C2416] mt-2">{stat.value}</p>
      </div>
    {/each}
  </div>
</div>
