import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { useDark, useToggle } from '@vueuse/core';

export const useAppStore = defineStore('app', () => {
  // 主题：dark / light
  const isDark = useDark({
    selector: 'body',
    attribute: 'arco-theme',
    valueDark: 'dark',
    valueLight: '',
    storageKey: 'cradle-theme',
  });
  const toggleDark = useToggle(isDark);

  // 侧栏折叠（默认强制展开，不记忆折叠状态，避免用户误操作后无法恢复）
  const menuCollapse = ref(false);
  // 每次页面加载强制展开（用户若需要折叠，可再点击折叠按钮）
  menuCollapse.value = false;

  // 主题色（Arco 的 primary 色板）
  const themeColor = ref(localStorage.getItem('cradle_theme_color') || '#165dff');
  watch(themeColor, (v) => {
    localStorage.setItem('cradle_theme_color', v);
    applyThemeColor(v);
  });

  function applyThemeColor(hex: string) {
    // 简化：直接设置 CSS 变量（实际 Arco 主题切换需要完整色板，这里做基础覆盖）
    const root = document.documentElement;
    root.style.setProperty('--primary-6', hexToRgb(hex));
  }

  function hexToRgb(hex: string): string {
    const h = hex.replace('#', '');
    const r = parseInt(h.substring(0, 2), 16);
    const g = parseInt(h.substring(2, 4), 16);
    const b = parseInt(h.substring(4, 6), 16);
    return `${r}, ${g}, ${b}`;
  }

  // 初始化主题色
  applyThemeColor(themeColor.value);

  return {
    isDark,
    toggleDark,
    menuCollapse,
    themeColor,
  };
});
