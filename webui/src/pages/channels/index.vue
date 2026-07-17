<template>
  <div class="page-container">
    <a-page-header title="渠道管理" subtitle="40+ IM 渠道真实连接" :show-back="false" />
    <a-row :gutter="16" class="mt-16">
      <a-col v-for="ch in channels" :key="ch.id" :xs="24" :sm="12" :md="8" :lg="6">
        <a-card hoverable>
          <div class="ch-card">
            <a-avatar :size="40" :style="{ backgroundColor: ch.color }">{{ ch.label.charAt(0) }}</a-avatar>
            <div class="ch-info">
              <div class="ch-name">{{ ch.label }}
                <a-badge :status="ch.state?.enabled ? (ch.state?.status === 'connected' ? 'success' : (ch.state?.status === 'error' ? 'danger' : 'warning')) : 'default'" />
              </div>
              <div class="ch-status">{{ statusText(ch) }}</div>
            </div>
            <a-switch v-model="ch.enabled" @change="toggle(ch)" />
          </div>
          <div class="ch-stats">
            <span>收 {{ ch.state?.receivedCount || 0 }}</span>
            <span>发 {{ ch.state?.sentCount || 0 }}</span>
          </div>
        </a-card>
      </a-col>
    </a-row>

    <a-drawer :visible="visible" :width="520" @cancel="visible = false" @ok="save" :ok-loading="saving">
      <template #title>配置 {{ current?.label }}</template>
      <a-form v-if="current" :model="current" layout="vertical">
        <a-form-item label="启用"><a-switch v-model="current.enabled" /></a-form-item>
        <a-form-item label="Webhook 接收路径">
          <a-input :model-value="`/webhook/${current.id}`" disabled />
          <div class="hint">将此 URL 配置到 {{ current.label }} 的回调中</div>
        </a-form-item>
        <a-divider>发送配置（JSON）</a-divider>
        <a-form-item label="API 配置">
          <a-textarea v-model="configJson" :auto-size="{ minRows: 8 }" />
        </a-form-item>
        <a-button @click="test" :loading="testing">发送测试消息</a-button>
      </a-form>
    </a-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';

interface Channel {
  id: string; label: string; color: string; enabled: boolean;
  config?: any; state?: any;
}

const channelDefs = [
  { id: 'feishu', label: '飞书', color: '#3370ff' },
  { id: 'dingtalk', label: '钉钉', color: '#1677ff' },
  { id: 'wecom', label: '企业微信', color: '#07c160' },
  { id: 'telegram', label: 'Telegram', color: '#2aabee' },
  { id: 'discord', label: 'Discord', color: '#5865f2' },
  { id: 'slack', label: 'Slack', color: '#4a154b' },
  { id: 'whatsapp', label: 'WhatsApp', color: '#25d366' },
  { id: 'signal', label: 'Signal', color: '#3a76f0' },
  { id: 'qq', label: 'QQ', color: '#12b7f5' },
  { id: 'matrix', label: 'Matrix', color: '#0dbd8b' },
  { id: 'teams', label: 'Teams', color: '#5059c9' },
  { id: 'webhook', label: 'Webhook', color: '#86909c' },
];

const channels = ref<Channel[]>([]);
const visible = ref(false);
const saving = ref(false);
const testing = ref(false);
const current = ref<Channel | null>(null);
const configJson = ref('{}');

function statusText(ch: Channel) {
  if (!ch.state?.enabled) return '未启用';
  const s = ch.state?.status;
  return { configured: '已配置', connected: '已连接', disconnected: '未连接', error: '错误', polling: '轮询中' }[s] || s;
}

async function load() {
  try {
    const res = await rpc.call<{ channels: any[] }>('channels.list');
    const states = await rpc.call<{ states: any }>('channels.states').catch(() => ({ states: {} }));
    const cfgMap = new Map((res.channels || []).map((c) => [c.id, c]));
    channels.value = channelDefs.map((d) => {
      const cfg = cfgMap.get(d.id) || {};
      return {
        ...d,
        enabled: !!cfg.enabled,
        config: cfg,
        state: states.states?.[d.id] || { status: cfg.enabled ? 'configured' : 'disconnected', enabled: !!cfg.enabled },
      };
    });
  } catch (e: any) {
    channels.value = channelDefs.map((d) => ({ ...d, enabled: false }));
  }
}

function openEdit(ch: Channel) {
  current.value = JSON.parse(JSON.stringify(ch));
  configJson.value = JSON.stringify(ch.config || {}, null, 2);
  visible.value = true;
}

async function toggle(ch: Channel) {
  try {
    await rpc.call('channels.set', { id: ch.id, enabled: ch.enabled });
    Message.success(ch.enabled ? '已启用' : '已禁用');
  } catch (e: any) { Message.error(e.message); }
}

async function save() {
  saving.value = true;
  try {
    const cfg = JSON.parse(configJson.value);
    await rpc.call('channels.set', { id: current.value!.id, enabled: current.value!.enabled, config: cfg });
    Message.success('已保存');
    visible.value = false;
    await load();
  } catch (e: any) { Message.error(e.message); }
  finally { saving.value = false; }
}

async function test() {
  testing.value = true;
  try {
    await rpc.call('channels.test', { id: current.value!.id });
    Message.success('测试消息已发送');
  } catch (e: any) { Message.error(e.message); }
  finally { testing.value = false; }
}

onMounted(load);
</script>

<style lang="less" scoped>
.ch-card { display: flex; align-items: center; gap: 12px; }
.ch-info { flex: 1; }
.ch-name { font-weight: 600; color: var(--color-text-1); }
.ch-status { font-size: 12px; color: var(--color-text-3); margin-top: 2px; }
.ch-stats { margin-top: 12px; display: flex; justify-content: space-between; font-size: 12px; color: var(--color-text-3); }
.hint { font-size: 12px; color: var(--color-text-3); margin-top: 4px; }
</style>
