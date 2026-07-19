<script lang="ts">
  import { getSetting, setSetting } from "../lib/db";
  import { t, onLangChange } from "../lib/i18n";

  let apiKey = $state("");
  let saved = $state(false);

  // i18n 响应式
  let langVersion = $state(0);
  onLangChange(() => langVersion++);
  function localized(key: string): string {
    langVersion;
    return t(key);
  }

  // 进入页面时读取已保存的 Key
  $effect(() => {
    (async () => {
      apiKey = (await getSetting("deepseek_api_key")) ?? "";
    })();
  });

  async function handleSave() {
    await setSetting("deepseek_api_key", apiKey.trim());
    saved = true;
    setTimeout(() => (saved = false), 2000);
  }
</script>

<div class="p-8 max-w-xl">
  <h2 class="font-serif text-xl text-[#2C2416] tracking-wide mb-6">{localized("settings.title")}</h2>

  <label for="deepseek-api-key" class="block text-sm text-[#2C2416] font-medium mb-2">
    {localized("settings.api_key")}
  </label>
  <div class="flex gap-2">
    <input
      id="deepseek-api-key"
      type="password"
      bind:value={apiKey}
      placeholder={localized("settings.api_key_placeholder")}
      class="flex-1 px-3 py-2 text-sm border border-[#D4C8B0] bg-white text-[#2C2416]
        focus:outline-none focus:border-[#8B1A2B]"
    />
    <button
      onclick={handleSave}
      class="px-4 py-2 bg-[#8B1A2B] text-white text-sm hover:bg-[#6d1522] transition-colors"
    >
      {localized("settings.save")}
    </button>
  </div>
  {#if saved}
    <p class="mt-2 text-xs text-[#6B5E4A]">{localized("settings.saved")}</p>
  {/if}
</div>
