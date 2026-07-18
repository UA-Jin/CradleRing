<template>
  <div class="page-container">
    <a-page-header title="服务管理" subtitle="systemd 服务启停与日志" :show-back="false">
      <template #extra>
        <a-space>
          <a-select v-model="currentNodeId" style="width: 200px" @change="loadServices" size="small">
            <a-option value="local">本机</a-option>
            <a-option v-for="n in nodes" :key="n.id" :value="n.id">{{ n.name }}</a-option>
          </a-select>
          <a-button @click="loadServices"><template #icon><icon-refresh /></template></a-button>
        </a-space>
      </template>
    </a-page-header>

    <a-card class="mt-16">
      <a-table :data="filteredServices" :loading="loading" :pagination="{ pageSize: 20 }" row-key="name">
        <template #columns>
          <a-table-column title="服务" :width="200">
            <template #cell="{ record }">
              <a-link @click="viewLogs(record)">{{ record.name }}</a-link>
            </template>
          </a-table-column>
          <a-table-column title="描述" data-index="description" ellipsis tooltip />
          <a-table-column title="状态" :width="100">
            <template #cell="{ record }">
              <a-badge :status="record.status === 'active' ? 'success' : record.status === 'failed' ? 'error' : 'default'"
                :text="record.status" />
            </template>
          </a-table-column>
          <a-table-column title="PID" :width="80" data-index="pid" />
          <a-table-column title="内存" :width="100">
            <template #cell="{ record }">{{ record.memory || '-' }}</template>
          </a-table-column>
          <a-table-column title="操作" :width="200" fixed="right">
            <template #cell="{ record }">
              <a-space>
                <a-button v-if="record.status !== 'active'" size="small" type="primary" @click="serviceAction(record, 'start')">启动</a-button>
                <a-button v-else size="small" @click="serviceAction(record, 'restart')">重启</a-button>
                <a-button v-if="record.status === 'active'" size="small" status="warning" @click="serviceAction(record, 'stop')">停止</a-button>
                <a-button size="small" @click="viewLogs(record)">日志</a-button>
              </a-space>
            </template>
          </a-table-column>
        </template>
      </a-table>
    </a-card>

    <!-- 日志对话框 -->
    <a-modal :visible="logVisible" :title="`${viewingService?.name} 日志`" @cancel="logVisible = false" :footer="false" :width="800">
      <pre class="log-content">{{ serviceLogs }}</pre>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import { IconRefresh } from '@arco-design/web-vue/es/icon';

const currentNodeId = ref('local');
const nodes = ref<any[]>([]);
const services = ref<any[]>([]);
const loading = ref(false);
const logVisible = ref(false);
const viewingService = ref<any>(null);
const serviceLogs = ref('');
const searchKey = ref('');

const filteredServices = computed(() =>
  services.value.filter((s) => !searchKey.value || s.name.includes(searchKey.value)),
);

async function loadServices() {
  loading.value = true;
  try {
    const res = await rpc.call<any>('services.list', { nodeId: currentNodeId.value });
    services.value = res.services || [];
  } catch (e: any) {
    Message.error(e.message);
  } finally {
    loading.value = false;
  }
}

async function loadNodes() {
  try {
    const res = await rpc.call<any>('nodes.list');
    nodes.value = (res.nodes || []).filter((n: any) => n.status === 'online');
  } catch { /* ignore */ }
}

async function serviceAction(svc: any, action: string) {
  try {
    await rpc.call(`services.${action}`, { name: svc.name, nodeId: currentNodeId.value });
    Message.success(`${svc.name} ${action} 成功`);
    await loadServices();
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function viewLogs(svc: any) {
  viewingService.value = svc;
  try {
    const res = await rpc.call<any>('services.logs', { name: svc.name, lines: 200, nodeId: currentNodeId.value });
    serviceLogs.value = res.logs || '(无日志)';
  } catch (e: any) {
    serviceLogs.value = `读取失败: ${e.message}`;
  }
  logVisible.value = true;
}

onMounted(() => {
  loadNodes();
  loadServices();
});
</script>

<style lang="less" scoped>
.log-content {
  max-height: 500px;
  overflow: auto;
  background: var(--color-bg-3);
  padding: 16px;
  border-radius: 6px;
  font-family: monospace;
  font-size: 11px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
