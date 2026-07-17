<template>
  <div class="page-container">
    <a-page-header title="模型管理" subtitle="LLM 提供商与模型配置" :show-back="false" />
    <a-table class="mt-16" :data="providers" :loading="loading" :pagination="false" row-key="name">
      <template #columns>
        <a-table-column title="提供商" data-index="name" :width="160">
          <template #cell="{ record }"><a-tag :color="record.color">{{ record.label }}</a-tag></template>
        </a-table-column>
        <a-table-column title="Base URL" data-index="baseUrl" :width="300" ellipsis />
        <a-table-column title="模型" data-index="model" :width="200" />
        <a-table-column title="API Key" :width="120">
          <template #cell="{ record }">{{ record.apiKey ? '已配置' : '未配置' }}</template>
        </a-table-column>
        <a-table-column title="状态" :width="100">
          <template #cell="{ record }">
            <a-badge :status="record.apiKey ? 'success' : 'default'" :text="record.apiKey ? '可用' : '未配置'" />
          </template>
        </a-table-column>
      </template>
    </a-table>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { rpc } from '@/api/rpc';

const loading = ref(false);
const providers = ref<any[]>([]);

async function load() {
  loading.value = true;
  try {
    const res = await rpc.call<{ providers: any[] }>('providers.list');
    const colors: Record<string, string> = {
      openai: 'green', anthropic: 'orange', deepseek: 'purple', qwen: 'blue', zhipu: 'red', kimi: 'gray',
    };
    providers.value = (res.providers || []).map((p) => ({
      ...p,
      color: colors[p.name] || 'arcoblue',
      label: p.label || p.name,
    }));
  } catch {
    providers.value = [];
  }
  finally { loading.value = false; }
}

onMounted(load);
</script>
