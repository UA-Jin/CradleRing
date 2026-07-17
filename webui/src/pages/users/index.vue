<template>
  <div class="page-container">
    <a-page-header title="用户管理" subtitle="多账号系统 · 角色权限 · 自定义角色" :show-back="false">
      <template #extra>
        <a-space>
          <a-input v-model="searchKey" placeholder="搜索用户..." allow-clear style="width: 200px" />
          <a-button @click="showRoleManager = true"><template #icon><icon-settings /></template>管理角色</a-button>
          <a-button type="primary" @click="openCreate"><template #icon><icon-plus /></template>新建用户</a-button>
        </a-space>
      </template>
    </a-page-header>

    <!-- 角色统计 -->
    <a-row :gutter="12" class="mt-16">
      <a-col :xs="12" :md="6" v-for="r in roleSummary" :key="r.role">
        <a-card>
          <a-statistic :title="r.label" :value="r.count">
            <template #prefix><a-avatar :size="28" :style="{ backgroundColor: r.color }">{{ r.label.charAt(0) }}</a-avatar></template>
          </a-statistic>
        </a-card>
      </a-col>
    </a-row>

    <a-table class="mt-16" :data="filteredUsers" :loading="loading" :pagination="{ pageSize: 15, showTotal: true }" row-key="id">
      <template #columns>
        <a-table-column title="用户" :width="180">
          <template #cell="{ record }">
            <a-space>
              <a-avatar :size="32" :style="{ backgroundColor: roleColor(record.role) }">{{ record.displayName?.charAt(0) || record.username.charAt(0) }}</a-avatar>
              <div>
                <div>{{ record.displayName }}</div>
                <div class="muted">@{{ record.username }}</div>
              </div>
            </a-space>
          </template>
        </a-table-column>
        <a-table-column title="角色" :width="120">
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
              <a-tag v-for="s in (record.scopes || []).slice(0, 4)" :key="s" size="small">{{ s }}</a-tag>
              <a-tag v-if="(record.scopes || []).length > 4" size="small">+{{ record.scopes.length - 4 }}</a-tag>
            </a-space>
          </template>
        </a-table-column>
        <a-table-column title="状态" :width="80">
          <template #cell="{ record }">
            <a-badge :status="record.enabled ? 'success' : 'default'" :text="record.enabled ? '正常' : '禁用'" />
          </template>
        </a-table-column>
        <a-table-column title="最后登录" :width="160">
          <template #cell="{ record }">{{ record.lastLogin ? dayjs(record.lastLogin).format('MM-DD HH:mm') : '从未' }}</template>
        </a-table-column>
        <a-table-column title="操作" :width="180" fixed="right">
          <template #cell="{ record }">
            <a-space>
              <a-button size="small" @click="openEdit(record)">编辑</a-button>
              <a-popconfirm content="确认删除该用户？" @ok="onDelete(record.id)" :disabled="record.username === 'admin'">
                <a-button size="small" status="danger" :disabled="record.username === 'admin'">删除</a-button>
              </a-popconfirm>
            </a-space>
          </template>
        </a-table-column>
      </template>
    </a-table>

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

    <!-- 角色管理抽屉 -->
    <a-drawer :visible="showRoleManager" :width="640" @cancel="showRoleManager = false" :footer="false">
      <template #title>角色管理</template>
      <a-space class="mb-16">
        <a-button type="primary" @click="openCreateRole"><template #icon><icon-plus /></template>新建角色</a-button>
      </a-space>
      <a-table :data="allRoles" :pagination="{ pageSize: 20 }" row-key="name">
        <template #columns>
          <a-table-column title="角色" :width="140">
            <template #cell="{ record }">
              <a-tag :color="record.color">{{ record.label }}</a-tag>
              <div class="muted">@{{ record.name }}</div>
            </template>
          </a-table-column>
          <a-table-column title="类型" :width="80">
            <template #cell="{ record }">
              <a-tag :color="record.builtin ? 'arcoblue' : 'green'" size="small">{{ record.builtin ? '预置' : '自定义' }}</a-tag>
            </template>
          </a-table-column>
          <a-table-column title="描述" data-index="description" :width="200" ellipsis />
          <a-table-column title="权限" :width="200">
            <template #cell="{ record }">
              <a-space wrap>
                <a-tag v-for="s in (record.scopes || []).slice(0, 3)" :key="s" size="small">{{ s }}</a-tag>
                <a-tag v-if="(record.scopes || []).length > 3" size="small">+{{ record.scopes.length - 3 }}</a-tag>
              </a-space>
            </template>
          </a-table-column>
          <a-table-column title="操作" :width="140" fixed="right">
            <template #cell="{ record }">
              <a-space>
                <a-button size="small" @click="openEditRole(record)">编辑</a-button>
                <a-popconfirm content="确认删除该角色？" @ok="onDeleteRole(record.name)" :disabled="record.builtin">
                  <a-button size="small" status="danger" :disabled="record.builtin">删除</a-button>
                </a-popconfirm>
              </a-space>
            </template>
          </a-table-column>
        </template>
      </a-table>
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
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted } from 'vue';
import dayjs from 'dayjs';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';

const loading = ref(false);
const users = ref<any[]>([]);
const allRoles = ref<any[]>([]);
const searchKey = ref('');
const visible = ref(false);
const saving = ref(false);
const showRoleManager = ref(false);
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

const roleColors = ['#f53f3f', '#722ed1', '#165dff', '#00b42a', '#ff7d00', '#0fc6c2', '#86909c'];

const editing = reactive<any>({
  id: '', username: '', displayName: '', email: '', password: '',
  role: 'operator', agentId: 'main', enabled: true, approvalEnabled: true, scopes: [],
});

const editingRole = reactive<any>({
  name: '', label: '', description: '', color: '#0fc6c2', scopes: [], builtin: false, isNew: false,
});

const defaultScopes = ref<string[]>([]);

const filteredUsers = computed(() =>
  users.value.filter((u) => !searchKey.value || u.username.toLowerCase().includes(searchKey.value.toLowerCase()) || (u.displayName || '').toLowerCase().includes(searchKey.value.toLowerCase())),
);

const roleSummary = computed(() => {
  const map: Record<string, any> = {};
  for (const r of allRoles.value) {
    map[r.name] = { role: r.name, label: r.label, color: r.color || '#86909c', count: users.value.filter((u) => u.role === r.name).length };
  }
  return Object.values(map);
});

function roleColor(r: string) { return allRoles.value.find((x) => x.name === r)?.color || '#86909c'; }
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

function openCreateRole() {
  Object.assign(editingRole, { name: '', label: '', description: '', color: '#0fc6c2', scopes: [], builtin: false, isNew: true });
  roleVisible.value = true;
}

function openEditRole(r: any) {
  Object.assign(editingRole, { ...r, isNew: false });
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
.muted { color: var(--color-text-3); font-size: 12px; }
.hint { font-size: 12px; color: var(--color-text-3); margin-top: 4px; }
</style>
