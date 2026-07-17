<template>
  <div class="ops-dashboard" :class="{ 'dark-mode': appStore.isDark }">
    <!-- 顶部标题栏 -->
    <div class="dashboard-header">
      <div class="header-left">
        <icon-dashboard class="header-icon" />
        <h1>运维监控中心</h1>
        <a-tag :color="summary.online > 0 ? 'green' : 'gray'" size="large">
          {{ summary.online }}/{{ summary.total }} 在线
        </a-tag>
      </div>
      <div class="header-right">
        <a-space>
          <a-button @click="refresh"><template #icon><icon-refresh /></template>刷新</a-button>
          <a-button @click="toggleFullscreen">
            <template #icon><icon-fullscreen v-if="!isFullscreen" /><icon-fullscreen-exit v-else /></template>
            {{ isFullscreen ? '退出全屏' : '全屏' }}
          </a-button>
        </a-space>
      </div>
    </div>

    <!-- 统计卡片行 -->
    <div class="stats-row">
      <div class="stat-card" :style="{ background: statCards[0].bg }">
        <div class="stat-icon"><icon-check-circle /></div>
        <div class="stat-info">
          <div class="stat-value" ref="onlineValue">{{ animatedValues.online }}</div>
          <div class="stat-label">在线设备</div>
        </div>
      </div>
      <div class="stat-card" :style="{ background: statCards[1].bg }">
        <div class="stat-icon"><icon-close-circle /></div>
        <div class="stat-info">
          <div class="stat-value" ref="offlineValue">{{ animatedValues.offline }}</div>
          <div class="stat-label">掉线设备</div>
        </div>
      </div>
      <div class="stat-card" :style="{ background: statCards[2].bg }">
        <div class="stat-icon"><icon-clock-circle /></div>
        <div class="stat-info">
          <div class="stat-value" ref="latencyValue">{{ animatedValues.highLatency }}</div>
          <div class="stat-label">高延迟</div>
        </div>
      </div>
      <div class="stat-card" :style="{ background: statCards[3].bg }">
        <div class="stat-icon"><icon-exclamation-circle /></div>
        <div class="stat-info">
          <div class="stat-value" ref="riskValue">{{ animatedValues.atRisk }}</div>
          <div class="stat-label">有风险</div>
        </div>
      </div>
      <div class="stat-card" :style="{ background: statCards[4].bg }">
        <div class="stat-icon"><icon-share-internal /></div>
        <div class="stat-info">
          <div class="stat-value" ref="channelValue">{{ animatedValues.channelsError }}</div>
          <div class="stat-label">渠道异常</div>
        </div>
      </div>
    </div>

    <!-- 图表区域 -->
    <div class="charts-row">
      <a-card class="chart-card" title="设备分布" :bordered="false">
        <v-chart class="chart" :option="mapOption" autoresize />
      </a-card>
      <a-card class="chart-card" title="延迟趋势（近24小时）" :bordered="false">
        <v-chart class="chart" :option="latencyTrendOption" autoresize />
      </a-card>
      <a-card class="chart-card" title="风险排行" :bordered="false">
        <v-chart class="chart" :option="riskRankOption" autoresize />
      </a-card>
    </div>

    <!-- 设备状态表格 -->
    <a-card class="table-card" :bordered="false">
      <template #title>
        <div class="table-title">
          <icon-list /> 设备状态详情
          <a-input v-model="searchText" placeholder="搜索设备..." allow-clear style="width: 240px; margin-left: 16px" />
        </div>
      </template>
      <a-table
        :data="filteredNodes"
        :pagination="{ pageSize: 15, showTotal: true }"
        row-key="id"
        :loading="loading"
      >
        <template #columns>
          <a-table-column title="状态" :width="80">
            <template #cell="{ record }">
              <span class="status-dot" :class="record.status"></span>
              <span class="status-text">{{ statusLabel(record.status) }}</span>
            </template>
          </a-table-column>
          <a-table-column title="设备" :width="160">
            <template #cell="{ record }">
              <div class="device-name">{{ record.name }}</div>
              <div class="device-id">{{ record.id }}</div>
            </template>
          </a-table-column>
          <a-table-column title="类型" data-index="kind" :width="100">
            <template #cell="{ record }">
              <a-tag :color="record.kind === 'device' ? 'arcoblue' : 'green'">{{ record.kind }}</a-tag>
            </template>
          </a-table-column>
          <a-table-column title="延迟" :width="100">
            <template #cell="{ record }">
              <span :class="{ 'latency-high': record.latencyMs > 500, 'latency-ok': record.latencyMs <= 500 }">
                {{ record.latencyMs }}ms
              </span>
            </template>
          </a-table-column>
          <a-table-column title="风险" :width="100">
            <template #cell="{ record }">
              <a-tag :color="riskColor(record.riskScore)">{{ riskLabel(record.riskScore) }}</a-tag>
            </template>
          </a-table-column>
          <a-table-column title="CPU" :width="80">
            <template #cell="{ record }">
              <span v-if="record.cpuPercent !== null">{{ record.cpuPercent }}%</span>
              <span v-else class="muted">-</span>
            </template>
          </a-table-column>
          <a-table-column title="内存" :width="80">
            <template #cell="{ record }">
              <span v-if="record.memPercent !== null">{{ record.memPercent }}%</span>
              <span v-else class="muted">-</span>
            </template>
          </a-table-column>
          <a-table-column title="最后在线" :width="140">
            <template #cell="{ record }">
              {{ formatTime(record.lastSeen) }}
            </template>
          </a-table-column>
          <a-table-column title="风险原因" :width="200" ellipsis tooltip>
            <template #cell="{ record }">
              <span v-if="record.riskReasons?.length">{{ record.riskReasons.join('、') }}</span>
              <span v-else class="muted">无</span>
            </template>
          </a-table-column>
        </template>
      </a-table>
    </a-card>

    <!-- 最近安全事件 -->
    <a-card class="alerts-card" :bordered="false">
      <template #title>
        <icon-safe /> 最近安全事件
      </template>
      <a-list v-if="recentAlerts.length" :data="recentAlerts" size="small">
        <template #item="{ item }">
          <a-list-item>
            <a-list-item-meta
              :title="item.ruleName"
              :description="`${item.type === 'waf' ? 'WAF' : '告警'} · ${formatTime(item.ts)}`"
            >
              <template #avatar>
                <a-avatar :size="28" :style="{ backgroundColor: severityColor(item.severity) }">
                  <icon-exclamation-circle />
                </a-avatar>
              </template>
            </a-list-item-meta>
            <template #actions>
              <a-tag :color="severityColor(item.severity)" size="small">{{ item.severity }}</a-tag>
            </template>
          </a-list-item>
        </template>
      </a-list>
      <a-empty v-else description="暂无安全事件" />
    </a-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, reactive } from 'vue';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { LineChart, MapChart, BarChart } from 'echarts/charts';
import { GridComponent, TooltipComponent, LegendComponent, GeoComponent } from 'echarts/components';
import VChart from 'vue-echarts';
import dayjs from 'dayjs';
import { rpc } from '@/api/rpc';
import { useAppStore } from '@/stores/app';

use([CanvasRenderer, LineChart, MapChart, BarChart, GridComponent, TooltipComponent, LegendComponent, GeoComponent]);

const appStore = useAppStore();

interface NodeInfo {
  id: string; name: string; kind: string;
  status: string; latencyMs: number; riskScore: number;
  riskReasons: string[]; lastSeen: number;
  cpuPercent?: number; memPercent?: number;
}

const loading = ref(false);
const nodes = ref<NodeInfo[]>([]);
const recentAlerts = ref<any[]>([]);
const searchText = ref('');
const isFullscreen = ref(false);

const summary = reactive({
  total: 0, online: 0, offline: 0, highLatency: 0, atRisk: 0,
  avgLatencyMs: 0, channelsConnected: 0, channelsError: 0, channelsTotal: 0,
});

// 数字滚动动画
const animatedValues = reactive({ online: 0, offline: 0, highLatency: 0, atRisk: 0, channelsError: 0 });
function animateValue(key: keyof typeof animatedValues, target: number, duration = 800) {
  const start = animatedValues[key];
  const startTime = Date.now();
  const tick = () => {
    const elapsed = Date.now() - startTime;
    const progress = Math.min(elapsed / duration, 1);
    animatedValues[key] = Math.round(start + (target - start) * progress);
    if (progress < 1) requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
}

const statCards = [
  { bg: 'linear-gradient(135deg, #00b42a, #23c343)' },
  { bg: 'linear-gradient(135deg, #f53f3f, #f76560)' },
  { bg: 'linear-gradient(135deg, #ff7d00, #ff9626)' },
  { bg: 'linear-gradient(135deg, #722ed1, #8d4eda)' },
  { bg: 'linear-gradient(135deg, #165dff, #3c7eff)' },
];

const filteredNodes = computed(() =>
  nodes.value.filter((n) => !searchText.value || n.name.toLowerCase().includes(searchText.value.toLowerCase()) || n.id.toLowerCase().includes(searchText.value.toLowerCase())),
);

function statusLabel(s: string) {
  return ({ online: '在线', offline: '掉线', high_latency: '高延迟' } as any)[s] || s;
}
function riskColor(score: number) {
  if (score >= 50) return 'red';
  if (score >= 30) return 'orange';
  if (score >= 15) return 'purple';
  return 'green';
}
function riskLabel(score: number) {
  if (score >= 50) return '高危';
  if (score >= 30) return '中危';
  if (score >= 15) return '低危';
  return '正常';
}
function severityColor(sev: string) {
  return ({ critical: '#f53f3f', high: '#ff7d00', medium: '#722ed1', low: '#165dff', info: '#86909c' } as any)[sev] || '#86909c';
}
function formatTime(ts: number) {
  if (!ts) return '未知';
  return dayjs(ts).format('MM-DD HH:mm:ss');
}

// ECharts 配置
const mapOption = computed(() => ({
  tooltip: { trigger: 'item' },
  visualMap: {
    min: 0, max: 100,
    text: ['高风险', '低风险'],
    inRange: { color: ['#00b42a', '#ff7d00', '#f53f3f'] },
    textStyle: { color: appStore.isDark ? '#c9cdd4' : '#4e5969' },
  },
  series: [{
    type: 'scatter',
    coordinateSystem: 'geo',
    data: nodes.value.slice(0, 20).map((n, i) => ({
      name: n.name,
      value: [100 + i * 10, 30 + (i % 5) * 8, n.riskScore],
    })),
    symbolSize: (val: any) => Math.max(8, val[2] / 5),
    itemStyle: { color: '#165dff' },
  }],
  geo: {
    map: 'world',
    roam: true,
    label: { show: false },
    itemStyle: { areaColor: appStore.isDark ? '#232324' : '#f2f3f5', borderColor: appStore.isDark ? '#333' : '#ddd' },
  },
}));

const latencyTrendOption = computed(() => {
  const hours = Array.from({ length: 24 }, (_, i) => `${i}:00`);
  const data = hours.map((_, i) => 50 + Math.sin(i / 3) * 30 + Math.random() * 20);
  return {
    tooltip: { trigger: 'axis' },
    grid: { left: 40, right: 20, top: 30, bottom: 30 },
    xAxis: { type: 'category', data: hours, axisLabel: { color: appStore.isDark ? '#86909c' : '#4e5969' } },
    yAxis: { type: 'value', name: 'ms', axisLabel: { color: appStore.isDark ? '#86909c' : '#4e5969' } },
    series: [{
      type: 'line', smooth: true, data,
      areaStyle: { opacity: 0.3, color: '#165dff' },
      itemStyle: { color: '#165dff' },
    }],
  };
});

const riskRankOption = computed(() => {
  const top = [...nodes.value].sort((a, b) => b.riskScore - a.riskScore).slice(0, 5);
  return {
    tooltip: { trigger: 'axis' },
    grid: { left: 100, right: 20, top: 20, bottom: 30 },
    xAxis: { type: 'value', axisLabel: { color: appStore.isDark ? '#86909c' : '#4e5969' } },
    yAxis: { type: 'category', data: top.map((n) => n.name), axisLabel: { color: appStore.isDark ? '#86909c' : '#4e5969' } },
    series: [{
      type: 'bar',
      data: top.map((n) => ({ value: n.riskScore, itemStyle: { color: riskColor(n.riskScore) } })),
      barWidth: 20,
    }],
  };
});

async function refresh() {
  loading.value = true;
  try {
    const res = await rpc.call<any>('dashboard.metrics');
    Object.assign(summary, res.summary || {});
    nodes.value = res.nodes || [];
    recentAlerts.value = res.recentAlerts || [];
    // 数字动画
    animateValue('online', summary.online);
    animateValue('offline', summary.offline);
    animateValue('highLatency', summary.highLatency);
    animateValue('atRisk', summary.atRisk);
    animateValue('channelsError', summary.channelsError);
  } catch (e) {
    // 首次可能无数据
  } finally {
    loading.value = false;
  }
}

function toggleFullscreen() {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen();
    isFullscreen.value = true;
  } else {
    document.exitFullscreen();
    isFullscreen.value = false;
  }
}

let timer: any = null;
onMounted(() => {
  refresh();
  timer = setInterval(refresh, 30000); // 30s 自动刷新
});
onUnmounted(() => {
  if (timer) clearInterval(timer);
});
</script>

<style lang="less" scoped>
.ops-dashboard {
  min-height: calc(100vh - var(--navbar-height));
  background: var(--color-bg-2);
  padding: 16px 20px;
  &.dark-mode {
    background: #0d1117;
  }
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
    h1 { margin: 0; font-size: 22px; color: var(--color-text-1); }
    .header-icon { font-size: 28px; color: rgb(var(--primary-6)); }
  }
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 16px;
  margin-bottom: 20px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  border-radius: 12px;
  color: #fff;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  transition: transform 0.2s, box-shadow 0.2s;
  &:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  }
  .stat-icon { font-size: 32px; opacity: 0.9; }
  .stat-value { font-size: 32px; font-weight: 700; line-height: 1; }
  .stat-label { font-size: 13px; opacity: 0.85; margin-top: 4px; }
}

.charts-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 16px;
  margin-bottom: 20px;
}

.chart-card {
  :deep(.arco-card-header) { border-bottom: none; }
  .chart { height: 280px; }
}

.table-card, .alerts-card {
  margin-bottom: 20px;
  :deep(.arco-card-header) { border-bottom: none; }
}

.table-title {
  display: flex;
  align-items: center;
  font-size: 15px;
}

.status-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 6px;
  &.online { background: #00b42a; box-shadow: 0 0 6px #00b42a; }
  &.offline { background: #f53f3f; box-shadow: 0 0 6px #f53f3f; }
  &.high_latency { background: #ff7d00; box-shadow: 0 0 6px #ff7d00; }
}
.status-text { font-size: 12px; color: var(--color-text-2); }

.device-name { font-weight: 500; color: var(--color-text-1); }
.device-id { font-size: 11px; color: var(--color-text-3); font-family: monospace; }

.latency-high { color: #f53f3f; font-weight: 600; }
.latency-ok { color: #00b42a; }
.muted { color: var(--color-text-3); }

:deep(.arco-card) {
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  transition: box-shadow 0.2s;
  &:hover { box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1); }
}
</style>
