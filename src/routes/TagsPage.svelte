<script lang="ts">
  import { getTags, createTag, deleteTag, type Tag } from "../lib/db";
  import { t, onLangChange } from "../lib/i18n";

  let tags = $state<Tag[]>([]);
  let newName = $state("");
  let newColor = $state("#8B1A2B");

  let langVersion = $state(0);
  onLangChange(() => langVersion++);
  function localized(key: string): string {
    langVersion;
    return t(key);
  }

  async function loadTags() { tags = await getTags(); }
  $effect(() => { loadTags(); });

  async function handleCreate() {
    if (!newName.trim()) return;
    await createTag(newName.trim(), newColor);
    newName = "";
    await loadTags();
  }

  async function handleDelete(id: number) {
    await deleteTag(id);
    await loadTags();
  }
</script>

<div class="p-8">
  <h2 class="font-serif text-2xl text-[#2C2416] mb-6 tracking-wide">{localized("tags.title")}</h2>

  <div class="bg-white border border-[#D4C8B0] p-5 mb-6">
    <div class="flex gap-3 items-end">
      <div class="flex-1">
        <label for="tag-name" class="text-xs text-[#6B5E4A] block mb-1.5">{localized("tags.name")}</label>
        <input
          id="tag-name"
          bind:value={newName}
          placeholder={localized("tags.placeholder")}
          class="w-full border border-[#D4C8B0] px-3 py-2 text-sm text-[#2C2416] bg-[#FAF7F2] focus:outline-none focus:border-[#8B1A2B]"
          onkeydown={(e) => e.key === "Enter" && handleCreate()}
        />
      </div>
      <div>
        <label for="tag-color" class="text-xs text-[#6B5E4A] block mb-1.5">{localized("tags.color")}</label>
        <input id="tag-color" type="color" bind:value={newColor}
          class="h-[38px] w-10 border border-[#D4C8B0] cursor-pointer" />
      </div>
      <button
        onclick={handleCreate}
        class="px-5 py-2 bg-[#8B1A2B] text-white text-sm hover:bg-[#6d1522] transition-colors"
      >
        {localized("tags.create")}
      </button>
    </div>
  </div>

  <div class="space-y-0">
    {#each tags as tag}
      <div class="bg-white border border-[#D4C8B0] border-b-0 last:border-b p-4 flex items-center justify-between hover:bg-[#FAF7F2] transition-colors">
        <div class="flex items-center gap-3">
          <span class="w-3 h-3 rounded-full shrink-0" style="background-color: {tag.color || '#8B1A2B'}"></span>
          <span class="text-sm text-[#2C2416] italic font-serif">{tag.name}</span>
        </div>
        <button
          onclick={() => handleDelete(tag.id!)}
          class="text-xs text-[#8B1A2B] hover:underline"
        >
          {localized("tags.remove")}
        </button>
      </div>
    {:else}
      <p class="text-sm text-[#6B5E4A] py-12 text-center italic">{localized("tags.empty")}</p>
    {/each}
  </div>
</div>
