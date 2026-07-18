<template>
  <div class="auth-wrapper">
    <!-- 背景装饰（Materialize 树插图） -->
    <img src="/illustrations/auth-basic-mask-light.png" class="auth-mask" alt="" />
    <img src="/illustrations/tree.png" class="auth-tree-left" alt="" />
    <img src="/illustrations/tree-3.png" class="auth-tree-right" alt="" />

    <div class="auth-card">
      <!-- Logo -->
      <div class="auth-brand">
        <svg class="brand-logo" width="34" height="28" viewBox="0 0 34 28" fill="none">
          <path d="M2 4 L12 10 L12 22 L2 16 Z" fill="#8c57ff"/>
          <path d="M12 10 L22 4 L22 16 L12 22 Z" fill="#7e4ee6"/>
          <path d="M22 4 L32 10 L32 22 L22 16 Z" fill="#a785fa"/>
          <path d="M12 10 L12 22 L17 25 L17 13 Z" fill="#6d40d8" opacity="0.7"/>
        </svg>
        <span class="brand-text">CradleRing</span>
      </div>

      <!-- 标题 -->
      <h3 class="auth-title">欢迎回来！👋</h3>
      <p class="auth-subtitle">请登录您的账号，开始 AI Agent 协作之旅</p>

      <!-- 表单 -->
      <a-form :model="form" :rules="rules" layout="vertical" @submit="onSubmit" v-if="!show2fa">
        <a-form-item field="username" label="用户名" hide-asterisk>
          <a-input v-model="form.username" placeholder="请输入用户名" size="large" allow-clear>
            <template #prefix><icon-user /></template>
          </a-input>
        </a-form-item>
        <a-form-item field="password" label="密码" hide-asterisk>
          <a-input-password v-model="form.password" placeholder="请输入密码" size="large" allow-clear>
            <template #prefix><icon-lock /></template>
          </a-input-password>
        </a-form-item>

        <div class="auth-options">
          <a-checkbox v-model="form.remember">记住我</a-checkbox>
          <a-link class="forgot-link">忘记密码？</a-link>
        </div>

        <a-button type="primary" html-type="submit" size="large" long :loading="userStore.loading" class="login-btn">
          登录
        </a-button>
      </a-form>

      <!-- 二步验证 -->
      <div v-if="show2fa" class="twofa-step">
        <div class="twofa-icon"><icon-safe /></div>
        <h4 class="twofa-title">{{ twofaMethod === 'totp' ? '请输入身份验证器中的验证码' : '请输入邮箱收到的验证码' }}</h4>
        <p class="twofa-desc">{{ twofaMethod === 'totp' ? '打开谷歌身份验证器，输入 6 位验证码' : `验证码已发送到您的邮箱（10 分钟内有效）` }}</p>
        <a-input
          v-model="twofaCode"
          :max-length="6"
          placeholder="000000"
          size="large"
          style="text-align:center;font-size:24px;letter-spacing:8px;width:220px;margin:0 auto"
          @keydown.enter="confirm2fa"
        />
        <div style="text-align:center;margin-top:16px;display:flex;gap:12px;justify-content:center">
          <a-button @click="cancel2fa">返回</a-button>
          <a-button type="primary" :loading="verifying" @click="confirm2fa">确认</a-button>
          <a-button v-if="twofaMethod === 'email'" @click="resend2fa" :loading="resending">重新发送</a-button>
        </div>
      </div>

      <!-- 首次安装提示 -->
      <div v-if="showInitTip" class="init-tip">
        <icon-info-circle />
        <span>首次安装后，请使用安装时生成的随机凭据登录（保存在数据目录 <code>.admin_credentials</code>）</span>
      </div>

      <div class="auth-footer">
        Copyright © 2026 CradleRing
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { Message } from '@arco-design/web-vue';
import { useUserStore } from '@/stores/user';
import { rpc } from '@/api/rpc';
import { IconUser, IconLock, IconInfoCircle, IconSafe } from '@arco-design/web-vue/es/icon';

const router = useRouter();
const route = useRoute();
const userStore = useUserStore();

const showInitTip = ref(false);

// 二步验证状态
const show2fa = ref(false);
const twofaMethod = ref<'totp' | 'email'>('totp');
const twofaCode = ref('');
const verifying = ref(false);
const resending = ref(false);
const pendingUsername = ref('');

const form = reactive({
  username: '',
  password: '',
  remember: true,
});

const rules = {
  username: [{ required: true, message: '请输入用户名' }],
  password: [{ required: true, message: '请输入密码' }],
};

async function onSubmit() {
  if (!form.username || !form.password) {
    Message.warning('请输入用户名和密码');
    return;
  }
  try {
    // 尝试登录，后端可能返回需要二步验证
    const res = await fetch('/api/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: form.username, password: form.password }),
    });
    const data = await res.json().catch(() => ({}));
    if (data.ok) {
      // 登录成功（无 2FA）
      rpc.login(form.username, form.password).then(() => {
        Message.success('登录成功');
        const redirect = (route.query.redirect as string) || '/';
        router.push(redirect);
      });
      return;
    }
    // 检查是否需要二步验证
    if (data.requires2fa) {
      twofaMethod.value = data.method || 'totp';
      pendingUsername.value = form.username;
      show2fa.value = true;
      if (twofaMethod.value === 'email') {
        // 后端已自动发送邮件验证码
        Message.info('验证码已发送到您的邮箱');
      }
    } else {
      Message.error(data.error?.message || '登录失败');
    }
  } catch (e: any) {
    Message.error(e.message || '登录失败');
  }
}

async function confirm2fa() {
  if (!twofaCode.value || twofaCode.value.length !== 6) {
    Message.warning('请输入 6 位验证码');
    return;
  }
  verifying.value = true;
  try {
    const res = await fetch('/api/login/2fa', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: pendingUsername.value, code: twofaCode.value, password: form.password }),
    });
    const data = await res.json().catch(() => ({}));
    if (data.ok) {
      // 设置 token
      localStorage.setItem('cradle_token', data.token);
      localStorage.setItem('cradle_user', JSON.stringify(data.user));
      Message.success('登录成功');
      const redirect = (route.query.redirect as string) || '/';
      router.push(redirect);
    } else {
      Message.error(data.error?.message || '验证码错误');
    }
  } catch (e: any) {
    Message.error(e.message);
  } finally {
    verifying.value = false;
  }
}

function cancel2fa() {
  show2fa.value = false;
  twofaCode.value = '';
}

async function resend2fa() {
  resending.value = true;
  try {
    await fetch('/api/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: pendingUsername.value, password: form.password }),
    });
    Message.success('已重新发送');
  } catch { /* ignore */ }
  finally { resending.value = false; }
}

// 检查是否是首次访问（提示用户使用安装时生成的密码）
onMounted(() => {
  if (!localStorage.getItem('cradle_token') && !localStorage.getItem('cradle_user')) {
    showInitTip.value = true;
  }
});
</script>

<style lang="less" scoped>
.auth-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background-color: #f8f7fa;
  overflow: hidden;
}

body[arco-theme='dark'] .auth-wrapper {
  background-color: #28243d;
}

/* 背景装饰 */
.auth-mask {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 172px;
  object-fit: cover;
  opacity: 0.9;
  pointer-events: none;
}
.auth-tree-left {
  position: absolute;
  bottom: 8%;
  left: 4%;
  width: 150px;
  pointer-events: none;
  z-index: 1;
}
.auth-tree-right {
  position: absolute;
  bottom: 8%;
  right: 4%;
  width: 180px;
  pointer-events: none;
  z-index: 1;
}

/* 登录卡片 */
.auth-card {
  position: relative;
  z-index: 2;
  width: 460px;
  max-width: calc(100vw - 32px);
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 0.25rem 1rem 0 rgba(46, 38, 61, 0.1);
  padding: 40px 40px 24px;
}

body[arco-theme='dark'] .auth-card {
  background: #2f2b40;
  box-shadow: 0 0.25rem 1rem 0 rgba(0, 0, 0, 0.35);
}

/* Logo */
.auth-brand {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 32px;
  .brand-logo {
    display: block;
  }
  .brand-text {
    font-size: 24px;
    font-weight: 700;
    color: var(--color-text-1);
    letter-spacing: 0.5px;
  }
}

/* 标题 */
.auth-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--color-text-1);
  margin: 0 0 8px;
}
.auth-subtitle {
  font-size: 14px;
  color: var(--color-text-3);
  margin: 0 0 28px;
  line-height: 1.6;
}

/* 表单间距 */
:deep(.arco-form-item) {
  margin-bottom: 22px;
}
:deep(.arco-form-item-label) {
  font-weight: 500;
  color: var(--color-text-2);
  font-size: 13px;
}
:deep(.arco-input-wrapper),
:deep(.arco-input-password) {
  border-radius: 6px;
  &:hover {
    border-color: var(--primary-6);
  }
}

/* 记住我 + 忘记密码 */
.auth-options {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  .forgot-link {
    font-size: 13px;
  }
}

/* 登录按钮 */
.login-btn {
  height: 44px;
  font-size: 15px;
  font-weight: 500;
  border-radius: 6px;
}

/* 首次提示 */
.init-tip {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin-top: 20px;
  padding: 12px 14px;
  background: rgba(22, 177, 255, 0.08);
  border-radius: 6px;
  font-size: 12px;
  color: var(--color-text-3);
  line-height: 1.6;
  svg {
    color: #16b1ff;
    flex-shrink: 0;
    margin-top: 2px;
  }
  code {
    background: var(--color-bg-3);
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 11px;
  }
}

/* 页脚 */
.auth-footer {
  margin-top: 32px;
  text-align: center;
  font-size: 12px;
  color: var(--color-text-4);
}

/* 移动端：隐藏插图 */
@media (max-width: 992px) {
  .auth-tree-left, .auth-tree-right, .auth-mask {
    display: none;
  }
}
</style>
