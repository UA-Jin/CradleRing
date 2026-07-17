<template>
  <div class="page-container">
    <a-page-header title="设置" subtitle="个人偏好 · 主题 · 外观" :show-back="false" />
    <a-row :gutter="16" class="mt-16">
      <a-col :xs="24" :md="12">
        <a-card title="外观">
          <a-form layout="vertical">
            <a-form-item label="暗色模式">
              <a-switch :model-value="appStore.isDark" @change="appStore.toggleDark()" />
            </a-form-item>
            <a-form-item label="主题色">
              <a-radio-group type="button" v-model="appStore.themeColor">
                <a-radio v-for="c in colors" :key="c.value" :value="c.value">{{ c.label }}</a-radio>
              </a-radio-group>
            </a-form-item>
            <a-form-item label="侧边栏折叠">
              <a-switch v-model="appStore.menuCollapse" />
            </a-form-item>
          </a-form>
        </a-card>
      </a-col>
      <a-col :xs="24" :md="12">
        <a-card title="个人资料">
          <a-form layout="vertical">
            <a-form-item label="用户名">
              <a-input :model-value="userStore.user?.username" disabled />
            </a-form-item>
            <a-form-item label="显示名称">
              <a-input v-model="profile.displayName" />
            </a-form-item>
            <a-form-item label="邮箱">
              <a-input v-model="profile.email" placeholder="user@example.com" />
            </a-form-item>
            <a-form-item label="启用审批（关闭后危险命令免审批）">
              <a-switch v-model="profile.approvalEnabled" />
            </a-form-item>
            <a-form-item label="修改密码（留空不修改）">
              <a-input-password v-model="profile.password" placeholder="••••••" />
            </a-form-item>
            <a-button type="primary" :loading="saving" @click="saveProfile">保存资料</a-button>
          </a-form>
        </a-card>
      </a-col>
    </a-row>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch, onMounted } from 'vue';
import { Message } from '@arco-design/web-vue';
import { useAppStore } from '@/stores/app';
import { useUserStore } from '@/stores/user';
import { rpc } from '@/api/rpc';

const appStore = useAppStore();
const userStore = useUserStore();

const colors = [
  { value: '#165dff', label: '蓝' },
  { value: '#00b42a', label: '绿' },
  { value: '#ff7d00', label: '橙' },
  { value: '#f53f3f', label: '红' },
  { value: '#722ed1', label: '紫' },
];

const profile = reactive({ displayName: '', email: '', approvalEnabled: true, password: '' });
const saving = ref(false);

watch(() => userStore.user, (u) => {
  if (u) {
    profile.displayName = u.displayName;
    profile.email = u.email || '';
    profile.approvalEnabled = u.approvalEnabled;
    profile.password = '';
  }
}, { immediate: true });

async function saveProfile() {
  saving.value = true;
  try {
    const res = await rpc.call<{ ok: boolean; user: any }>('users.updateProfile', {
      token: rpc.getToken(),
      displayName: profile.displayName,
      email: profile.email,
      approvalEnabled: profile.approvalEnabled,
      password: profile.password || undefined,
    });
    if (res.user) userStore.setUser(res.user);
    Message.success('已保存');
    profile.password = '';
  } catch (e: any) { Message.error(e.message); }
  finally { saving.value = false; }
}
</script>
