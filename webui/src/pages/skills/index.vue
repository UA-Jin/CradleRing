<template>
  <div class="page-container">
    <a-page-header title="技能" subtitle="Agent 技能与插件" :show-back="false" />
    <a-table class="mt-16" :data="skills" :loading="loading" :pagination="{ pageSize: 15 }" row-key="id">
      <template #columns>
        <a-table-column title="ID" data-index="id" :width="160" />
        <a-table-column title="名称" data-index="name" :width="180" />
        <a-table-column title="描述" data-index="description" ellipsis />
        <a-table-column title="分类" data-index="category" :width="120">
          <template #cell="{ record }"><a-tag>{{ record.category }}</a-tag></template>
        </a-table-column>
        <a-table-column title="状态" :width="100">
          <template #cell="{ record }">
            <a-switch v-model="record.enabled" @change="toggle(record)" />
          </template>
        </a-table-column>
      </template>
    </a-table>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';

const loading = ref(false);
const skills = ref<any[]>([]);

async function load() {
  loading.value = true;
  try {
    const res = await rpc.call<{ plugins: any[] }>('plugins.list');
    skills.value = (res.plugins || []).map((p) => ({ ...p, enabled: p.enabled !== false }));
  } catch (e: any) { Message.error(e.message); }
  finally { loading.value = false; }
}

async function toggle(p: any) {
  try { await rpc.call('plugins.toggle', { id: p.id, enabled: p.enabled }); } catch (e: any) { Message.error(e.message); }
}

onMounted(load);
</script>
