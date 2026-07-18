<template>
  <div class="page-container">
    <!-- 统计卡（Materialize：标题 + 大数字 + 百分比徽章 + 副标题 + 右上小彩色图标） -->
    <a-row :gutter="24">
      <a-col :xs="12" :lg="6" v-for="s in statCards" :key="s.title">
        <a-card class="stat-card">
          <div class="stat-title">{{ s.title }}</div>
          <div class="stat-main">
            <span class="stat-num">{{ s.value }}</span>
            <span class="stat-badge" :class="s.badgeClass">{{ s.badge }}</span>
          </div>
          <div class="stat-sub">{{ s.sub }}</div>
          <div class="stat-icon" :style="{ background: s.iconBg }">
            <component :is="s.icon" />
          </div>
        </a-card>
      </a-col>
    </a-row>

    <!-- Filters（Materialize：Export + Search + Add New User） -->
    <a-card class="mt-24">
      <div class="filter-bar">
        <a-dropdown trigger="click">
          <a-button>
            <template #icon><icon-download /></template>
            导出 <icon-down />
          </a-button>
          <template #content>
            <a-doption @click="exportUsers('json')">导出 JSON</a-doption>
            <a-doption @click="exportUsers('csv')">导出 CSV</a-doption>
          </template>
        </a-dropdown>
        <div class="filter-right">
          <a-input v-model="searchKey" placeholder="搜索用户..." allow-clear style="width: 240px">
            <template #prefix><icon-search /></template>
          </a-input>
          <a-button type="primary" @click="openCreate">
            <template #icon><icon-plus /></template>
            新建用户
          </a-button>
        </div>
      </div>

      <a-table class="mt-16" :data="filteredUsers" :loading="loading" :pagination="{ pageSize: 15, showTotal: true }" row-key="id" :bordered="false">
        <template #columns>
          <a-table-column title="用户" :width="220">
            <template #cell="{ record }">
              <a-space>
                <a-avatar :size="34" :style="{ backgroundColor: roleColor(record.role) }">{{ (record.displayName || record.username).charAt(0).toUpperCase() }}</a-avatar>
                <div>
                  <div class="cell-name">{{ record.displayName }}</div>
                  <div class="muted">@{{ record.username }}</div>
                </div>
              </a-space>
            </template>
          </a-table-column>
          <a-table-column title="角色" :width="110">
            <template #cell="{ record }">
              <a-tag :color="roleColor(record.role)">{{ roleLabel(record.role) }}</a-tag>
            </template>
          </a-table-column>
          <a-table-column title="邮箱" data-index="email" :width="200" ellipsis tooltip>
            <template #cell="{ record }">{{ record.email || '-' }}</template>
          </a-table-column>
          <a-table-column title="权限" :width="200">
            <template #cell="{ record }">
              <a-space wrap>
                <a-tag v-for="s in (record.scopes || []).slice(0, 3)" :key="s" size="small">{{ s }}</a-tag>
                <a-tag v-if="(record.scopes || []).length > 3" size="small">+{{ record.scopes.length - 3 }}</a-tag>
              </a-space>
            </template>
          </a-table-column>
          <a-table-column title="状态" :width="90">
            <template #cell="{ record }">
              <a-badge :status="record.enabled ? 'success' : 'default'" :text="record.enabled ? '正常' : '禁用'" />
            </template>
          </a-table-column>
          <a-table-column title="最后登录" :width="140">
            <template #cell="{ record }">{{ record.lastLogin ? dayjs(record.lastLogin).format('MM-DD HH:mm') : '从未' }}</template>
          </a-table-column>
          <a-table-column title="操作" :width="150" fixed="right">
            <template #cell="{ record }">
              <a-space>
                <a-button size="small" type="text" @click="openEdit(record)"><icon-edit /></a-button>
                <a-popconfirm content="确认删除该用户？" @ok="onDelete(record.id)" :disabled="record.username === 'admin'">
                  <a-button size="small" type="text" status="danger" :disabled="record.username === 'admin'"><icon-delete /></a-button>
                </a-popconfirm>
              </a-space>
            </template>
          </a-table-column>
        </template>
      </a-table>
    </a-card>

    <!-- 角色卡网格（Materialize app-access-roles） -->
    <div class="roles-header mt-24">
      <h4 class="roles-title">角色列表</h4>
      <p class="roles-sub">角色提供预定义的菜单和功能访问权限，管理员可根据分配的角色访问所需内容</p>
    </div>
    <a-row :gutter="24" class="mt-8">
      <a-col :xs="24" :md="12" :lg="8" v-for="r in roleCards" :key="r.name">
        <a-card class="role-card">
          <div class="role-users">
            <span>共 {{ r.count }} 个用户</span>
            <div class="avatar-stack">
              <a-avatar v-for="(u, i) in r.users.slice(0, 3)" :key="u.id" :size="28" :style="{ backgroundColor: roleColor(u.role), zIndex: 3 - i, marginLeft: i > 0 ? '-8px' : '0' }">
                {{ (u.displayName || u.username).charAt(0).toUpperCase() }}
              </a-avatar>
              <a-avatar v-if="r.count > 3" :size="28" :style="{ backgroundColor: '#e8e0fe', color: '#8c57ff', marginLeft: '-8px', zIndex: 0 }">+{{ r.count - 3 }}</a-avatar>
            </div>
          </div>
          <div class="role-name-row">
            <div>
              <div class="role-name">
                {{ r.label }}
                <a-tag v-if="r.builtin" size="small" color="arcoblue">预置</a-tag>
              </div>
              <a-link class="role-edit" @click="openEditRole(r)">编辑角色</a-link>
            </div>
            <a-tooltip content="复制角色">
              <button class="role-copy" @click="duplicateRole(r)"><icon-copy /></button>
            </a-tooltip>
          </div>
        </a-card>
      </a-col>
      <!-- Add New Role 卡 -->
      <a-col :xs="24" :md="12" :lg="8">
        <a-card class="role-card add-role-card" @click="openCreateRole">
          <div class="add-role-inner">
            <div class="add-role-icon"><icon-plus /></div>
            <a-button type="primary" class="mt-16">新建角色</a-button>
            <p class="add-role-desc">如果角色不存在，请添加新角色</p>
          </div>
        </a-card>
      </a-col>
    </a-row>

    <!-- 用户编辑抽屉 -->
    <a-drawer :visible="visible" :width="520" @cancel="visible = false" @ok="onSave" :ok-loading="saving">
      <template #title>{{ editing.id ? '编辑用户' : '新建用户' }}</template>
      <a-form :model="editing" layout="vertical">
        <a-form-item label="用户名" required>
          <a-input v-model="editing.username" :disabled="!!editing.id" placeholder="登录用户名" />
        </a-form-item>
        <a-form-item label="显示名称">
          <a-input v-model="editing.displayName" placeholder="如：张三" />
        </a-form-item>
        <a-form-item label="邮箱">
          <a-input v-model="editing.email" placeholder="user@example.com" />
        </a-form-item>
        <a-form-item :label="editing.id ? '重置密码（留空不修改）' : '密码'" :required="!editing.id">
          <a-input-password v-model="editing.password" placeholder="••••••••" />
        </a-form-item>
        <a-row :gutter="8">
          <a-col :span="12">
            <a-form-item label="角色">
              <a-select v-model="editing.role" placeholder="选择角色">
                <a-option v-for="r in allRoles" :key="r.name" :value="r.name">{{ r.label }} ({{ r.name }})</a-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="关联 Agent">
              <a-input v-model="editing.agentId" placeholder="main" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-form-item label="权限范围（细粒度，留空使用角色默认）">
          <a-select v-model="editing.scopes" multiple placeholder="选择权限">
            <a-option v-for="s in availableScopes" :key="s" :value="s">{{ s }}</a-option>
          </a-select>
          <div class="hint">当前角色默认权限：<a-tag v-for="s in defaultScopes" :key="s" size="small">{{ s }}</a-tag></div>
        </a-form-item>
        <a-row :gutter="8">
          <a-col :span="12">
            <a-form-item label="启用"><a-switch v-model="editing.enabled" /></a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="启用审批">
              <a-switch v-model="editing.approvalEnabled" />
              <div class="hint">关闭后该用户执行危险命令时无需审批</div>
            </a-form-item>
          </a-col>
        </a-row>
      </a-form>
    </a-drawer>

    <!-- 角色编辑对话框 -->
    <a-modal :visible="roleVisible" :title="editingRole.builtin ? '编辑预置角色' : (editingRole.isNew ? '新建角色' : '编辑角色')"
      @cancel="roleVisible = false" @ok="saveRole" :ok-loading="savingRole" :width="560">
      <a-form :model="editingRole" layout="vertical">
        <a-form-item label="角色 ID（英文，如 devops）" :required="!editingRole.builtin">
          <a-input v-model="editingRole.name" :disabled="editingRole.builtin" placeholder="custom_devops" />
        </a-form-item>
        <a-form-item label="显示名称" required>
          <a-input v-model="editingRole.label" placeholder="如：运维开发" />
        </a-form-item>
        <a-form-item label="描述">
          <a-input v-model="editingRole.description" placeholder="角色职责描述" />
        </a-form-item>
        <a-form-item label="颜色">
          <a-radio-group type="button" v-model="editingRole.color">
            <a-radio v-for="c in roleColors" :key="c" :value="c">{{ c }}</a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item label="权限范围">
          <a-select v-model="editingRole.scopes" multiple placeholder="选择权限">
            <a-option v-for="s in availableScopes" :key="s" :value="s">{{ s }}</a-option>
          </a-select>
        </a-form-item>
        <a-form-item v-if="!editingRole.builtin && !editingRole.isNew">
          <a-popconfirm content="确认删除该角色？" @ok="onDeleteRole(editingRole.name); roleVisible = false">
            <a-button status="danger" long>删除角色</a-button>
          </a-popconfirm>
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, markRaw } from 'vue';
import dayjs from 'dayjs';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import {
  IconPlus, IconSearch, IconDownload, IconDown, IconEdit, IconDelete,
  IconCopy, IconUserGroup, IconUser, IconCheckCircle, IconClockCircle,
} from '@arco-design/web-vue/es/icon';

const loading = ref(false);
const users = ref<any[]>([]);
const allRoles = ref<any[]>([]);
const searchKey = ref('');
const visible = ref(false);
const saving = ref(false);
const roleVisible = ref(false);
const savingRole = ref(false);

const availableScopes = [
  'chat', 'sessions.read', 'sessions.write', 'sessions.*',
  'memory.read', 'memory.write', 'memory.*',
  'tools.exec', 'tools.web_search', 'tools.*',
  'approval.approve', 'approval.advanced', 'approval.*',
  'channels.read', 'channels.write', 'cron.read', 'cron.write',
  'config.read', 'config.write', 'users.read', 'users.write',
  'workflows.read', 'workflows.write', 'logs.read', 'admin',
];

const roleColors = ['#ff4c51', '#7340e0', '#8c57ff', '#56ca00', '#ffb400', '#16b1ff', '#6d6777'];

const editing = reactive<any>({
  id: '', username: '', displayName: '', email: '', password: '',
  role: 'operator', agentId: 'main', enabled: true, approvalEnabled: true, scopes: [],
});

const editingRole = reactive<any>({
  name: '', label: '', description: '', color: '#16b1ff', scopes: [], builtin: false, isNew: false,
});

const defaultScopes = ref<string[]>([]);

const filteredUsers = computed(() =>
  users.value.filter((u) => !searchKey.value || u.username.toLowerCase().includes(searchKey.value.toLowerCase()) || (u.displayName || '').toLowerCase().includes(searchKey.value.toLowerCase())),
);

// 统计卡（Materialize）
const statCards = computed(() => {
  const total = users.value.length;
  const enabled = users.value.filter((u) => u.enabled).length;
  const adminCount = users.value.filter((u) => u.role === 'admin').length;
  const recentLogin = users.value.filter((u) => u.lastLogin && Date.now() - u.lastLogin < 7 * 86400000).length;
  return [
    { title: '总用户', value: total, badge: `${enabled} 启用`, badgeClass: 'up', sub: '全部账号', icon: markRaw(IconUserGroup), iconBg: 'rgba(140, 87, 255, 0.15)' },
    { title: '管理员', value: adminCount, badge: '核心', badgeClass: 'up', sub: '最高权限', icon: markRaw(IconUser), iconBg: 'rgba(255, 76, 81, 0.15)' },
    { title: '活跃用户', value: recentLogin, badge: '7 天内', badgeClass: 'up', sub: '最近登录', icon: markRaw(IconCheckCircle), iconBg: 'rgba(86, 202, 0, 0.15)' },
    { title: '待审批用户', value: users.value.filter((u) => u.approvalEnabled).length, badge: '审批流', badgeClass: 'down', sub: '启用审批', icon: markRaw(IconClockCircle), iconBg: 'rgba(255, 180, 0, 0.15)' },
  ];
});

// 角色卡（Materialize app-access-roles）
const roleCards = computed(() =>
  allRoles.value.map((r) => {
    const roleUsers = users.value.filter((u) => u.role === r.name);
    return { ...r, count: roleUsers.length, users: roleUsers };
  }),
);

function roleColor(r: string) { return allRoles.value.find((x) => x.name === r)?.color || '#6d6777'; }
function roleLabel(r: string) { return allRoles.value.find((x) => x.name === r)?.label || r; }

async function load() {
  loading.value = true;
  try {
    const [usersRes, rolesRes] = await Promise.all([
      rpc.call<{ users: any[] }>('users.list'),
      rpc.call<{ roles: any[] }>('roles.list'),
    ]);
    users.value = usersRes.users || [];
    allRoles.value = rolesRes.roles || [];
  } catch (e: any) { Message.error(e.message); }
  finally { loading.value = false; }
}

function openCreate() {
  Object.assign(editing, { id: '', username: '', displayName: '', email: '', password: '', role: 'operator', agentId: 'main', enabled: true, approvalEnabled: true, scopes: [] });
  loadDefaultScopes('operator');
  visible.value = true;
}

function openEdit(u: any) {
  Object.assign(editing, { ...u, password: '', scopes: u.scopes || [] });
  loadDefaultScopes(u.role);
  visible.value = true;
}

async function loadDefaultScopes(role: string) {
  try {
    const res = await rpc.call<{ scopes: string[] }>('roles.scopes', { role });
    defaultScopes.value = res.scopes || [];
  } catch { /* ignore */ }
}

watch(() => editing.role, (r) => loadDefaultScopes(r));

async function onSave() {
  if (!editing.username.trim()) { Message.warning('请输入用户名'); return; }
  if (!editing.id && !editing.password) { Message.warning('请输入密码'); return; }
  saving.value = true;
  try {
    const payload = {
      ...(editing.id ? { id: editing.id } : {}),
      username: editing.username, displayName: editing.displayName || editing.username,
      email: editing.email || undefined, password: editing.password || undefined,
      role: editing.role, agentId: editing.agentId,
      enabled: editing.enabled, approvalEnabled: editing.approvalEnabled,
      scopes: editing.scopes,
    };
    if (editing.id) await rpc.call('users.update', payload);
    else await rpc.call('users.create', payload);
    Message.success(editing.id ? '已更新' : '已创建');
    visible.value = false;
    await load();
  } catch (e: any) { Message.error(e.message); }
  finally { saving.value = false; }
}

async function onDelete(id: string) {
  try { await rpc.call('users.delete', { id }); Message.success('已删除'); await load(); }
  catch (e: any) { Message.error(e.message); }
}

function exportUsers(format: string) {
  const data = filteredUsers.value.map((u) => ({
    username: u.username, displayName: u.displayName, role: u.role,
    email: u.email, enabled: u.enabled, lastLogin: u.lastLogin,
  }));
  let content: string;
  let mime: string;
  let ext: string;
  if (format === 'csv') {
    const header = 'username,displayName,role,email,enabled,lastLogin';
    const rows = data.map((d) => `"${d.username}","${d.displayName}","${d.role}","${d.email || ''}","${d.enabled}","${d.lastLogin || ''}"`);
    content = [header, ...rows].join('\n');
    mime = 'text/csv';
    ext = 'csv';
  } else {
    content = JSON.stringify(data, null, 2);
    mime = 'application/json';
    ext = 'json';
  }
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `users-${dayjs().format('YYYYMMDD-HHmm')}.${ext}`;
  a.click();
  URL.revokeObjectURL(url);
  Message.success(`已导出 ${data.length} 个用户`);
}

// ---------- 角色管理 ----------
function openCreateRole() {
  Object.assign(editingRole, { name: '', label: '', description: '', color: '#16b1ff', scopes: [], builtin: false, isNew: true });
  roleVisible.value = true;
}

function openEditRole(r: any) {
  Object.assign(editingRole, { ...r, isNew: false });
  roleVisible.value = true;
}

function duplicateRole(r: any) {
  Object.assign(editingRole, {
    name: `${r.name}_copy`, label: `${r.label} 副本`, description: r.description,
    color: r.color, scopes: [...(r.scopes || [])], builtin: false, isNew: true,
  });
  roleVisible.value = true;
}

async function saveRole() {
  if (!editingRole.name.trim() || !editingRole.label.trim()) { Message.warning('请输入角色 ID 和名称'); return; }
  savingRole.value = true;
  try {
    const payload = {
      name: editingRole.name, label: editingRole.label,
      description: editingRole.description, color: editingRole.color,
      scopes: editingRole.scopes,
    };
    if (editingRole.isNew) await rpc.call('roles.create', payload);
    else await rpc.call('roles.update', payload);
    Message.success(editingRole.isNew ? '已创建' : '已更新');
    roleVisible.value = false;
    await load();
  } catch (e: any) { Message.error(e.message); }
  finally { savingRole.value = false; }
}

async function onDeleteRole(name: string) {
  try { await rpc.call('roles.delete', { name }); Message.success('已删除'); await load(); }
  catch (e: any) { Message.error(e.message); }
}

onMounted(load);
</script>

<style lang="less" scoped>
/* 统计卡（Materialize） */
.stat-card {
  position: relative;
  overflow: hidden;
  .stat-title {
    font-size: 13px;
    color: var(--color-text-3);
  }
  .stat-main {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-top: 4px;
  }
  .stat-num {
    font-size: 24px;
    font-weight: 700;
    color: var(--color-text-1);
  }
  .stat-badge {
    font-size: 11px;
    font-weight: 600;
    &.up { color: var(--brand-success); }
    &.down { color: var(--brand-danger); }
  }
  .stat-sub {
    font-size: 12px;
    color: var(--color-text-4);
    margin-top: 4px;
  }
  .stat-icon {
    position: absolute;
    top: 16px;
    right: 16px;
    width: 38px;
    height: 38px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 19px;
    color: var(--brand-primary);
  }
}

/* Filters */
.filter-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}
.filter-right {
  display: flex;
  gap: 12px;
  align-items: center;
}

.cell-name {
  font-weight: 500;
  color: var(--color-text-1);
}
.muted { color: var(--color-text-3); font-size: 12px; }
.hint { font-size: 12px; color: var(--color-text-3); margin-top: 4px; }

/* 角色卡（Materialize app-access-roles） */
.roles-header {
  .roles-title {
    font-size: 17px;
    font-weight: 600;
    color: var(--color-text-1);
    margin: 0;
  }
  .roles-sub {
    font-size: 13px;
    color: var(--color-text-3);
    margin: 4px 0 0;
  }
}

.role-card {
  height: 100%;
  .role-users {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
    color: var(--color-text-3);
  }
  .avatar-stack {
    display: flex;
  }
  .role-name-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-top: 16px;
  }
  .role-name {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-1);
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .role-edit {
    font-size: 13px;
    margin-top: 4px;
    display: inline-block;
  }
  .role-copy {
    border: none;
    background: transparent;
    color: var(--color-text-4);
    cursor: pointer;
    font-size: 16px;
    padding: 6px;
    border-radius: 6px;
    display: flex;
    &:hover { color: var(--brand-primary); background: var(--color-bg-3); }
  }
}

.add-role-card {
  cursor: pointer;
  border: 1.5px dashed var(--color-border-2);
  box-shadow: none !important;
  &:hover { border-color: var(--brand-primary); }
  .add-role-inner {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 120px;
    text-align: center;
  }
  .add-role-icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: rgba(140, 87, 255, 0.12);
    color: var(--brand-primary);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 22px;
  }
  .add-role-desc {
    font-size: 12px;
    color: var(--color-text-4);
    margin: 8px 0 0;
  }
}
</style>
