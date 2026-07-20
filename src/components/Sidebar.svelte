<script lang="ts">
  import { toggleLang } from "../lib/i18n";
  import { useI18n } from "../lib/useI18n.svelte";

  let { currentRoute, navigate } = $props();

  const localized = useI18n();

  const navItems = [
    { path: "/", labelKey: "nav.dashboard" },
    { path: "/catalog", labelKey: "nav.builtin_rss" },
    { path: "/feeds", labelKey: "nav.feeds" },
    { path: "/journals", labelKey: "nav.journals" },
    { path: "/tags", labelKey: "nav.tags" },
  ];
</script>

<aside class="w-60 bg-[#F5F0E8] border-r border-[#D4C8B0] flex flex-col">
  <div class="p-4 border-b border-[#D4C8B0]">
    <h1 class="font-serif text-lg text-[#2C2416] tracking-wide">{localized("app.title")}</h1>
  </div>

  <nav class="flex-1 p-3 space-y-0.5">
    {#each navItems as item}
      <button
        onclick={() => navigate(item.path)}
        class="w-full text-left px-3 py-2 text-sm transition-colors
          {currentRoute === item.path
            ? 'bg-[#8B1A2B]/10 text-[#8B1A2B] border-l-2 border-[#8B1A2B] font-medium'
            : 'text-[#6B5E4A] hover:bg-[#D4C8B0]/30'}"
      >
        {localized(item.labelKey)}
      </button>
    {/each}
  </nav>

  <div class="p-3 border-t border-[#D4C8B0] space-y-2">
    <button
      onclick={toggleLang}
      class="w-full text-left px-3 py-1.5 text-xs text-[#6B5E4A] hover:bg-[#D4C8B0]/30 transition-colors border border-[#D4C8B0]"
    >
      {localized("lang.switch")}
    </button>
    <p class="text-xs text-[#9B9B8A]">{localized("version")}</p>
  </div>
</aside>
