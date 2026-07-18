<template>
  <a-layout class="layout-wrapper">
    <!-- 侧栏（白色，Materialize 风格） -->
    <aside
      class="layout-sider"
      :class="{ collapsed: appStore.menuCollapse, 'hover-expand': appStore.menuHoverExpand }"
      @mouseenter="onSiderEnter"
      @mouseleave="onSiderLeave"
    >
      <!-- Logo + 靶心小圆点（点击折叠/展开） -->
      <div class="app-brand">
        <a class="app-brand-link" @click.prevent="$router.push('/dashboard')">
          <span class="app-brand-logo">
            <svg viewBox="0 0 34 28" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M2 4 L12 10 L12 22 L2 16 Z" fill="#8c57ff"/>
              <path d="M12 10 L22 4 L22 16 L12 22 Z" fill="#7e4ee6"/>
              <path d="M22 4 L32 10 L32 22 L22 16 Z" fill="#a785fa"/>
              <path d="M12 10 L12 22 L17 25 L17 13 Z" fill="#6d40d8" opacity="0.7"/>
            </svg>
          </span>
          <transition name="fade">
            <span v-if="!siderCollapsed" class="app-brand-text">CradleRing</span>
          </transition>
        </a>
        <!-- 靶心小圆点：展开=双环（可折叠），折叠=单环（可展开） -->
        <button class="menu-toggle-dot" @click="appStore.toggleMenuCollapse()" :title="appStore.menuCollapse ? '展开侧栏' : '折叠侧栏'">
          <svg v-if="!appStore.menuCollapse" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 22C6.477 22 2 17.523 2 12S6.477 2 12 2s10 4.477 10 10s-4.477 10-10 10m0-2a8 8 0 1 0 0-16a8 8 0 0 0 0 16m0-3a5 5 0 1 1 0-10a5 5 0 0 1 0 10"/>
          </svg>
          <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 22C6.477 22 2 17.523 2 12S6.477 2 12 2s10 4.477 10 10s-4.477 10-10 10m0-2a8 8 0 1 0 0-16a8 8 0 0 0 0 16"/>
          </svg>
        </button>
      </div>

      <!-- 菜单（滚轮滑动区） -->
      <div class="menu-scroll">
        <app-menu :collapsed="siderCollapsed" />
      </div>
    </aside>

    <!-- 折叠遮罩（折叠态 hover 展开时，点击内容区收回） -->
    <div v-if="appStore.menuCollapse && appStore.menuHoverExpand" class="layout-overlay" @click="appStore.menuHoverExpand = false"></div>

    <!-- 主内容区 -->
    <a-layout class="layout-page" :style="{ marginLeft: pageMargin }">
      <!-- 顶部导航（detached 透明底） -->
      <a-layout-header class="layout-header">
        <app-navbar :sider-collapsed="siderCollapsed" @toggle-menu="appStore.toggleMenuCollapse()" />
      </a-layout-header>

      <!-- 页面内容 -->
      <a-layout-content class="layout-content">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </a-layout-content>

      <!-- 页脚 -->
      <footer class="layout-footer">
        <span>© 2026 CradleRing · 企业级 AI Agent 协作平台</span>
        <span class="footer-links">
          <a @click="$router.push('/logs')">日志</a>
          <a @click="$router.push('/audit')">审计</a>
          <a @click="$router.push('/settings')">设置</a>
        </span>
      </footer>
    </a-layout>
  </a-layout>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '@/stores/app';
import AppNavbar from './navbar/index.vue';
import AppMenu from './menu/index.vue';

const appStore = useAppStore();

// 实际生效的折叠态：折叠 && 未 hover 展开
const siderCollapsed = computed(() => appStore.menuCollapse && !appStore.menuHoverExpand);

// 内容区 margin：折叠 78px（Materialize collapsed 宽）/ 展开 260px
const pageMargin = computed(() => (appStore.menuCollapse ? '78px' : '260px'));

function onSiderEnter() {
  if (appStore.menuCollapse) appStore.menuHoverExpand = true;
}
function onSiderLeave() {
  if (appStore.menuCollapse) appStore.menuHoverExpand = false;
}
</script>

<style lang="less" scoped>
.layout-wrapper {
  height: 100vh;
  background-color: var(--color-bg-2);
  overflow: hidden;
}

/* 侧栏：白底 + 右侧轻阴影，折叠时宽度 78px，hover 浮出展开 260px */
.layout-sider {
  position: fixed;
  left: 0;
  top: 0;
  height: 100vh;
  width: 260px;
  background-color: var(--color-bg-1);
  box-shadow: 0 0.125rem 0.5rem 0 rgba(46, 38, 61, 0.08);
  z-index: 100;
  display: flex;
  flex-direction: column;
  transition: width 0.25s ease;
  overflow: hidden;

  &.collapsed {
    width: 78px;
  }
  /* hover 浮出展开：折叠态 hover 时浮出到 260px，盖在内容上方 */
  &.collapsed.hover-expand {
    width: 260px;
    box-shadow: 0 0.5rem 1.5rem 0 rgba(46, 38, 61, 0.25);
  }
}

/* Logo 区 */
.app-brand {
  height: var(--navbar-height);
  display: flex;
  align-items: center;
  padding: 0 18px;
  gap: 8px;
  flex-shrink: 0;
  position: relative;

  .app-brand-link {
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
    min-width: 0;
    flex: 1;
  }
  .app-brand-logo {
    width: 34px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    svg { width: 34px; height: 28px; }
  }
  .app-brand-text {
    font-size: 18px;
    font-weight: 700;
    color: var(--color-text-1);
    letter-spacing: 0.5px;
    white-space: nowrap;
  }
}

/* 靶心小圆点（Materialize 签名交互） */
.menu-toggle-dot {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--color-text-3);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: color 0.2s, background 0.2s;
  padding: 0;
  &:hover {
    color: var(--brand-primary);
    background: var(--color-bg-3);
  }
  /* 折叠态时固定在侧栏右缘 */
  .collapsed & {
    position: absolute;
    right: 12px;
  }
}

/* 菜单滚动区（滚轮滑动） */
.menu-scroll {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding-bottom: 16px;
  /* 细滚动条 */
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: transparent; border-radius: 2px; }
  &:hover::-webkit-scrollbar-thumb { background: var(--color-border-3); }
}

/* 折叠 hover 展开的遮罩 */
.layout-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 99;
}

/* 主内容 */
.layout-page {
  transition: margin-left 0.25s ease;
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.layout-header {
  height: var(--navbar-height);
  padding: 0;
  background-color: transparent;
  border: none;
  flex-shrink: 0;
  z-index: 50;
}

.layout-content {
  background-color: var(--color-bg-2);
  overflow-y: auto;
  flex: 1;
}

/* 页脚 */
.layout-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 24px;
  font-size: 12px;
  color: var(--color-text-4);
  flex-shrink: 0;
  .footer-links {
    display: flex;
    gap: 16px;
    a { cursor: pointer; color: var(--brand-primary); }
  }
}

/* fade 过渡 */
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

@media (max-width: 768px) {
  .layout-page {
    margin-left: 78px !important;
  }
  .layout-footer {
    flex-direction: column;
    gap: 6px;
  }
}
</style>
