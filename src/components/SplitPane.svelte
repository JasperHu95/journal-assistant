<script lang="ts">
  import type { Snippet } from "svelte";
  import { untrack } from "svelte";

  let {
    leftWidth = $bindable(256),
    minWidth = 200,
    maxWidth = 600,
    storageKey = "",
    left,
    right,
  }: {
    leftWidth?: number;
    minWidth?: number;
    maxWidth?: number;
    storageKey?: string;
    left: Snippet;
    right: Snippet;
  } = $props();

  // 从 localStorage 恢复上次拖拽的宽度（仅在组件初始化时读取一次）
  untrack(() => {
    if (storageKey) {
      const saved = localStorage.getItem(storageKey);
      if (saved) {
        const parsed = parseInt(saved);
        if (!Number.isNaN(parsed)) leftWidth = parsed;
      }
    }
  });

  let dragging = $state(false);
  let container = $state<HTMLDivElement | null>(null);

  function onPointerDown(e: PointerEvent) {
    dragging = true;
    // 捕获指针，拖出分隔条范围后仍能收到 move/up 事件
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    e.preventDefault();
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging || !container) return;
    // clientX 是视口坐标，需减去容器左缘偏移（如 Sidebar 占用的宽度）
    const offsetX = e.clientX - container.getBoundingClientRect().left;
    const max = Math.min(maxWidth, window.innerWidth / 2);
    leftWidth = Math.round(Math.min(max, Math.max(minWidth, offsetX)));
  }

  function onPointerUp() {
    if (!dragging) return;
    dragging = false;
    if (storageKey) localStorage.setItem(storageKey, String(leftWidth));
  }
</script>

<div bind:this={container} class="flex h-full {dragging ? 'select-none' : ''}">
  <div style="width: {leftWidth}px" class="shrink-0 overflow-auto">
    {@render left()}
  </div>
  <div
    role="separator"
    aria-orientation="vertical"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    class="w-1 shrink-0 cursor-col-resize hover:bg-[#8B1A2B]/30 transition-colors {dragging
      ? 'bg-[#8B1A2B]/30'
      : 'bg-[#D4C8B0]'}"
  ></div>
  <div class="flex-1 min-w-0 overflow-auto">
    {@render right()}
  </div>
</div>
