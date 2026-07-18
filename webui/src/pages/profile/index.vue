<template>
  <div class="page-container">
    <div class="pill-tabs mb-16">
      <div v-for="t in tabs" :key="t.key" class="pill-tab" :class="{ active: activeTab === t.key }" @click="activeTab = t.key">
        <component :is="t.icon" /><span>{{ t.label }}</span>
      </div>
    </div>

    <!-- 个人资料 -->
    <a-card v-show="activeTab === 'profile'">
      <div class="profile-header">
        <div class="big-avatar">{{ (userStore.user?.displayName || 'U').charAt(0).toUpperCase() }}</div>
        <div>
          <h3 style="margin:0">{{ userStore.user?.displayName || userStore.user?.username }}</h3>
          <a-tag :color="roleColor" size="small">{{ roleLabel }}</a-tag>
          <p style="margin:4px 0 0;color:var(--color-text-3);font-size:13px">@{{ userStore.user?.username }}</p>
        </div>
      </div>
      <a-divider />
      <a-form :model="profileForm" layout="vertical" style="max-width:560px">
        <a-form-item label="显示名称"><a-input v-model="profileForm.displayName" /></a-form-item>
        <a-form-item label="邮箱"><a-input v-model="profileForm.email" placeholder="user@example.com" /></a-form-item>
        <a-form-item label="关联 Agent"><a-input v-model="profileForm.agentId" disabled /></a-form-item>
        <a-button type="primary" :loading="savingProfile" @click="saveProfile">保存修改</a-button>
      </a-form>
    </a-card>

    <!-- 安全 -->
    <a-card v-show="activeTab === 'security'">
      <a-form layout="vertical" style="max-width:560px">
        <a-divider orientation="left">修改密码</a-divider>
        <a-form-item label="当前密码"><a-input-password v-model="pwForm.oldPassword" /></a-form-item>
        <a-form-item label="新密码"><a-input-password v-model="pwForm.newPassword" /></a-form-item>
        <a-form-item label="确认新密码"><a-input-password v-model="pwForm.confirmPassword" /></a-form-item>
        <a-button type="primary" :loading="savingPw" @click="changePassword">修改密码</a-button>

        <a-divider orientation="left">二步验证（2FA）</a-divider>
        <div class="twofa-section">
          <div class="twofa-status">
            <a-tag :color="twofaStatus.enabled ? 'green' : 'gray'">{{ twofaStatus.enabled ? '已启用' : '未启用' }}</a-tag>
            <span v-if="twofaStatus.enabled" class="muted">{{ twofaStatus.method === 'totp' ? '谷歌身份验证器' : '邮件验证码' }}</span>
          </div>
          <div class="twofa-method" v-if="!twofaStatus.enabled">
            <a-radio-group v-model="twofaMethod" type="button">
              <a-radio value="totp">谷歌身份验证器</a-radio>
              <a-radio value="email">邮件验证码</a-radio>
            </a-radio-group>
            <a-button type="primary" :loading="settingUp2fa" @click="start2faSetup" style="margin-top:12px">启用二步验证</a-button>
          </div>
          <!-- TOTP 设置流程 -->
          <div v-if="totpSetup.secret" class="totp-setup">
            <p>请用谷歌身份验证器扫描下方二维码，或手动输入密钥：</p>
            <div class="totp-qr"><img :src="totpSetup.qrUrl" alt="QR Code" style="width:200px;height:200px" /></div>
            <a-input :model-value="totpSetup.secret" readonly style="margin-top:8px" />
            <a-form-item label="输入验证器中的 6 位验证码" style="margin-top:12px">
              <a-input v-model="totpCode" placeholder="000000" :max-length="6" style="width:160px" />
              <a-button type="primary" style="margin-left:12px" :loading="verifying2fa" @click="verify2faCode">确认启用</a-button>
            </a-form-item>
          </div>
          <!-- 邮件验证码设置 -->
          <div v-if="email2faSent" class="email2fa-setup">
            <p>验证码已发送到 {{ profileForm.email || '你的邮箱' }}</p>
            <a-input v-model="emailCode" placeholder="6 位验证码" :max-length="6" style="width:160px" />
            <a-button type="primary" style="margin-left:12px" :loading="verifying2fa" @click="verifyEmail2fa">确认启用</a-button>
          </div>
          <a-button v-if="twofaStatus.enabled" status="danger" style="margin-top:12px" @click="disable2fa">关闭二步验证</a-button>
        </div>
      </a-form>
    </a-card>

    <!-- 偏好 -->
    <a-card v-show="activeTab === 'preference'">
      <a-form layout="vertical" style="max-width:560px">
        <a-form-item label="主题模式">
          <a-radio-group v-model="appStore.themeMode" type="button" @change="(v:any) => appStore.setThemeMode(v)">
            <a-radio value="light">浅色</a-radio><a-radio value="dark">深色</a-radio><a-radio value="system">跟随系统</a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item label="语言">
          <a-radio-group v-model="appStore.locale" type="button" @change="(v:any) => appStore.setLocale(v)">
            <a-radio value="zh-CN">中文</a-radio><a-radio value="en-US">English</a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item label="侧栏默认折叠"><a-switch v-model="appStore.menuCollapse" /></a-form-item>
      </a-form>
    </a-card>

    <!-- API Token -->
    <a-card v-show="activeTab === 'token'">
      <a-alert type="info" class="mb-16">API Token 用于程序化调用网关 RPC（WebSocket），与登录密码独立</a-alert>
      <a-form-item label="当前 Gateway Token">
        <a-input-password :model-value="rpc.getToken()" readonly />
      </a-form-item>
      <a-button type="primary" @click="copyToken"><template #icon><icon-copy /></template>复制 Token</a-button>
    </a-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, markRaw } from 'vue';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import { useUserStore } from '@/stores/user';
import { useAppStore } from '@/stores/app';
import { IconUser, IconSafe, IconSettings, IconLock } from '@arco-design/web-vue/es/icon';

const userStore = useUserStore();
const appStore = useAppStore();

const tabs = [
  { key: 'profile', label: '个人资料', icon: markRaw(IconUser) },
  { key: 'security', label: '安全', icon: markRaw(IconSafe) },
  { key: 'preference', label: '偏好', icon: markRaw(IconSettings) },
  { key: 'token', label: 'API Token', icon: markRaw(IconLock) },
];

const activeTab = ref('profile');
const savingProfile = ref(false);
const savingPw = ref(false);
const profileForm = reactive({
  displayName: userStore.user?.displayName || '',
  email: userStore.user?.email || '',
  agentId: userStore.user?.agentId || 'main',
});

const pwForm = reactive({ oldPassword: '', newPassword: '', confirmPassword: '' });

const roleLabel = computed(() => {
  const map: Record<string,string> = { admin:'管理员', manager:'经理', supervisor:'主管', operator:'操作员', viewer:'访客' };
  return map[userStore.role] || userStore.role;
});
const roleColor = computed(() => {
  const map: Record<string,string> = { admin:'red', manager:'purple', supervisor:'arcoblue', operator:'green', viewer:'gray' };
  return map[userStore.role] || 'gray';
});

async function saveProfile() {
  savingProfile.value = true;
  try {
    await rpc.call('users.updateProfile', { displayName: profileForm.displayName, email: profileForm.email });
    Message.success('已保存');
    await userStore.refresh();
  } catch (e: any) { Message.error(e.message); }
  finally { savingProfile.value = false; }
}

async function changePassword() {
  if (!pwForm.oldPassword || !pwForm.newPassword) { Message.warning('请填写密码'); return; }
  if (pwForm.newPassword !== pwForm.confirmPassword) { Message.warning('两次新密码不一致'); return; }
  savingPw.value = true;
  try {
    await rpc.call('users.changePassword', { oldPassword: pwForm.oldPassword, newPassword: pwForm.newPassword });
    Message.success('密码已修改');
    pwForm.oldPassword = ''; pwForm.newPassword = ''; pwForm.confirmPassword = '';
  } catch (e: any) { Message.error(e.message); }
  finally { savingPw.value = false; }
}

// ========== 二步验证 ==========
const twofaStatus = reactive({ enabled: false, method: '' });
const twofaMethod = ref('totp');
const settingUp2fa = ref(false);
const verifying2fa = ref(false);
const totpSetup = reactive({ secret: '', qrUrl: '' });
const totpCode = ref('');
const email2faSent = ref(false);
const emailCode = ref('');

async function load2faStatus() {
  try {
    const res = await rpc.call<any>('auth.2fa.status');
    twofaStatus.enabled = res.enabled || false;
    twofaStatus.method = res.method || '';
  } catch { /* ignore */ }
}

async function start2faSetup() {
  settingUp2fa.value = true;
  totpSetup.secret = ''; totpSetup.qrUrl = ''; email2faSent.value = false;
  try {
    if (twofaMethod.value === 'totp') {
      const res = await rpc.call<any>('auth.2fa.setup', { method: 'totp' });
      totpSetup.secret = res.secret || '';
      totpSetup.qrUrl = res.qrUrl || '';
    } else {
      // 邮件方式：发送验证码
      await rpc.call('auth.2fa.send_email_code');
      email2faSent.value = true;
      Message.success('验证码已发送到你的邮箱');
    }
  } catch (e: any) { Message.error(e.message); }
  finally { settingUp2fa.value = false; }
}

async function verify2faCode() {
  if (!totpCode.value || totpCode.value.length !== 6) { Message.warning('请输入 6 位验证码'); return; }
  verifying2fa.value = true;
  try {
    await rpc.call('auth.2fa.verify', { code: totpCode.value, method: 'totp' });
    Message.success('二步验证已启用');
    totpSetup.secret = ''; totpCode.value = '';
    await load2faStatus();
  } catch (e: any) { Message.error(e.message); }
  finally { verifying2fa.value = false; }
}

async function verifyEmail2fa() {
  if (!emailCode.value || emailCode.value.length !== 6) { Message.warning('请输入 6 位验证码'); return; }
  verifying2fa.value = true;
  try {
    await rpc.call('auth.2fa.verify', { code: emailCode.value, method: 'email' });
    Message.success('二步验证已启用');
    email2faSent.value = false; emailCode.value = '';
    await load2faStatus();
  } catch (e: any) { Message.error(e.message); }
  finally { verifying2fa.value = false; }
}

async function disable2fa() {
  try {
    await rpc.call('auth.2fa.disable');
    Message.success('已关闭');
    await load2faStatus();
  } catch (e: any) { Message.error(e.message); }
}

function copyToken() {
  navigator.clipboard.writeText(rpc.getToken()).then(() => Message.success('已复制'));
}

onMounted(() => { load2faStatus(); });
</script>

<style lang="less" scoped>
.pill-tabs { display: flex; gap: 8px; }
.pill-tab {
  display: flex; align-items: center; gap: 8px; padding: 9px 18px;
  border-radius: 8px; font-size: 14px; font-weight: 500; color: var(--color-text-2);
  cursor: pointer; transition: all .2s; border: 1px solid transparent;
  &:hover { background: var(--color-bg-1); color: var(--brand-primary); }
  &.active { background: var(--brand-primary); color: #fff; box-shadow: var(--shadow-xs); }
}
.profile-header { display: flex; align-items: center; gap: 20px; }
.big-avatar {
  width: 80px; height: 80px; border-radius: 50%; background: var(--brand-primary);
  color: #fff; display: flex; align-items: center; justify-content: center;
  font-size: 32px; font-weight: 700; flex-shrink: 0; box-shadow: var(--shadow-xs);
}
.muted { color: var(--color-text-3); font-size: 13px; margin-left: 8px; }
.twofa-section { padding: 8px 0; }
.twofa-status { margin-bottom: 12px; }
.totp-setup { margin-top: 16px; }
.totp-qr { display: flex; justify-content: center; margin: 12px 0; }
.mb-16 { margin-bottom: 16px; }
</style>
