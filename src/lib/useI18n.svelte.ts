import { onLangChange, t } from "./i18n";

// 注意：$state 等 runes 只在 .svelte 和 .svelte.ts/.svelte.js 文件中生效，
// 因此本文件必须是 .svelte.ts 后缀
export function useI18n() {
  // 语言切换时 bump 版本号，触发引用 localized() 的组件重渲染
  let langVersion = $state(0);
  onLangChange(() => langVersion++);
  return function localized(key: string): string {
    langVersion;
    return t(key);
  };
}
