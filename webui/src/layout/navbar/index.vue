<template>
  <div class="navbar">
    <div class="navbar-left">
      <a-button type="text" shape="circle" @click="appStore.menuCollapse = !appStore.menuCollapse">
        <icon-menu-fold v-if="!appStore.menuCollapse" />
        <icon-menu-unfold v-else />
      </a-button>
      <a-breadcrumb class="breadcrumb">
        <a-breadcrumb-item v-for="item in breadcrumbs" :key="item.path">
          {{ item.label }}
        </a-breadcrumb-item>
      </a-breadcrumb>
    </div>

    <div class="navbar-center">
      <a-input-search
        v-model="searchQuery"
        :style="{ width: '320px' }"
        placeholder="搜索会话、记忆、任务..."
        allow-clear
        @search="onSearch"
      />
    </div>

    <div class="navbar-right">
      <!-- 待审批通知 -->
      <a-badge :count="pendingApprovals" :offset="[-4, 4]" dot>
        <a-button type="text" shape="circle" @click="goApprovals">
          <icon-notification />
        </a-button>
      </a-badge>

      <!-- 主题切换 -->
      <a-tooltip :content="appStore.isDark ? '切换到浅色' : '切换到深色'">
        <a-button type="text" shape="circle" @click="appStore.toggleDark()">
          <icon-moon-fill v-if="!appStore.isDark" />
          <icon-sun-fill v-else />
        </a-button>
      </a-tooltip>

      <!-- 设置抽屉 -->
      <a-button type="text" shape="circle" @click="settingVisible = true">
        <icon-settings />
      </a-button>

      <!-- 用户下拉 -->
      <a-dropdown trigger="click">
        <div class="user-info">
          <a-avatar :size="30" :style="{ backgroundColor: '#165dff' }">
            {{ (userStore.user?.displayName || 'U').charAt(0) }}
          </a-avatar>
          <span class="username">{{ userStore.user?.displayName || '未登录' }}</span>
          <a-tag v-if="userStore.user" :color="roleColor" size="small">{{ roleLabel }}</a-tag>
        </div>
        <template #content>
          <a-doption @click="$router.push('/settings')">
            <template #icon><icon-user /></template>
            个人设置
          </a-doption>
          <a-doption @click="$router.push('/users')" v-if="userStore.isAdmin">
            <template #icon><icon-user-group /></template>
            用户管理
          </a-doption>
          <a-doption @click="onLogout">
            <template #icon><icon-export /></template>
            退出登录
          </a-doption>
        </template>
      </a-dropdown>
    </div>

    <global-setting-trigger v-model:visible="settingVisible" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useAppStore } from '@/stores/app';
import { useUserStore } from '@/stores/user';
import { rpc } from '@/api/rpc';
import GlobalSettingTrigger from './global-setting-trigger.vue';

const router = useRouter();
const route = useRoute();
const appStore = useAppStore();
const userStore = useUserStore();

const searchQuery = ref('');
const settingVisible = ref(false);
const pendingApprovals = ref(0);

const roleLabel = computed(() => {
  const map: Record<string, string> = {
    admin: '管理员', manager: '经理', supervisor: '主管',
    operator: '操作员', viewer: '访客',
  };
  return map[userStore.role] || userStore.role;
});

const roleColor = computed(() => {
  const map: Record<string, string> = {
    admin: 'red', manager: 'purple', supervisor: 'blue',
    operator: 'green', viewer: 'gray',
  };
  return map[userStore.role] || 'gray';
});

const breadcrumbs = computed(() => {
  return route.matched
    .filter((r) => r.meta?.label)
    .map((r) => ({ path: r.path, label: r.meta.label as string }));
});

function onSearch(v: string) {
  if (!v) return;
  router.push({ path: '/sessions', query: { q: v } });
}

function goApprovals() {
  router.push('/approvals/instances');
}

function onLogout() {
  userStore.logout();
  router.push('/login');
}

let unsub: (() => void) | null = null;
let timer: any = null;

async function refreshPending() {
  try {
    const res = await rpc.call<{ pending: number }>('approval.stats');
    pendingApprovals.value = res.pending || 0;
  } catch { /* ignore */ }
}

onMounted(() => {
  refreshPending();
  timer = setInterval(refreshPending, 15000);
  unsub = rpc.on('approval.instance.created', refreshPending);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
  if (unsub) unsub();
});
</script>

<style lang="less" scoped>
.navbar {
  height: var(--navbar-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  gap: 16px;
}

.navbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.navbar-center {
  flex: 1;
  display: flex;
  justify-content: center;
}

.navbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  justify-content: flex-end;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 10px;
  border-radius: 20px;
  cursor: pointer;
  transition: background 0.2s;
  &:hover {
    background-color: var(--color-bg-3);
  }
  .username {
    font-size: 14px;
    color: var(--color-text-1);
    font-weight: 500;
  }
}
</style>
