<template>
  <div class="page-container">
    <a-page-header title="运维审计" subtitle="对标 ongrid：变更事件追踪 · AI SOP 二审 · 命令策略沙箱" :show-back="false">
      <template #extra>
        <a-radio-group v-model="tab" type="button" @change="load">
          <a-radio value="events">变更事件</a-radio>
          <a-radio value="reviews">AI SOP 二审</a-radio>
          <a-radio value="policy">命令策略</a-radio>
        </a-radio-group>
      </template>
    </a-page-header>

    <!-- 变更事件 -->
    <div v-if="tab === 'events'" class="mt-16">
      <a-space class="mb-16">
        <a-select v-model="kindFilter" placeholder="按类型过滤" allow-clear style="width: 200px" @change="load">
          <a-option value="exec_write">命令执行</a-option>
          <a-option value="service_restart">服务重启</a-option>
          <a-option value="config_change">配置变更</a-option>
          <a-option value="exec_denied">拒绝执行</a-option>
        </a-select>
        <a-input v-model="searchText" placeholder="搜索目标..." allow-clear style="width: 240px" />
        <a-button @click="load"><template #icon><icon-refresh /></template>刷新</a-button>
      </a-space>
      <a-table :data="filteredEvents" :pagination="{ pageSize: 20 }" row-key="id">
        <template #columns>
          <a-table-column title="时间" :width="160">
            <template #cell="{ record }">{{ dayjs(record.ts).format('MM-DD HH:mm:ss') }}</template>
          </a-table-column>
          <a-table-column title="类型" :width="100">
            <template #cell="{ record }"><a-tag :color="kindColor(record.kind)">{{ kindLabel(record.kind) }}</a-tag></template>
          </a-table-column>
          <a-table-column title="操作" data-index="action" :width="180" ellipsis />
          <a-table-column title="目标" :width="300" ellipsis tooltip>
            <template #cell="{ record }"><a-typography-text code>{{ record.target }}</a-typography-text></template>
          </a-table-column>
          <a-table-column title="执行者" data-index="actor" :width="100" />
          <a-table-column title="结果" :width="80">
            <template #cell="{ record }">
              <a-badge :status="record.result === 'ok' ? 'success' : record.result === 'denied' ? 'danger' : 'warning'" :text="record.result" />
            </template>
          </a-table-column>
          <a-table-column title="回滚提示" data-index="rollbackHint" :width="200" ellipsis tooltip />
        </template>
      </a-table>
    </div>

    <!-- AI SOP 二审 -->
    <div v-if="tab === 'reviews'" class="mt-16">
      <a-space class="mb-16">
        <a-button @click="load"><template #icon><icon-refresh /></template>刷新</a-button>
        <a-button type="primary" @click="showReviewDialog = true">发起 AI 审查</a-button>
      </a-space>
      <a-table :data="reviews" :pagination="{ pageSize: 20 }" row-key="id">
        <template #columns>
          <a-table-column title="时间" :width="160">
            <template #cell="{ record }">{{ dayjs(record.ts).format('MM-DD HH:mm:ss') }}</template>
          </a-table-column>
          <a-table-column title="决策" :width="100">
            <template #cell="{ record }">
              <a-tag :color="record.decision === 'approve' ? 'green' : 'red'">{{ record.decision === 'approve' ? '✅ 通过' : '❌ 拒绝' }}</a-tag>
            </template>
          </a-table-column>
          <a-table-column title="目标" data-index="target" :width="240" ellipsis tooltip>
            <template #cell="{ record }"><a-typography-text code>{{ record.target }}</a-typography-text></template>
          </a-table-column>
          <a-table-column title="SOP覆盖" :width="80">
            <template #cell="{ record }"><a-badge :status="record.hasSop ? 'success' : 'danger'" /></template>
          </a-table-column>
          <a-table-column title="无并行" :width="80">
            <template #cell="{ record }"><a-badge :status="record.noParallelOp ? 'success' : 'danger'" /></template>
          </a-table-column>
          <a-table-column title="可回滚" :width="80">
            <template #cell="{ record }"><a-badge :status="record.rollbackKnown ? 'success' : 'danger'" /></template>
          </a-table-column>
          <a-table-column title="审查意见" data-index="comment" ellipsis tooltip />
          <a-table-column title="匹配SOP" :width="200" ellipsis tooltip>
            <template #cell="{ record }">{{ record.matchedSop || '-' }}</template>
          </a-table-column>
        </template>
      </a-table>
    </div>

    <!-- 命令策略 -->
    <div v-if="tab === 'policy'" class="mt-16">
      <a-row :gutter="16">
        <a-col :span="24">
          <a-card title="命令策略沙箱（cmdpolicy）">
            <a-space class="mb-16">
              <a-input v-model="testCommand" placeholder="输入命令测试策略..." style="width: 400px" />
              <a-button type="primary" @click="checkPolicy">检查</a-button>
            </a-space>
            <a-alert v-if="policyResult" :type="policyResult.safe ? 'success' : (policyResult.destructive ? 'error' : 'warning')" class="mb-16">
              <div><strong>分类：</strong>{{ policyResult.class }} | <strong>安全：</strong>{{ policyResult.safe ? '是' : '否' }} | <strong>需审批：</strong>{{ policyResult.needsApproval ? '是' : '否' }}</div>
              <div><strong>原因：</strong>{{ policyResult.reason }}</div>
            </a-alert>
            <a-table :data="policyClasses" :pagination="false" row-key="name">
              <template #columns>
                <a-table-column title="分类" data-index="label" :width="180">
                  <template #cell="{ record }"><a-tag :color="classColor(record.name)">{{ record.label }}</a-tag></template>
                </a-table-column>
                <a-table-column title="命令示例" :width="400">
                  <template #cell="{ record }">
                    <a-space wrap><a-tag v-for="e in record.examples" :key="e" size="small">{{ e }}</a-tag></a-space>
                  </template>
                </a-table-column>
              </template>
            </a-table>
            <a-descriptions :column="3" class="mt-16" bordered>
              <a-descriptions-item label="路径白名单">{{ policyInfo.pathAllowlist?.join(', ') || '-' }}</a-descriptions-item>
              <a-descriptions-item label="stdout 上限">{{ (policyInfo.stdoutCap || 0) / 1024 }}KB</a-descriptions-item>
              <a-descriptions-item label="超时">{{ policyInfo.timeoutSecs }}s</a-descriptions-item>
            </a-descriptions>
          </a-card>
        </a-col>
      </a-row>
    </div>

    <!-- AI 审查对话框 -->
    <a-modal :visible="showReviewDialog" title="AI SOP 二审" @cancel="showReviewDialog = false" @ok="doReview" :ok-loading="reviewing">
      <a-form layout="vertical">
        <a-form-item label="操作类型">
          <a-select v-model="reviewForm.action">
            <a-option value="exec">执行命令</a-option>
            <a-option value="service_restart">服务重启</a-option>
            <a-option value="config_change">配置变更</a-option>
          </a-select>
        </a-form-item>
        <a-form-item label="目标（命令/文件/服务）" required>
          <a-input v-model="reviewForm.target" placeholder="如：systemctl restart nginx" />
        </a-form-item>
        <a-form-item label="理由">
          <a-textarea v-model="reviewForm.reason" :auto-size="{ minRows: 2 }" />
        </a-form-item>
        <a-form-item label="影响范围">
          <a-input v-model="reviewForm.blastRadius" placeholder="如：单机 / 单服务 / 集群" />
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import dayjs from 'dayjs';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';

const tab = ref('events');
const kindFilter = ref('');
const searchText = ref('');
const events = ref<any[]>([]);
const reviews = ref<any[]>([]);
const policyClasses = ref<any[]>([]);
const policyInfo = ref<any>({});
const testCommand = ref('');
const policyResult = ref<any>(null);
const showReviewDialog = ref(false);
const reviewing = ref(false);
const reviewForm = reactive({ action: 'exec', target: '', reason: '', blastRadius: '' });

const filteredEvents = computed(() =>
  events.value.filter((e) => !searchText.value || e.target.toLowerCase().includes(searchText.value.toLowerCase())),
);

function kindLabel(k: string) {
  return ({ exec_write: '命令执行', service_restart: '服务重启', config_change: '配置变更', exec_denied: '拒绝执行', approval: '审批' } as any)[k] || k;
}
function kindColor(k: string) {
  return ({ exec_write: 'arcoblue', service_restart: 'orange', config_change: 'purple', exec_denied: 'red', approval: 'green' } as any)[k] || 'gray';
}
function classColor(c: string) {
  return ({ read_fs: 'green', read_system: 'arcoblue', mixed: 'orange', net_diag: 'purple', write: 'orange', destructive: 'red' } as any)[c] || 'gray';
}

async function load() {
  if (tab.value === 'events') {
    try {
      const res = await rpc.call<{ events: any[] }>('change_events.list', { limit: 100, kind: kindFilter.value || undefined });
      events.value = res.events || [];
    } catch { /* ignore */ }
  } else if (tab.value === 'reviews') {
    try {
      const res = await rpc.call<{ reviews: any[] }>('reviews.list', { limit: 100 });
      reviews.value = res.reviews || [];
    } catch { /* ignore */ }
  } else {
    try {
      const res = await rpc.call<any>('cmdpolicy.classes');
      policyClasses.value = res.classes || [];
      policyInfo.value = res;
    } catch { /* ignore */ }
  }
}

async function checkPolicy() {
  if (!testCommand.value.trim()) { Message.warning('请输入命令'); return; }
  try {
    const res = await rpc.call<any>('cmdpolicy.check', { command: testCommand.value });
    policyResult.value = res;
  } catch (e: any) { Message.error(e.message); }
}

async function doReview() {
  if (!reviewForm.target.trim()) { Message.warning('请输入目标'); return; }
  reviewing.value = true;
  try {
    const res = await rpc.call<any>('reviews.review', reviewForm);
    Message.success(`审查完成：${res.decision === 'approve' ? '✅ 通过' : '❌ 拒绝'}`);
    showReviewDialog.value = false;
    await load();
  } catch (e: any) { Message.error(e.message); }
  finally { reviewing.value = false; }
}

onMounted(load);
</script>
