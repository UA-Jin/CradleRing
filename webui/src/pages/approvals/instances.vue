<template>
  <div class="page-container">
    <a-page-header title="审批实例" subtitle="处理待审批请求，支持多级审批工作流" :show-back="false">
      <template #extra>
        <a-radio-group v-model="statusFilter" type="button" @change="load">
          <a-radio value="pending">待审批</a-radio>
          <a-radio value="approved">已批准</a-radio>
          <a-radio value="rejected">已拒绝</a-radio>
          <a-radio value="history">历史</a-radio>
          <a-radio value="">全部</a-radio>
        </a-radio-group>
      </template>
    </a-page-header>

    <!-- 统计 -->
    <a-row :gutter="12" class="mt-16">
      <a-col :xs="12" :md="4" v-for="s in statCards" :key="s.key">
        <a-card>
          <a-statistic :title="s.label" :value="stats[s.key] || 0" :value-style="{ color: s.color }" />
        </a-card>
      </a-col>
    </a-row>

    <a-table
      class="mt-16"
      :data="store.instances"
      :loading="store.loading"
      :pagination="{ pageSize: 15, showTotal: true, showPageSize: true }"
      row-key="id"
      :scroll="{ x: 1200 }"
    >
      <template #columns>
        <a-table-column title="标题" data-index="title" :width="200">
          <template #cell="{ record }">
            <a-link @click="showDetail(record)">{{ record.title }}</a-link>
            <a-tag v-if="record.asyncNonBlocking" size="small" color="arcoblue">异步</a-tag>
          </template>
        </a-table-column>
        <a-table-column title="审批流" data-index="flowName" :width="140" />
        <a-table-column title="命令" :width="280" ellipsis tooltip>
          <template #cell="{ record }">
            <a-typography-text code>{{ record.command }}</a-typography-text>
          </template>
        </a-table-column>
        <a-table-column title="进度" :width="120">
          <template #cell="{ record }">
            <a-space>
              <span>{{ record.currentStep }}/{{ record.totalSteps }}</span>
              <a-progress
                v-if="record.totalSteps"
                :percent="record.currentStep / record.totalSteps"
                size="mini"
                :status="progressStatus(record.status)"
              />
            </a-space>
          </template>
        </a-table-column>
        <a-table-column title="状态" :width="100">
          <template #cell="{ record }">
            <a-tag :color="statusColor(record.status)">{{ statusLabel(record.status) }}</a-tag>
          </template>
        </a-table-column>
        <a-table-column title="请求人" data-index="requestedUsername" :width="100" />
        <a-table-column title="创建时间" :width="160">
          <template #cell="{ record }">{{ dayjs(record.createdAt).format('MM-DD HH:mm:ss') }}</template>
        </a-table-column>
        <a-table-column title="操作" :width="200" fixed="right">
          <template #cell="{ record }">
            <a-space>
              <a-button v-if="record.status === 'pending'" type="primary" size="small" @click="onApprove(record)">
                同意
              </a-button>
              <a-button v-if="record.status === 'pending'" status="danger" size="small" @click="onReject(record)">
                拒绝
              </a-button>
              <a-button size="small" @click="showDetail(record)">详情</a-button>
            </a-space>
          </template>
        </a-table-column>
      </template>
    </a-table>

    <!-- 详情抽屉 -->
    <a-drawer :visible="detailVisible" :width="560" @cancel="detailVisible = false" :footer="false">
      <template #title>审批详情</template>
      <div v-if="current" class="detail-body">
        <a-descriptions :column="1" bordered>
          <a-descriptions-item label="标题">{{ current.title }}</a-descriptions-item>
          <a-descriptions-item label="审批流">{{ current.flowName }}</a-descriptions-item>
          <a-descriptions-item label="状态">
            <a-tag :color="statusColor(current.status)">{{ statusLabel(current.status) }}</a-tag>
          </a-descriptions-item>
          <a-descriptions-item label="请求人">{{ current.requestedUsername }}</a-descriptions-item>
          <a-descriptions-item label="描述">{{ current.description || '-' }}</a-descriptions-item>
          <a-descriptions-item label="命令">
            <pre class="cmd-text">{{ current.command }}</pre>
          </a-descriptions-item>
          <a-descriptions-item label="创建时间">{{ dayjs(current.createdAt).format('YYYY-MM-DD HH:mm:ss') }}</a-descriptions-item>
          <a-descriptions-item v-if="current.completedAt" label="完成时间">{{ dayjs(current.completedAt).format('YYYY-MM-DD HH:mm:ss') }}</a-descriptions-item>
        </a-descriptions>

        <h4 class="mt-16">审批步骤进度</h4>
        <a-steps :current="current.currentStep - 1" :status="stepStatus(current.status)" direction="vertical">
          <a-step v-for="i in current.totalSteps" :key="i" :title="`第 ${i} 步`" :description="stepDesc(i)" />
        </a-steps>

        <h4 class="mt-16">决策历史</h4>
        <a-timeline v-if="current.decisions?.length">
          <a-timeline-item v-for="(d, idx) in current.decisions" :key="idx">
            <div class="decision-item">
              <a-tag :color="d.decision === 'approve' ? 'green' : 'red'">
                {{ d.decision === 'approve' ? '同意' : '拒绝' }}
              </a-tag>
              <strong>{{ d.approverUsername }}</strong>
              <span class="dec-step">第 {{ d.stepOrder }} 步</span>
              <a-tag size="small">{{ d.viaChannel }}</a-tag>
              <div class="dec-time">{{ dayjs(d.decidedAt).format('YYYY-MM-DD HH:mm:ss') }}</div>
              <div v-if="d.comment" class="dec-comment">{{ d.comment }}</div>
            </div>
          </a-timeline-item>
        </a-timeline>
        <a-empty v-else description="暂无决策记录" />

        <div v-if="current.executionResult" class="mt-16">
          <h4>执行结果</h4>
          <pre class="cmd-text">{{ current.executionResult }}</pre>
        </div>

        <div v-if="current.status === 'pending'" class="mt-16 action-row">
          <a-textarea v-model="comment" placeholder="审批意见（可选）" :auto-size="{ minRows: 2 }" />
          <a-space class="mt-8">
            <a-button type="primary" @click="onApprove(current)">同意</a-button>
            <a-button status="danger" @click="onReject(current)">拒绝</a-button>
          </a-space>
        </div>
      </div>
    </a-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import dayjs from 'dayjs';
import { Message } from '@arco-design/web-vue';
import { useApprovalStore, type ApprovalInstance } from '@/stores/approval';

const store = useApprovalStore();

const statusFilter = ref('pending');
const detailVisible = ref(false);
const current = ref<ApprovalInstance | null>(null);
const comment = ref('');

const stats = ref<any>({});
const statCards = [
  { key: 'total', label: '总数', color: '#165dff' },
  { key: 'pending', label: '待审批', color: '#ff7d00' },
  { key: 'approved', label: '已批准', color: '#00b42a' },
  { key: 'rejected', label: '已拒绝', color: '#f53f3f' },
  { key: 'timeout', label: '超时', color: '#86909c' },
  { key: 'completed', label: '已完成', color: '#722ed1' },
];

async function load() {
  await store.loadInstances(statusFilter.value || undefined);
  try {
    stats.value = await (await import('@/api/rpc')).rpc.call('approval.stats');
  } catch { /* ignore */ }
}

function statusLabel(s: string) {
  return { pending: '待审批', approved: '已批准', rejected: '已拒绝', timeout: '超时', executing: '执行中', completed: '已完成', failed: '失败', cancelled: '已取消' }[s] || s;
}
function statusColor(s: string) {
  return { pending: 'orange', approved: 'green', rejected: 'red', timeout: 'gray', executing: 'arcoblue', completed: 'purple', failed: 'red', cancelled: 'gray' }[s] || 'gray';
}
function progressStatus(s: string) {
  return ({ approved: 'success', rejected: 'danger', timeout: 'warning' } as any)[s] || undefined;
}
function stepStatus(s: string) {
  return ({ approved: 'finish', rejected: 'error', timeout: 'warning', completed: 'finish' } as any)[s] || 'process';
}

function stepDesc(step: number) {
  if (!current.value) return '';
  const ds = current.value.decisions.filter((d) => d.stepOrder === step);
  if (!ds.length) return step < current.value.currentStep ? '已通过' : '等待审批';
  return ds.map((d) => `${d.approverUsername} ${d.decision === 'approve' ? '同意' : '拒绝'}`).join('、');
}

function showDetail(r: ApprovalInstance) {
  current.value = r;
  comment.value = '';
  detailVisible.value = true;
}

async function onApprove(r: ApprovalInstance) {
  try {
    await store.approveInstance(r.id, comment.value);
    Message.success('已同意');
    detailVisible.value = false;
    await load();
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function onReject(r: ApprovalInstance) {
  try {
    await store.rejectInstance(r.id, comment.value || '拒绝');
    Message.success('已拒绝');
    detailVisible.value = false;
    await load();
  } catch (e: any) {
    Message.error(e.message);
  }
}

onMounted(load);
</script>

<style lang="less" scoped>
.cmd-text {
  background-color: var(--color-bg-3);
  padding: 8px 12px;
  border-radius: 4px;
  font-family: 'Menlo', 'Monaco', monospace;
  font-size: 12px;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}
.decision-item {
  .dec-step { margin: 0 8px; color: var(--color-text-3); font-size: 12px; }
  .dec-time { font-size: 12px; color: var(--color-text-3); margin-top: 4px; }
  .dec-comment { margin-top: 4px; color: var(--color-text-2); }
}
.action-row { padding-top: 12px; border-top: 1px solid var(--color-border-1); }
</style>
