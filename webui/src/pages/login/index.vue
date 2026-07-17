<template>
  <div class="login-container">
    <div class="login-banner">
      <div class="banner-inner">
        <div class="logo-row">
          <icon-storage class="logo-icon" />
          <span class="logo-text">CradleRing</span>
        </div>
        <h1 class="slogan">AI Agent 智能协作平台</h1>
        <p class="subtitle">企业级 AI Agent 协作平台 · 多级审批 · 多账号 · 全渠道接入</p>
        <div class="features">
          <div class="feature-item"><icon-check-circle /> 40+ IM 渠道真实连接</div>
          <div class="feature-item"><icon-check-circle /> 多级审批工作流</div>
          <div class="feature-item"><icon-check-circle /> 多账号角色权限</div>
          <div class="feature-item"><icon-check-circle /> 28+ 内置工具</div>
          <div class="feature-item"><icon-check-circle /> 15+ 搜索引擎</div>
          <div class="feature-item"><icon-check-circle /> 无损上下文压缩</div>
        </div>
      </div>
    </div>

    <div class="login-form-wrap">
      <div class="login-form">
        <div class="form-header">
          <h2>欢迎回来 👋</h2>
          <p>请使用您的账号登录控制台</p>
        </div>

        <a-form
          ref="formRef"
          :model="form"
          :rules="rules"
          layout="vertical"
          @submit="onSubmit"
        >
          <a-form-item field="username" label="用户名">
            <a-input
              v-model="form.username"
              placeholder="请输入用户名"
              size="large"
              allow-clear
            >
              <template #prefix><icon-user /></template>
            </a-input>
          </a-form-item>
          <a-form-item field="password" label="密码">
            <a-input-password
              v-model="form.password"
              placeholder="请输入密码"
              size="large"
              allow-clear
              @keyup.enter="onSubmit"
            >
              <template #prefix><icon-lock /></template>
            </a-input-password>
          </a-form-item>
          <a-form-item>
            <div class="form-options">
              <a-checkbox v-model="form.remember">记住我</a-checkbox>
              <a-link>忘记密码？</a-link>
            </div>
          </a-form-item>
          <a-form-item>
            <a-button
              type="primary"
              html-type="submit"
              long
              size="large"
              :loading="userStore.loading"
            >
              登录
            </a-button>
          </a-form-item>
          <a-form-item v-if="showInitTip">
            <a-alert type="info" banner>
              首次安装后，请使用安装时生成的随机密码登录
            </a-alert>
          </a-form-item>
        </a-form>
      </div>

      <div class="footer">
        Copyright © 2026 CradleRing · 基于 Arco Design Pro
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { Message } from '@arco-design/web-vue';
import { useUserStore } from '@/stores/user';

const router = useRouter();
const route = useRoute();
const userStore = useUserStore();

const showInitTip = ref(false);

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
    await userStore.login(form.username, form.password);
    Message.success('登录成功');
    const redirect = (route.query.redirect as string) || '/';
    router.push(redirect);
  } catch (e: any) {
    Message.error(e.message || '登录失败');
  }
}

// 检查是否是首次访问（提示用户使用安装时生成的密码）
onMounted(() => {
  if (!localStorage.getItem('cradle_token') && !localStorage.getItem('cradle_user')) {
    showInitTip.value = true;
  }
});
</script>

<style lang="less" scoped>
.login-container {
  display: flex;
  height: 100vh;
  background-color: var(--color-bg-1);
}

.login-banner {
  flex: 1;
  background: linear-gradient(135deg, #8c57ff 0%, #16b1ff 100%);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;

  &::before {
    content: '';
    position: absolute;
    inset: 0;
    background-image: radial-gradient(circle at 20% 30%, rgba(255, 255, 255, 0.15) 0%, transparent 50%),
      radial-gradient(circle at 80% 70%, rgba(255, 255, 255, 0.1) 0%, transparent 50%);
  }
}

.banner-inner {
  position: relative;
  max-width: 460px;
  padding: 0 40px;
}

.logo-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 32px;

  .logo-icon {
    font-size: 40px;
  }
  .logo-text {
    font-size: 28px;
    font-weight: 800;
    letter-spacing: 1px;
  }
}

.slogan {
  font-size: 36px;
  font-weight: 700;
  line-height: 1.3;
  margin: 0 0 16px;
}

.subtitle {
  font-size: 15px;
  opacity: 0.9;
  margin: 0 0 40px;
  line-height: 1.6;
}

.features {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;

  .feature-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    opacity: 0.95;
  }
}

.login-form-wrap {
  width: 460px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 0 60px;
  background-color: var(--color-bg-1);
}

.login-form {
  width: 100%;
  max-width: 340px;
  margin: 0 auto;
}

.form-header {
  text-align: center;
  margin-bottom: 32px;

  h2 {
    font-size: 26px;
    font-weight: 700;
    margin: 0 0 8px;
    color: var(--color-text-1);
  }
  p {
    font-size: 14px;
    color: var(--color-text-3);
    margin: 0;
  }
}

.form-options {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.footer {
  text-align: center;
  margin-top: 40px;
  font-size: 12px;
  color: var(--color-text-3);
}

@media (max-width: 768px) {
  .login-banner {
    display: none;
  }
  .login-form-wrap {
    width: 100%;
    padding: 0 20px;
  }
}
</style>
