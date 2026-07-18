<template>
  <div class="layout-navbar">
    <!-- 左侧：菜单切换 + 面包屑 -->
    <div class="navbar-left">
      <a-tooltip :content="appStore.menuCollapse ? '展开侧栏' : '折叠侧栏'">
        <button class="nav-icon-btn" @click="$emit('toggle-menu')">
          <icon-menu-fold v-if="!appStore.menuCollapse" />
          <icon-menu-unfold v-else />
        </button>
      </a-tooltip>
      <a-breadcrumb class="navbar-breadcrumb">
        <a-breadcrumb-item v-for="item in breadcrumbs" :key="item.path">
          {{ item.label }}
        </a-breadcrumb-item>
      </a-breadcrumb>
    </div>

    <!-- 中间：搜索 -->
    <div class="navbar-center">
      <a-input-search
        v-model="searchQuery"
        :style="{ width: 'min(420px, 40vw)' }"
        placeholder="搜索会话、记忆、任务..."
        allow-clear
        @search="onSearch"
      />
    </div>

    <!-- 右侧：语言 / 主题 / 快捷入口 / 通知 / 用户 -->
    <div class="navbar-right">
      <!-- 语言切换 -->
      <a-dropdown trigger="click">
        <button class="nav-icon-btn"><icon-translate /></button>
        <template #content>
          <a-doption v-for="l in locales" :key="l.value" @click="appStore.setLocale(l.value)">
            <div class="dropdown-check-item">
              <icon-check v-if="appStore.locale === l.value" class="check-icon" />
              <span v-else class="check-placeholder"></span>
              {{ l.label }}
            </div>
          </a-doption>
        </template>
      </a-dropdown>

      <!-- 主题切换（浅色/深色/跟随系统） -->
      <a-dropdown trigger="click">
        <button class="nav-icon-btn">
          <icon-sun-fill v-if="appStore.themeMode === 'light'" />
          <icon-moon-fill v-else-if="appStore.themeMode === 'dark'" />
          <icon-computer v-else />
        </button>
        <template #content>
          <a-doption v-for="t in themeModes" :key="t.value" @click="appStore.setThemeMode(t.value as any)">
            <div class="dropdown-check-item">
              <icon-check v-if="appStore.themeMode === t.value" class="check-icon" />
              <span v-else class="check-placeholder"></span>
              <component :is="t.icon" class="mode-icon" />
              {{ t.label }}
            </div>
          </a-doption>
        </template>
      </a-dropdown>

      <!-- 快捷入口 -->
      <a-dropdown trigger="click">
        <button class="nav-icon-btn"><icon-star /></button>
        <template #content>
          <div class="shortcuts-panel">
            <div class="shortcuts-grid">
              <div v-for="s in shortcuts" :key="s.path" class="shortcut-item" @click="goShortcut(s.path)">
                <div class="shortcut-icon" :style="{ background: s.bg }">
                  <component :is="s.icon" />
                </div>
                <div class="shortcut-label">{{ s.label }}</div>
                <div class="shortcut-desc">{{ s.desc }}</div>
              </div>
            </div>
          </div>
        </template>
      </a-dropdown>

      <!-- 通知（真实数据：待审批 + 系统事件） -->
      <a-dropdown trigger="click" :popup-max-height="480">
        <button class="nav-icon-btn">
          <a-badge :count="unreadCount" :max-count="99" :dot="unreadCount > 0" :dot-style="{ background: '#ff4c51' }">
            <icon-notification />
          </a-badge>
        </button>
        <template #content>
          <div class="notif-panel">
            <!-- 头部 -->
            <div class="notif-header">
              <span class="notif-title">通知</span>
              <a-tag v-if="unreadCount > 0" color="purple" size="small">{{ unreadCount }} 条新通知</a-tag>
              <a-tooltip content="全部标为已读">
                <button class="notif-read-all" @click.stop="markAllRead"><icon-email /></button>
              </a-tooltip>
            </div>
            <!-- 列表 -->
            <div class="notif-list">
              <a-empty v-if="!notifications.length" description="暂无通知" :image-style="{ width: '60px' }" />
              <div v-for="n in notifications" :key="n.id" class="notif-item" :class="{ unread: !n.read }" @click="openNotification(n)">
                <div class="notif-avatar" :style="{ background: n.bg }">
                  <component :is="n.icon" />
                </div>
                <div class="notif-body">
                  <div class="notif-item-title">{{ n.title }}</div>
                  <div class="notif-desc">{{ n.desc }}</div>
                  <div class="notif-time">{{ n.time }}</div>
                </div>
                <div class="notif-actions">
                  <span v-if="!n.read" class="unread-dot"></span>
                  <button class="notif-close" @click.stop="dismissNotification(n.id)"><icon-close /></button>
                </div>
              </div>
            </div>
            <!-- 底部 -->
            <div v-if="notifications.length" class="notif-footer" @click="$router.push('/approvals/instances')">
              查看全部
            </div>
          </div>
        </template>
      </a-dropdown>

      <!-- 用户头像下拉 -->
      <a-dropdown trigger="click">
        <div class="navbar-user">
          <div class="user-avatar">
            {{ (userStore.user?.displayName || userStore.user?.username || 'U').charAt(0).toUpperCase() }}
            <span class="avatar-status"></span>
          </div>
        </div>
        <template #content>
          <div class="user-panel">
            <!-- 用户信息卡 -->
            <div class="user-card" @click="$router.push('/settings')">
              <div class="user-card-avatar">
                {{ (userStore.user?.displayName || userStore.user?.username || 'U').charAt(0).toUpperCase() }}
                <span class="avatar-status"></span>
              </div>
              <div class="user-card-info">
                <div class="user-card-name">{{ userStore.user?.displayName || userStore.user?.username || '未登录' }}</div>
                <div class="user-card-role">{{ roleLabel }}</div>
              </div>
            </div>
            <a-divider class="my-0" />
            <!-- 菜单项 -->
            <div class="user-menu">
              <div class="user-menu-item" @click="$router.push('/settings')">
                <icon-user /> 我的资料
              </div>
              <div class="user-menu-item" @click="$router.push('/config')">
                <icon-settings /> 配置管理
              </div>
              <div v-if="userStore.isAdmin" class="user-menu-item" @click="$router.push('/users')">
                <icon-user-group /> 用户管理
              </div>
              <div class="user-menu-item" @click="$router.push('/logs')">
                <icon-file /> 日志
              </div>
            </div>
            <a-divider class="my-0" />
            <!-- 退出登录 -->
            <div class="user-logout" @click="onLogout">
              <icon-export /> 退出登录
            </div>
          </div>
        </template>
      </a-dropdown>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, markRaw } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useAppStore } from '@/stores/app';
import { useUserStore } from '@/stores/user';
import { rpc } from '@/api/rpc';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/zh-cn';
import {
  IconMenuFold, IconMenuUnfold, IconTranslate, IconCheck, IconSunFill, IconMoonFill,
  IconComputer, IconStar, IconNotification, IconEmail, IconClose, IconUser,
  IconSettings, IconUserGroup, IconFile, IconExport, IconDashboard, IconMessage,
  IconBookmark, IconSafe, IconCheckCircle, IconClockCircle, IconInfoCircle,
} from '@arco-design/web-vue/es/icon';

dayjs.extend(relativeTime);
dayjs.locale('zh-cn');

defineEmits(['toggle-menu']);

const router = useRouter();
const route = useRoute();
const appStore = useAppStore();
const userStore = useUserStore();

const searchQuery = ref('');

// 语言选项
const locales = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'en-US', label: 'English' },
];

// 主题模式
const themeModes = [
  { value: 'light', label: '浅色', icon: markRaw(IconSunFill) },
  { value: 'dark', label: '深色', icon: markRaw(IconMoonFill) },
  { value: 'system', label: '跟随系统', icon: markRaw(IconComputer) },
];

// 快捷入口
const shortcuts = [
  { path: '/chat', label: '对话', desc: 'AI 助手', icon: markRaw(IconMessage), bg: 'linear-gradient(135deg, #8c57ff, #a785fa)' },
  { path: '/memory', label: '记忆库', desc: '长期记忆', icon: markRaw(IconBookmark), bg: 'linear-gradient(135deg, #16b1ff, #56d0ff)' },
  { path: '/approvals/instances', label: '审批', desc: '待办事项', icon: markRaw(IconCheckCircle), bg: 'linear-gradient(135deg, #56ca00, #82e040)' },
  { path: '/ops-dashboard', label: '运维大屏', desc: '节点监控', icon: markRaw(IconDashboard), bg: 'linear-gradient(135deg, #ffb400, #ffd040)' },
  { path: '/audit', label: '运维审计', desc: '安全合规', icon: markRaw(IconSafe), bg: 'linear-gradient(135deg, #ff4c51, #ff8083)' },
  { path: '/config', label: '配置', desc: '系统设置', icon: markRaw(IconSettings), bg: 'linear-gradient(135deg, #6d6777, #9a94a5)' },
];

function goShortcut(path: string) {
  router.push(path);
}

// ---------- 通知（真实数据） ----------
interface Notification {
  id: string;
  title: string;
  desc: string;
  time: string;
  icon: any;
  bg: string;
  read: boolean;
  link?: string;
}

const notifications = ref<Notification[]>([]);
const readIds = ref<Set<string>>(new Set(JSON.parse(localStorage.getItem('cradle-notif-read') || '[]')));

const unreadCount = computed(() => notifications.value.filter((n) => !n.read).length);

const roleLabel = computed(() => {
  const map: Record<string, string> = {
    admin: '管理员', manager: '经理', supervisor: '主管',
    operator: '操作员', viewer: '访客',
  };
  return map[userStore.role] || userStore.role;
});

const breadcrumbs = computed(() =>
  route.matched.filter((r) => r.meta?.label).map((r) => ({ path: r.path, label: r.meta.label as string }))
);

function onSearch(v: string) {
  if (v) router.push({ path: '/sessions', query: { q: v } });
}

async function loadNotifications() {
  const list: Notification[] = [];
  // 待审批（最重要）
  try {
    const res = await rpc.call<any>('approval.instances.list', { status: 'pending' });
    const instances = (res.instances || []).slice(0, 5);
    for (const inst of instances) {
      list.push({
        id: `approval-${inst.id}`,
        title: `待审批：${inst.title || '命令执行'}`,
        desc: `${inst.requestedUsername || '用户'} · ${(inst.command || '').slice(0, 40)}`,
        time: dayjs(inst.createdAt || Date.now()).fromNow(),
        icon: markRaw(IconClockCircle),
        bg: 'linear-gradient(135deg, #ffb400, #ffd040)',
        read: readIds.value.has(`approval-${inst.id}`),
        link: '/approvals/instances',
      });
    }
  } catch { /* ignore */ }
  // 系统事件
  try {
    const res = await rpc.call<any>('gateway.events', { limit: 5 });
    const events = (res.events || []).slice(-5).reverse();
    for (const ev of events) {
      const id = `event-${ev.ts}-${ev.event}`;
      list.push({
        id,
        title: ev.event || '系统事件',
        desc: String(ev.payload?.message || ev.payload?.error || '').slice(0, 50) || '系统状态变更',
        time: dayjs(ev.ts || Date.now()).fromNow(),
        icon: markRaw(IconInfoCircle),
        bg: 'linear-gradient(135deg, #16b1ff, #56d0ff)',
        read: readIds.value.has(id),
        link: '/logs',
      });
    }
  } catch { /* ignore */ }
  notifications.value = list;
}

function openNotification(n: Notification) {
  n.read = true;
  readIds.value.add(n.id);
  persistRead();
  if (n.link) router.push(n.link);
}

function dismissNotification(id: string) {
  const n = notifications.value.find((x) => x.id === id);
  if (n) {
    n.read = true;
    readIds.value.add(id);
    persistRead();
  }
  notifications.value = notifications.value.filter((x) => x.id !== id);
}

function markAllRead() {
  notifications.value.forEach((n) => {
    n.read = true;
    readIds.value.add(n.id);
  });
  persistRead();
}

function persistRead() {
  localStorage.setItem('cradle-notif-read', JSON.stringify(Array.from(readIds.value)));
}

function onLogout() {
  userStore.logout();
  router.push('/login');
}

let timer: any = null;
onMounted(() => {
  loadNotifications();
  timer = setInterval(loadNotifications, 30000);
});
onUnmounted(() => { if (timer) clearInterval(timer); });
</script>

<style lang="less" scoped>
.layout-navbar {
  height: var(--navbar-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  gap: 16px;
}

.navbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.navbar-breadcrumb {
  font-size: 14px;
}

.navbar-center {
  flex: 1;
  display: flex;
  justify-content: center;
}

.navbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

/* 图标按钮（Materialize 圆形 icon button） */
.nav-icon-btn {
  width: 38px;
  height: 38px;
  border: none;
  background: transparent;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-2);
  font-size: 19px;
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
  padding: 0;
  &:hover {
    background: var(--color-bg-3);
    color: var(--brand-primary);
  }
}

/* 下拉勾选标记 */
.dropdown-check-item {
  display: flex;
  align-items: center;
  gap: 8px;
  .check-icon { color: var(--brand-primary); font-size: 16px; }
  .check-placeholder { width: 16px; }
  .mode-icon { font-size: 15px; }
}

/* 快捷入口面板 */
.shortcuts-panel {
  padding: 12px;
  width: 300px;
}
.shortcuts-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 4px;
}
.shortcut-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 8px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s;
  text-align: center;
  &:hover { background: var(--color-bg-3); }
}
.shortcut-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 19px;
  margin-bottom: 8px;
  box-shadow: var(--shadow-xs);
}
.shortcut-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-1);
}
.shortcut-desc {
  font-size: 11px;
  color: var(--color-text-4);
  margin-top: 2px;
}

/* 通知面板 */
.notif-panel {
  width: 360px;
  max-width: calc(100vw - 32px);
}
.notif-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border-1);
  .notif-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text-1);
    flex: 1;
  }
}
.notif-read-all {
  border: none;
  background: transparent;
  color: var(--color-text-3);
  cursor: pointer;
  font-size: 16px;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  &:hover { color: var(--brand-primary); background: var(--color-bg-3); }
}
.notif-list {
  max-height: 380px;
  overflow-y: auto;
}
.notif-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.15s;
  border-bottom: 1px solid var(--color-border-1);
  &:hover { background: var(--color-bg-3); }
  &.unread { background: rgba(140, 87, 255, 0.04); }
}
.notif-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  flex-shrink: 0;
}
.notif-body {
  flex: 1;
  min-width: 0;
}
.notif-item-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-1);
  line-height: 1.4;
}
.notif-desc {
  font-size: 12px;
  color: var(--color-text-3);
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.notif-time {
  font-size: 11px;
  color: var(--color-text-4);
  margin-top: 4px;
}
.notif-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}
.unread-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--brand-primary);
}
.notif-close {
  border: none;
  background: transparent;
  color: var(--color-text-4);
  cursor: pointer;
  font-size: 14px;
  padding: 2px;
  border-radius: 4px;
  display: flex;
  opacity: 0;
  .notif-item:hover & { opacity: 1; }
  &:hover { color: var(--brand-danger); background: var(--color-bg-4); }
}
.notif-footer {
  padding: 10px;
  text-align: center;
  font-size: 13px;
  color: var(--brand-primary);
  cursor: pointer;
  border-top: 1px solid var(--color-border-1);
  &:hover { background: var(--color-bg-3); }
}

/* 用户面板 */
.navbar-user {
  margin-left: 8px;
  cursor: pointer;
}
.user-avatar {
  position: relative;
  width: 38px;
  height: 38px;
  border-radius: 50%;
  background: var(--brand-primary);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 15px;
  box-shadow: var(--shadow-xs);
}
.avatar-status {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--brand-success);
  border: 2px solid var(--color-bg-1);
}

.user-panel {
  width: 260px;
  padding: 8px 0;
}
.user-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  cursor: pointer;
  &:hover { background: var(--color-bg-3); }
}
.user-card-avatar {
  position: relative;
  width: 42px;
  height: 42px;
  border-radius: 50%;
  background: var(--brand-primary);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 17px;
  flex-shrink: 0;
}
.user-card-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-1);
}
.user-card-role {
  font-size: 12px;
  color: var(--color-text-3);
  margin-top: 2px;
}
.user-menu {
  padding: 6px 0;
}
.user-menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  font-size: 13px;
  color: var(--color-text-2);
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
  &:hover { background: var(--color-bg-3); color: var(--brand-primary); }
  svg { font-size: 16px; }
}
.user-logout {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin: 8px 12px 4px;
  padding: 9px;
  border-radius: 6px;
  background: var(--brand-danger);
  color: #fff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
  &:hover { background: #e04348; }
}

.my-0 { margin: 0 !important; }
</style>
