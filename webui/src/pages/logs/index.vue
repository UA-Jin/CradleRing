<template>
  <div class="page-container">
    <a-page-header title="系统日志" subtitle="事件流与用量日志" :show-back="false">
      <template #extra>
        <a-radio-group v-model="tab" type="button" @change="load">
          <a-radio value="events">事件</a-radio>
          <a-radio value="usage">用量</a-radio>
        </a-radio-group>
      </template>
    </a-page-header>
    <a-table class="mt-16" :data="rows" :loading="loading" :pagination="{ pageSize: 30 }" row-key="ts">
      <template #columns>
        <template v-if="tab === 'events'">
          <a-table-column title="事件" data-index="event" :width="220">
            <template #cell="{ record }"><a-tag>{{ record.event }}</a-tag></template>
          </a-table-column>
          <a-table-column title="数据">
            <template #cell="{ record }"><pre class="json-text">{{ JSON.stringify(record.payload || record, null, 2) }}</pre></template>
          </a-table-column>
          <a-table-column title="时间" :width="160">
            <template #cell="{ record }">{{ dayjs(record.ts || record.timestamp).format('MM-DD HH:mm:ss') }}</template>
          </a-table-column>
        </template>
        <template v-else>
          <a-table-column title="Provider" data-index="provider" :width="120" />
          <a-table-column title="模型" data-index="model" :width="200" />
          <a-table-column title="输入 token" data-index="promptTokens" :width="120" />
          <a-table-column title="输出 token" data-index="completionTokens" :width="120" />
          <a-table-column title="费用($)" data-index="costUsd" :width="100" />
          <a-table-column title="时间" :width="160">
            <template #cell="{ record }">{{ dayjs(record.ts).format('MM-DD HH:mm:ss') }}</template>
          </a-table-column>
        </template>
      </template>
    </a-table>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import dayjs from 'dayjs';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';

const tab = ref('events');
const loading = ref(false);
const rows = ref<any[]>([]);

async function load() {
  loading.value = true;
  try {
    if (tab.value === 'events') {
      const res = await rpc.call<{ events: any[] }>('gateway.events', { limit: 100 });
      rows.value = (res.events || []).reverse();
    } else {
      const res = await rpc.call<{ logs: any[] }>('usage.logs', { limit: 100 });
      rows.value = (res.logs || []).reverse();
    }
  } catch (e: any) { Message.error(e.message); rows.value = []; }
  finally { loading.value = false; }
}

onMounted(load);
</script>

<style lang="less" scoped>
.json-text {
  font-family: monospace;
  font-size: 12px;
  background-color: var(--color-bg-3);
  padding: 6px;
  border-radius: 4px;
  margin: 0;
  max-height: 120px;
  overflow: auto;
}
</style>
