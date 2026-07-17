<template>
  <div class="page-container">
    <a-page-header title="配置管理" subtitle="傻瓜式表单编辑 · JSON 高级编辑 · 自定义路径" :show-back="false">
      <template #extra>
        <a-space>
          <a-button @click="loadConfig"><template #icon><icon-refresh /></template>重新加载</a-button>
          <a-button type="primary" :loading="saving" @click="saveAll">
            <template #icon><icon-save /></template>保存全部
          </a-button>
        </a-space>
      </template>
    </a-page-header>

    <a-tabs v-model:active-key="activeTab" type="rounded" class="mt-16">
      <!-- 表单模式 -->
      <a-tab-pane key="form" title="表单模式">
        <a-tabs v-model:active-key="activeSection" type="card-gutter" class="mt-8">
          <!-- Provider -->
          <a-tab-pane key="providers" title="Provider">
            <a-card title="LLM Provider 配置" class="mt-16">
              <div v-for="(p, idx) in formData.providers" :key="idx" class="provider-card">
                <div class="provider-head">
                  <a-input v-model="p.name" placeholder="Provider 名称（如 openai）" style="width: 200px" />
                  <a-space>
                    <a-switch v-model="p.enabled" checked-text="启用" unchecked-text="禁用" />
                    <a-button status="danger" size="small" @click="formData.providers.splice(idx, 1)">删除</a-button>
                  </a-space>
                </div>
                <a-row :gutter="12" class="mt-8">
                  <a-col :span="12">
                    <a-form-item label="API Key">
                      <a-input-password v-model="p.apiKey" placeholder="sk-..." allow-clear />
                    </a-form-item>
                  </a-col>
                  <a-col :span="12">
                    <a-form-item label="Base URL">
                      <a-input v-model="p.baseUrl" placeholder="https://api.openai.com/v1" />
                    </a-form-item>
                  </a-col>
                </a-row>
                <a-row :gutter="12">
                  <a-col :span="12">
                    <a-form-item label="模型">
                      <a-input v-model="p.model" placeholder="gpt-4o-mini" />
                    </a-form-item>
                  </a-col>
                  <a-col :span="12">
                    <a-form-item label="支持 Thinking">
                      <a-switch v-model="p.supportsThinking" />
                    </a-form-item>
                  </a-col>
                </a-row>
              </div>
              <a-button long class="mt-8" @click="addProvider"><template #icon><icon-plus /></template>添加 Provider</a-button>
            </a-card>
          </a-tab-pane>

          <!-- Gateway -->
          <a-tab-pane key="gateway" title="Gateway">
            <a-card title="网关基础配置" class="mt-16">
              <a-form :model="formData.gateway" layout="vertical">
                <a-row :gutter="12">
                  <a-col :span="12">
                    <a-form-item label="Gateway Token">
                      <a-input-password v-model="formData.gateway.token" placeholder="访问令牌" />
                    </a-form-item>
                  </a-col>
                  <a-col :span="12">
                    <a-form-item label="端口">
                      <a-input-number v-model="formData.gateway.port" :min="1" :max="65535" />
                    </a-form-item>
                  </a-col>
                </a-row>
              </a-form>
            </a-card>
          </a-tab-pane>

          <!-- Models -->
          <a-tab-pane key="models" title="模型">
            <a-card title="模型配置" class="mt-16">
              <a-form :model="formData.models" layout="vertical">
                <a-form-item label="默认模型">
                  <a-select v-model="formData.models.primary" placeholder="选择默认模型">
                    <a-option v-for="p in formData.providers" :key="p.name" :value="p.model || p.name">
                      {{ p.name }} / {{ p.model || '默认' }}
                    </a-option>
                  </a-select>
                </a-form-item>
                <a-form-item label="备用模型（按顺序 fallback）">
                  <a-select v-model="formData.models.fallbacks" multiple placeholder="选择备用模型">
                    <a-option v-for="p in formData.providers" :key="p.name" :value="p.model || p.name">
                      {{ p.name }} / {{ p.model || '默认' }}
                    </a-option>
                  </a-select>
                </a-form-item>
              </a-form>
            </a-card>
          </a-tab-pane>

          <!-- Channels -->
          <a-tab-pane key="channels" title="渠道">
            <a-card title="IM 渠道配置" class="mt-16">
              <a-collapse :default-active-key="[]">
                <a-collapse-item v-for="ch in formData.channels" :key="ch.id" :header="ch.label">
                  <a-form layout="vertical">
                    <a-form-item label="启用">
                      <a-switch v-model="ch.enabled" />
                    </a-form-item>
                    <a-form-item label="Webhook 路径">
                      <a-input :model-value="`/webhook/${ch.id}`" disabled />
                    </a-form-item>
                    <a-form-item label="配置（JSON）">
                      <a-textarea v-model="ch.configJson" :auto-size="{ minRows: 4 }" placeholder='{"appId":"...","appSecret":"..."}' />
                    </a-form-item>
                  </a-form>
                </a-collapse-item>
              </a-collapse>
            </a-card>
          </a-tab-pane>

          <!-- Search -->
          <a-tab-pane key="search" title="搜索">
            <a-card title="搜索引擎配置" class="mt-16">
              <a-form :model="formData.search" layout="vertical">
                <a-form-item label="默认搜索引擎">
                  <a-select v-model="formData.search.default">
                    <a-option value="searxng">SearXNG</a-option>
                    <a-option value="brave">Brave</a-option>
                    <a-option value="tavily">Tavily</a-option>
                    <a-option value="duckduckgo">DuckDuckGo</a-option>
                    <a-option value="google">Google</a-option>
                    <a-option value="bing">Bing</a-option>
                  </a-select>
                </a-form-item>
                <a-form-item label="SearXNG URL">
                  <a-input v-model="formData.search.searxngUrl" placeholder="https://searx.example.com" />
                </a-form-item>
                <a-form-item label="Brave API Key">
                  <a-input-password v-model="formData.search.braveKey" placeholder="BSA..." />
                </a-form-item>
                <a-form-item label="Tavily API Key">
                  <a-input-password v-model="formData.search.tavilyKey" placeholder="tvly-..." />
                </a-form-item>
              </a-form>
            </a-card>
          </a-tab-pane>

          <!-- TTS -->
          <a-tab-pane key="tts" title="TTS">
            <a-card title="语音合成配置" class="mt-16">
              <a-form :model="formData.tts" layout="vertical">
                <a-form-item label="默认 TTS 引擎">
                  <a-select v-model="formData.tts.default">
                    <a-option value="openai">OpenAI TTS</a-option>
                    <a-option value="edge">Microsoft Edge TTS</a-option>
                    <a-option value="azure">Azure Speech</a-option>
                    <a-option value="elevenlabs">ElevenLabs</a-option>
                  </a-select>
                </a-form-item>
                <a-form-item label="OpenAI TTS Voice">
                  <a-select v-model="formData.tts.openaiVoice">
                    <a-option value="alloy">Alloy</a-option>
                    <a-option value="echo">Echo</a-option>
                    <a-option value="fable">Fable</a-option>
                    <a-option value="onyx">Onyx</a-option>
                    <a-option value="nova">Nova</a-option>
                    <a-option value="shimmer">Shimmer</a-option>
                  </a-select>
                </a-form-item>
              </a-form>
            </a-card>
          </a-tab-pane>
        </a-tabs>
      </a-tab-pane>

      <!-- JSON 模式 -->
      <a-tab-pane key="json" title="JSON 模式">
        <a-alert type="warning" class="mt-8">高级模式：直接编辑 JSON 配置，适合有特殊需求的用户</a-alert>
        <a-card class="mt-8">
          <a-textarea
            v-model="jsonText"
            :auto-size="{ minRows: 20, maxRows: 40 }"
            style="font-family: 'Menlo', 'Monaco', monospace; font-size: 13px"
            placeholder='{"gateway": {"token": "...", "port": 18800}, "providers": {...}}'
          />
        </a-card>
      </a-tab-pane>

      <!-- 自定义 path -->
      <a-tab-pane key="custom" title="自定义路径">
        <a-card title="自定义配置路径编辑器" class="mt-8">
          <a-form layout="vertical">
            <a-form-item label="JSON 路径（如 providers.openai.apiKey）">
              <a-input v-model="customPath" placeholder="gateway.token" />
            </a-form-item>
            <a-form-item label="当前值">
              <a-textarea :model-value="customValue" :auto-size="{ minRows: 3 }" readonly />
            </a-form-item>
            <a-form-item label="新值">
              <a-textarea v-model="customNewValue" :auto-size="{ minRows: 3 }" placeholder="输入新值（JSON 格式）" />
            </a-form-item>
            <a-space>
              <a-button @click="loadCustomValue">读取</a-button>
              <a-button type="primary" @click="saveCustomValue">保存到该路径</a-button>
            </a-space>
          </a-form>
        </a-card>
      </a-tab-pane>
    </a-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';

const activeTab = ref('form');
const activeSection = ref('providers');
const saving = ref(false);
const jsonText = ref('');

interface ProviderForm { name: string; apiKey: string; baseUrl: string; model: string; enabled: boolean; supportsThinking: boolean; }
interface ChannelForm { id: string; label: string; enabled: boolean; configJson: string; }

const formData = reactive({
  providers: [] as ProviderForm[],
  gateway: { token: '', port: 18800 },
  models: { primary: '', fallbacks: [] as string[] },
  channels: [] as ChannelForm[],
  search: { default: 'searxng', searxngUrl: '', braveKey: '', tavilyKey: '' },
  tts: { default: 'openai', openaiVoice: 'alloy' },
});

const customPath = ref('');
const customValue = ref('');
const customNewValue = ref('');

const channelDefs = [
  { id: 'feishu', label: '飞书' }, { id: 'dingtalk', label: '钉钉' },
  { id: 'wecom', label: '企业微信' }, { id: 'telegram', label: 'Telegram' },
  { id: 'discord', label: 'Discord' }, { id: 'slack', label: 'Slack' },
  { id: 'whatsapp', label: 'WhatsApp' }, { id: 'signal', label: 'Signal' },
  { id: 'qq', label: 'QQ' }, { id: 'matrix', label: 'Matrix' },
  { id: 'teams', label: 'Teams' }, { id: 'webhook', label: 'Webhook' },
];

async function loadConfig() {
  try {
    const res = await rpc.call<{ config: any }>('config.get');
    const cfg = res.config || {};
    jsonText.value = JSON.stringify(cfg, null, 2);

    // 解析 providers
    const providers = cfg.providers || {};
    formData.providers = Object.entries(providers)
      .filter(([k, v]: [string, any]) => v && typeof v === 'object' && !['search', 'tts', 'stt', 'embedding', 'rerank', 'vision', 'image'].includes(k))
      .map(([k, v]: [string, any]) => ({
        name: k,
        apiKey: v.apiKey || '',
        baseUrl: v.baseUrl || '',
        model: v.model || '',
        enabled: v.enabled !== false,
        supportsThinking: v.supportsThinking || false,
      }));

    // gateway
    formData.gateway.token = cfg.gateway?.token || cfg.gateway?.auth?.token || '';
    formData.gateway.port = cfg.gateway?.port || 18800;

    // models
    formData.models.primary = cfg.models?.primary || cfg.models?.default || '';
    formData.models.fallbacks = cfg.models?.fallbacks || [];

    // channels
    const channels = cfg.channels || {};
    formData.channels = channelDefs.map((d) => {
      const c = channels[d.id] || {};
      return {
        id: d.id, label: d.label,
        enabled: c.enabled || false,
        configJson: JSON.stringify(c, null, 2),
      };
    });

    // search
    formData.search.default = cfg.tools?.web?.search?.default || cfg.search?.default || 'searxng';
    formData.search.searxngUrl = cfg.tools?.web?.search?.searxngUrl || cfg.search?.searxngUrl || '';
    formData.search.braveKey = cfg.tools?.web?.search?.braveKey || cfg.search?.braveKey || '';
    formData.search.tavilyKey = cfg.tools?.web?.search?.tavilyKey || cfg.search?.tavilyKey || '';

    // tts
    formData.tts.default = cfg.tts?.default || 'openai';
    formData.tts.openaiVoice = cfg.tts?.openaiVoice || 'alloy';
  } catch (e: any) {
    Message.error(e.message);
  }
}

function addProvider() {
  formData.providers.push({ name: '', apiKey: '', baseUrl: '', model: '', enabled: true, supportsThinking: false });
}

async function saveAll() {
  saving.value = true;
  try {
    // 组装配置
    const providers: Record<string, any> = {};
    for (const p of formData.providers) {
      if (p.name.trim()) {
        providers[p.name] = {
          apiKey: p.apiKey || undefined,
          baseUrl: p.baseUrl || undefined,
          model: p.model || undefined,
          enabled: p.enabled,
          supportsThinking: p.supportsThinking || undefined,
        };
      }
    }
    const channels: Record<string, any> = {};
    for (const ch of formData.channels) {
      try {
        const parsed = JSON.parse(ch.configJson || '{}');
        channels[ch.id] = { ...parsed, enabled: ch.enabled };
      } catch {
        channels[ch.id] = { enabled: ch.enabled };
      }
    }
    const config = {
      gateway: { token: formData.gateway.token, port: formData.gateway.port },
      models: { primary: formData.models.primary, fallbacks: formData.models.fallbacks },
      providers,
      channels,
      tools: { web: { search: { default: formData.search.default, searxngUrl: formData.search.searxngUrl, braveKey: formData.search.braveKey, tavilyKey: formData.search.tavilyKey } } },
      tts: { default: formData.tts.default, openaiVoice: formData.tts.openaiVoice },
    };
    if (activeTab.value === 'json') {
      // JSON 模式：直接保存 JSON
      await rpc.call('config.set', { config: JSON.parse(jsonText.value) });
    } else {
      await rpc.call('config.set', { config });
    }
    Message.success('已保存');
  } catch (e: any) {
    Message.error(e.message);
  } finally {
    saving.value = false;
  }
}

async function loadCustomValue() {
  if (!customPath.value.trim()) { Message.warning('请输入路径'); return; }
  try {
    const res = await rpc.call<{ config: any }>('config.get');
    const cfg = res.config || {};
    const parts = customPath.value.split('.');
    let val: any = cfg;
    for (const p of parts) { val = val?.[p]; }
    customValue.value = JSON.stringify(val, null, 2);
  } catch (e: any) { Message.error(e.message); }
}

async function saveCustomValue() {
  if (!customPath.value.trim() || !customNewValue.value.trim()) { Message.warning('请输入路径和新值'); return; }
  try {
    const res = await rpc.call<{ config: any }>('config.get');
    const cfg = JSON.parse(JSON.stringify(res.config || {}));
    const parts = customPath.value.split('.');
    let cur: any = cfg;
    for (let i = 0; i < parts.length - 1; i++) {
      if (cur[parts[i]] === undefined) cur[parts[i]] = {};
      cur = cur[parts[i]];
    }
    cur[parts[parts.length - 1]] = JSON.parse(customNewValue.value);
    await rpc.call('config.set', { config: cfg });
    Message.success('已保存');
    loadCustomValue();
  } catch (e: any) { Message.error(e.message); }
}

onMounted(loadConfig);
</script>

<style lang="less" scoped>
.provider-card {
  border: 1px solid var(--color-border-1);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
  background: var(--color-bg-2);
}
.provider-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
</style>
