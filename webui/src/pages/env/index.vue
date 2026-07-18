<template>
  <div class="page-container">
    <a-page-header title="环境部署" subtitle="一键安装运行环境（PHP / NodeJS / Python / Go / Java / Nginx / Redis / MySQL / Docker）" :show-back="false">
      <template #extra>
        <a-button @click="loadEnvs"><template #icon><icon-refresh /></template>重新检测</a-button>
      </template>
    </a-page-header>
    <a-row :gutter="24" class="mt-16">
      <a-col :xs="24" :sm="12" :lg="8" v-for="env in envs" :key="env.id">
        <a-card class="env-card">
          <div class="env-head">
            <div class="env-icon" :style="{ background: env.installed ? env.color : '#e0dce8' }">{{ env.label.charAt(0) }}</div>
            <div class="env-info">
              <div class="env-name">{{ env.label }}</div>
              <div class="env-status">
                <a-tag :color="env.installed ? 'green' : 'gray'" size="small">{{ env.installed ? env.version : '未安装' }}</a-tag>
              </div>
            </div>
          </div>
          <div class="env-actions">
            <a-button v-if="!env.installed" type="primary" size="small" :loading="env._installing" @click="installEnv(env)">
              <template #icon><icon-download /></template>安装
            </a-button>
            <template v-else>
              <a-button size="small" @click="installEnv(env)" :loading="env._installing">升级</a-button>
              <a-popconfirm :content="`确认卸载 ${env.label}?`" @ok="uninstallEnv(env)">
                <a-button size="small" status="danger">卸载</a-button>
              </a-popconfirm>
            </template>
          </div>
          <div v-if="env.path" class="env-path">{{ env.path }}</div>
        </a-card>
      </a-col>
    </a-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import { IconRefresh, IconDownload } from '@arco-design/web-vue/es/icon';

interface Env {
  id: string; label: string; color: string;
  installed: boolean; version: string; path: string;
  _installing?: boolean;
}

const envs = ref<Env[]>([]);

const envPresets = [
  { id: 'php', label: 'PHP', color: '#777bb4' },
  { id: 'nodejs', label: 'Node.js', color: '#339933' },
  { id: 'python', label: 'Python', color: '#3776ab' },
  { id: 'go', label: 'Go', color: '#00add8' },
  { id: 'java', label: 'Java', color: '#f89820' },
  { id: 'nginx', label: 'Nginx', color: '#009639' },
  { id: 'redis', label: 'Redis', color: '#dc382d' },
  { id: 'mysql', label: 'MySQL', color: '#4479a1' },
  { id: 'docker', label: 'Docker', color: '#2496ed' },
];

async function loadEnvs() {
  try {
    const res = await rpc.call<any>("env.list");
    const map = new Map(Object.entries(res.environments || {}).map(([id, v]: [string, any]) => [id, v]))((e: any) => [e.id, e]));
    envs.value = envPresets.map((p) => {
      const info = map.get(p.id) || {};
      return { ...p, installed: info.installed || false, version: info.version || '', path: info.path || '' };
    });
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function installEnv(env: Env) {
  env._installing = true;
  try {
    const res = await rpc.call<any>('env.install', { id: env.id });
    if (res.ok) {
      Message.success(res.message || `${env.label} 安装完成`);
      await loadEnvs();
    } else {
      Message.error(res.error || '安装失败');
    }
  } catch (e: any) {
    Message.error(e.message);
  } finally {
    env._installing = false;
  }
}

async function uninstallEnv(env: Env) {
  try {
    const res = await rpc.call<any>('env.uninstall', { id: env.id });
    if (res.ok) {
      Message.success(`${env.label} 已卸载`);
      await loadEnvs();
    } else {
      Message.error(res.error || '卸载失败');
    }
  } catch (e: any) {
    Message.error(e.message);
  }
}

onMounted(loadEnvs);
</script>

<style lang="less" scoped>
.env-card {
  margin-bottom: 16px;
}
.env-head {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 16px;
}
.env-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  font-weight: 700;
  flex-shrink: 0;
  box-shadow: var(--shadow-xs);
}
.env-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-1);
}
.env-status {
  margin-top: 4px;
}
.env-actions {
  display: flex;
  gap: 8px;
}
.env-path {
  font-size: 11px;
  color: var(--color-text-4);
  margin-top: 8px;
  font-family: monospace;
}
</style>
