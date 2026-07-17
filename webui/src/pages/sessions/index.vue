<template>
  <div class="page-container">
    <a-page-header title="会话管理" subtitle="所有对话会话" :show-back="false">
      <template #extra>
        <a-input v-model="q" placeholder="搜索会话..." allow-clear style="width: 220px" />
      </template>
    </a-page-header>
    <a-table class="mt-16" :data="filtered" :loading="loading" :pagination="{ pageSize: 15 }" row-key="key">
      <template #columns>
        <a-table-column title="会话" :width="220">
          <template #cell="{ record }">
            <a-link @click="$router.push({ path: '/chat', query: { key: record.key } })">{{ record.displayName || record.key }}</a-link>
          </template>
        </a-table-column>
        <a-table-column title="类型" data-index="kind" :width="120">
          <template #cell="{ record }"><a-tag>{{ record.kind }}</a-tag></template>
        </a-table-column>
        <a-table-column title="渠道" :width="120">
          <template #cell="{ record }">{{ record.channel || '-' }}</template>
        </a-table-column>
        <a-table-column title="模型" :width="160">
          <template #cell="{ record }">{{ record.model || '-' }}</template>
        </a-table-column>
        <a-table-column title="更新时间" :width="160">
          <template #cell="{ record }">{{ dayjs(record.updatedAt).format('MM-DD HH:mm:ss') }}</template>
        </a-table-column>
        <a-table-column title="操作" :width="160" fixed="right">
          <template #cell="{ record }">
            <a-space>
              <a-button size="small" @click="$router.push({ path: '/chat', query: { key: record.key } })">打开</a-button>
              <a-popconfirm content="确认删除？" @ok="onDelete(record.key)">
                <a-button size="small" status="danger">删除</a-button>
              </a-popconfirm>
            </a-space>
          </template>
        </a-table-column>
      </template>
    </a-table>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import dayjs from 'dayjs';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';

const loading = ref(false);
const sessions = ref<any[]>([]);
const q = ref('');
const filtered = computed(() =>
  sessions.value.filter((s) =>
    !q.value || (s.displayName || s.key).toLowerCase().includes(q.value.toLowerCase()),
  ),
);

async function load() {
  loading.value = true;
  try {
    const res = await rpc.call<{ sessions: any[] }>('sessions.list');
    sessions.value = res.sessions || [];
  } catch (e: any) {
    Message.error(e.message);
  } finally { loading.value = false; }
}

async function onDelete(key: string) {
  try {
    await rpc.call('sessions.delete', { sessionKey: key });
    Message.success('已删除');
    await load();
  } catch (e: any) { Message.error(e.message); }
}

onMounted(load);
</script>
