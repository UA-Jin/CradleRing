<template>
  <a-layout class="layout-wrapper">
    <!-- 侧栏 -->
    <a-layout-sider
      :width="260"
      :collapsed-width="64"
      :collapsible="true"
      :collapsed="appStore.menuCollapse"
      @collapse="appStore.menuCollapse = $event"
      class="layout-sider"
      :style="{ position: 'fixed', height: '100vh', left: 0, top: 0, zIndex: 100 }"
    >
      <!-- Logo -->
      <div class="app-brand">
        <div class="app-brand-logo">
          <svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="16" cy="16" r="14" fill="url(#brand-grad)"/>
            <path d="M10 12L16 8L22 12V20L16 24L10 20V12Z" stroke="#fff" stroke-width="1.5" fill="none"/>
            <circle cx="16" cy="16" r="3" fill="#fff"/>
            <defs>
              <linearGradient id="brand-grad" x1="0" y1="0" x2="32" y2="32">
                <stop offset="0%" stop-color="#8c57ff"/>
                <stop offset="100%" stop-color="#16b1ff"/>
              </linearGradient>
            </defs>
          </svg>
        </div>
        <transition name="fade">
          <span v-if="!appStore.menuCollapse" class="app-brand-text logo-text">CradleRing</span>
        </transition>
      </div>
      <app-menu :collapsed="appStore.menuCollapse" />
    </a-layout-sider>

    <!-- 主内容区 -->
    <a-layout class="layout-page" :style="{ marginLeft: appStore.menuCollapse ? '64px' : '260px', transition: 'margin-left 0.25s ease' }">
      <!-- 顶部导航 -->
      <a-layout-header class="layout-header">
        <app-navbar />
      </a-layout-header>

      <!-- 页面内容 -->
      <a-layout-content class="layout-content">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </a-layout-content>
    </a-layout>
  </a-layout>
</template>

<script setup lang="ts">
import { useAppStore } from '@/stores/app';
import AppNavbar from './navbar/index.vue';
import AppMenu from './menu/index.vue';

const appStore = useAppStore();
</script>

<style lang="less" scoped>
.layout-wrapper {
  height: 100vh;
  background-color: var(--color-bg-2);
}

.layout-sider {
  background-color: var(--color-bg-1);
  border-right: 1px solid var(--color-border-1);
  :deep(.arco-layout-sider-children) {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
}

.app-brand {
  height: var(--navbar-height);
  display: flex;
  align-items: center;
  padding: 0 20px;
  gap: 12px;
  border-bottom: 1px solid var(--color-border-1);
  flex-shrink: 0;

  .app-brand-logo {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    svg { width: 32px; height: 32px; }
  }

  .app-brand-text {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: 0.5px;
    white-space: nowrap;
  }
}

.layout-header {
  height: var(--navbar-height);
  padding: 0;
  background-color: var(--color-bg-1);
  border-bottom: 1px solid var(--color-border-1);
  position: sticky;
  top: 0;
  z-index: 99;
  flex-shrink: 0;
}

.layout-content {
  background-color: var(--color-bg-2);
  overflow-y: auto;
  flex: 1;
}

@media (max-width: 768px) {
  .layout-page {
    margin-left: 64px !important;
  }
}
</style>
