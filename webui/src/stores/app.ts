import { defineStore } from 'pinia';
import { ref, watch, onMounted } from 'vue';

export type ThemeMode = 'light' | 'dark' | 'system';

export const useAppStore = defineStore('app', () => {
  // ---------- 主题模式：light / dark / system（跟随系统）----------
  const themeMode = ref<ThemeMode>((localStorage.getItem('cradle-theme-mode') as ThemeMode) || 'system');
  const isDark = ref(false);

  const systemDarkMedia = window.matchMedia('(prefers-color-scheme: dark)');

  function applyTheme() {
    const dark = themeMode.value === 'dark' || (themeMode.value === 'system' && systemDarkMedia.matches);
    isDark.value = dark;
    if (dark) {
      document.body.setAttribute('arco-theme', 'dark');
    } else {
      document.body.removeAttribute('arco-theme');
    }
  }

  function setThemeMode(mode: ThemeMode) {
    themeMode.value = mode;
    localStorage.setItem('cradle-theme-mode', mode);
    applyTheme();
  }

  // 跟随系统变化
  systemDarkMedia.addEventListener('change', () => {
    if (themeMode.value === 'system') applyTheme();
  });

  watch(themeMode, applyTheme);

  // ---------- 侧栏折叠（记忆状态，靶心小圆点切换）----------
  const menuCollapse = ref(localStorage.getItem('cradle-menu-collapse') === '1');
  // hover 浮出展开（折叠时鼠标悬停临时展开）
  const menuHoverExpand = ref(false);

  function toggleMenuCollapse() {
    menuCollapse.value = !menuCollapse.value;
    localStorage.setItem('cradle-menu-collapse', menuCollapse.value ? '1' : '0');
  }

  // ---------- 语言 ----------
  const locale = ref(localStorage.getItem('cradle-locale') || 'zh-CN');
  function setLocale(l: string) {
    locale.value = l;
    localStorage.setItem('cradle-locale', l);
  }

  // ---------- 主题色（固定 Materialize 紫，不再允许老蓝色覆盖）----------
  const themeColor = ref('#8c57ff');

  onMounted(() => {
    applyTheme();
  });

  return {
    themeMode,
    isDark,
    setThemeMode,
    applyTheme,
    menuCollapse,
    menuHoverExpand,
    toggleMenuCollapse,
    locale,
    setLocale,
    themeColor,
  };
});
