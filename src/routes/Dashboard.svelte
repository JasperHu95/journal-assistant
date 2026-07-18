<script lang="ts">
  import { getArticles, getFeeds, getTags } from "../lib/db";
  import { t, onLangChange } from "../lib/i18n";

  let stats = $state({ feeds: 0, articles: 0, unread: 0, tags: 0 });

  let langVersion = $state(0);
  onLangChange(() => langVersion++);
  function localized(key: string): string {
    langVersion;
    return t(key);
  }

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
</div>
