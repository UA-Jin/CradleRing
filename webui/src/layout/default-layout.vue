<template>
  <a-layout class="layout">
    <a-layout-sider
      :width="220"
      :collapsed-width="48"
      :collapsible="true"
      :collapsed="appStore.menuCollapse"
      @collapse="(v) => { if (v !== appStore.menuCollapse) appStore.menuCollapse = v; }"
      class="layout-sider"
    >
      <div class="logo">
        <icon-storage class="logo-icon" />
        <transition name="fade">
          <span v-if="!appStore.menuCollapse" class="logo-text">CradleRing</span>
        </transition>
      </div>
      <app-menu :collapsed="appStore.menuCollapse" />
    </a-layout-sider>

    <a-layout>
      <a-layout-header class="layout-header">
        <app-navbar />
      </a-layout-header>

      <a-layout-content class="layout-content">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </a-layout-content>
    </a-layout>

    <global-setting />
  </a-layout>
</template>

<script setup lang="ts">
import { useAppStore } from '@/stores/app';
import AppNavbar from './navbar/index.vue';
import AppMenu from './menu/index.vue';
import GlobalSetting from './global-setting/index.vue';

const appStore = useAppStore();
</script>

<style lang="less" scoped>
.layout {
  height: 100vh;
}

.layout-sider {
  position: fixed;
  top: 0;
  left: 0;
  height: 100vh;
  background-color: var(--color-bg-1);
  border-right: 1px solid var(--color-border-1);
  z-index: 100;
  :deep(.arco-layout-sider-children) {
    display: flex;
    flex-direction: column;
  }
}

.logo {
  height: var(--navbar-height);
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 10px;
  border-bottom: 1px solid var(--color-border-1);
  overflow: hidden;

  .logo-icon {
    font-size: 24px;
    color: rgb(var(--primary-6));
    flex-shrink: 0;
  }
  .logo-text {
    font-size: 17px;
    font-weight: 700;
    color: var(--color-text-1);
    white-space: nowrap;
    letter-spacing: 0.5px;
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
}

.layout-content {
  margin-left: 220px;
  transition: margin-left 0.2s ease;
  background-color: var(--color-bg-2);
  overflow-y: auto;
}

// 折叠时补偿内容区 margin
:global(.arco-layout-sider-collapsed) ~ .arco-layout .layout-content {
  margin-left: 48px;
}

@media (max-width: 992px) {
  .layout-content {
    margin-left: 48px;
  }
}
</style>
