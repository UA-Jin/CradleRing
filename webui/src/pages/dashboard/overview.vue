<template>
  <div class="page-container">
    <a-page-header title="概览" subtitle="CradleRing 系统状态总览" :show-back="false" />

    <!-- 状态卡片 -->
    <a-row :gutter="16" class="mt-16">
      <a-col :xs="12" :sm="12" :md="6" v-for="card in cards" :key="card.title">
        <a-card hoverable>
          <a-statistic :title="card.title" :value="card.value" :value-style="{ color: card.color }">
            <template #prefix><component :is="card.icon" /></template>
            <template #suffix v-if="card.suffix">{{ card.suffix }}</template>
          </a-statistic>
          <div class="card-footer">{{ card.foot }}</div>
        </a-card>
      </a-col>
    </a-row>

    <!-- 图表 -->
    <a-row :gutter="16" class="mt-16">
      <a-col :xs="24" :md="16">
        <a-card title="调用趋势（近 7 天）" :bordered="true">
          <v-chart class="chart" :option="trendOption" autoresize />
        </a-card>
      </a-col>
      <a-col :xs="24" :md="8">
        <a-card title="用量分布">
          <v-chart class="chart" :option="distOption" autoresize />
        </a-card>
      </a-col>
    </a-row>

    <!-- 待审批 + 最近会话 -->
    <a-row :gutter="16" class="mt-16">
      <a-col :xs="24" :md="12">
        <a-card title="待处理审批">
          <template #extra><a-link @click="$router.push('/approvals/instances')">查看全部</a-link></template>
          <a-empty v-if="!pendingApprovals.length" description="暂无待审批" />
          <a-list v-else>
            <a-list-item v-for="i in pendingApprovals" :key="i.id">
              <a-list-item-meta :title="i.title" :description="`${i.requestedUsername} · ${i.command.slice(0, 50)}`">
                <template #avatar><a-avatar :style="{ backgroundColor: '#ffb400' }"><icon-clock-circle /></a-avatar></template>
              </a-list-item-meta>
              <template #actions>
                <a-button type="primary" size="small" @click="quickApprove(i.id)">同意</a-button>
                <a-button status="danger" size="small" @click="quickReject(i.id)">拒绝</a-button>
              </template>
            </a-list-item>
          </a-list>
        </a-card>
      </a-col>
      <a-col :xs="24" :md="12">
        <a-card title="最近会话">
          <template #extra><a-link @click="$router.push('/sessions')">查看全部</a-link></template>
          <a-empty v-if="!recentSessions.length" />
          <a-list v-else>
            <a-list-item v-for="s in recentSessions" :key="s.key" @click="$router.push('/chat')">
              <a-list-item-meta :title="s.displayName || s.key" :description="dayjs(s.updatedAt).fromNow()">
                <template #avatar><a-avatar>{{ (s.kind || 'S').charAt(0).toUpperCase() }}</a-avatar></template>
              </a-list-item-meta>
            </a-list-item>
          </a-list>
        </a-card>
      </a-col>
    </a-row>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, markRaw } from 'vue';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { LineChart, PieChart } from 'echarts/charts';
import { GridComponent, TooltipComponent, LegendComponent } from 'echarts/components';
import VChart from 'vue-echarts';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/zh-cn';
import { rpc } from '@/api/rpc';
import { Message } from '@arco-design/web-vue';
import { IconStorage, IconCheckCircle, IconMindMapping, IconTrophy, IconClockCircle } from '@arco-design/web-vue/es/icon';

dayjs.extend(relativeTime);
dayjs.locale('zh-cn');

use([CanvasRenderer, LineChart, PieChart, GridComponent, TooltipComponent, LegendComponent]);

interface Card {
  title: string;
  value: number;
  color: string;
  icon: any;
  suffix?: string;
  foot: string;
}

const cards = ref<Card[]>([]);
const pendingApprovals = ref<any[]>([]);
const recentSessions = ref<any[]>([]);

const trendOption = computed(() => ({
  tooltip: { trigger: 'axis' },
  legend: { data: ['调用次数', '费用($)'], textStyle: { color: '#6d6777' } },
  grid: { left: 40, right: 40, top: 40, bottom: 30 },
  xAxis: { type: 'category', data: ['周一', '周二', '周三', '周四', '周五', '周六', '周日'] },
  yAxis: [
    { type: 'value', name: '次数' },
    { type: 'value', name: '费用' },
  ],
  series: [
    { name: '调用次数', type: 'line', smooth: true, data: [12, 18, 25, 30, 22, 15, 8], itemStyle: { color: '#8c57ff' } },
    { name: '费用($)', type: 'line', smooth: true, yAxisIndex: 1, data: [0.2, 0.5, 1.2, 1.8, 0.9, 0.4, 0.1], itemStyle: { color: '#56ca00' } },
  ],
}));

const distOption = computed(() => ({
  tooltip: { trigger: 'item' },
  legend: { bottom: 0, textStyle: { color: '#6d6777' } },
  series: [{
    type: 'pie',
    radius: ['40%', '70%'],
    data: [
      { value: 40, name: '对话', itemStyle: { color: '#8c57ff' } },
      { value: 25, name: '工具调用', itemStyle: { color: '#56ca00' } },
      { value: 20, name: '搜索', itemStyle: { color: '#ffb400' } },
      { value: 15, name: '其他', itemStyle: { color: '#6d6777' } },
    ],
  }],
}));

async function loadOverview() {
  try {
    const [stats, sessions, approvals] = await Promise.all([
      rpc.call<any>('approval.stats'),
      rpc.call<any>('sessions.list'),
      rpc.call<any>('approval.instances.list', { status: 'pending' }),
    ]);
    cards.value = [
      { title: '总会话数', value: sessions.count || 0, color: '#8c57ff', icon: markRaw(IconStorage), foot: '已创建的对话会话' },
      { title: '待审批', value: stats.pending || 0, color: '#ffb400', icon: markRaw(IconCheckCircle), foot: '等待处理的审批' },
      { title: '审批流模板', value: stats.flowsCount || 0, color: '#56ca00', icon: markRaw(IconMindMapping), foot: '已配置的审批流' },
      { title: '审批完成', value: (stats.approved || 0) + (stats.completed || 0), color: '#7340e0', icon: markRaw(IconTrophy), foot: '累计已完成的审批' },
    ];
    pendingApprovals.value = (approvals.instances || []).slice(0, 5);
    recentSessions.value = (sessions.sessions || []).slice(0, 6);
  } catch (e) {
    // 降级到空数据
    cards.value = [];
  }
}

async function quickApprove(id: string) {
  try {
    await rpc.call('approval.instances.approve', { id, comment: '概览页快速通过' });
    Message.success('已同意');
    loadOverview();
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function quickReject(id: string) {
  try {
    await rpc.call('approval.instances.reject', { id, comment: '概览页拒绝' });
    Message.success('已拒绝');
    loadOverview();
  } catch (e: any) {
    Message.error(e.message);
  }
}

onMounted(loadOverview);
</script>

<style lang="less" scoped>
.chart {
  height: 320px;
}
.card-footer {
  margin-top: 8px;
  font-size: 12px;
  color: var(--color-text-3);
}
</style>
