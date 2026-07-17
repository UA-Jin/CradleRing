<template>
  <div class="page-container">
    <a-page-header title="记忆库" subtitle="Cache-First · 向量检索 · 时序知识图谱" :show-back="false">
      <template #extra>
        <a-space>
          <a-button @click="loadStats">
            <template #icon><icon-refresh /></template>
            刷新
          </a-button>
          <a-button type="primary" @click="openCreate">
            <template #icon><icon-plus /></template>
            添加记忆
          </a-button>
        </a-space>
      </template>
    </a-page-header>

    <!-- 统计卡片 -->
    <a-row :gutter="16" class="mt-16">
      <a-col :xs="12" :sm="12" :md="6" v-for="card in statCards" :key="card.title">
        <a-card hoverable>
          <div class="stat-card">
            <div class="stat-icon" :style="{ background: card.bg }">
              <component :is="card.icon" />
            </div>
            <div class="stat-body">
              <div class="stat-value">{{ card.value }}</div>
              <div class="stat-label">{{ card.title }}</div>
            </div>
          </div>
        </a-card>
      </a-col>
    </a-row>

    <!-- 搜索栏（支持语义检索） -->
    <a-card class="mt-16 search-card">
      <a-input-search
        v-model="q"
        :placeholder="semanticAvailable ? '输入查询，按回车进行语义检索...' : '输入关键词搜索...'"
        search-button
        allow-clear
        size="large"
        @search="onSearch"
      >
        <template #button-icon>
          <icon-search v-if="!semanticSearching" />
          <icon-loading v-else />
        </template>
      </a-input-search>
      <div class="search-meta">
        <a-space size="small">
          <a-tag :color="semanticAvailable ? 'green' : 'gray'" bordered>
            {{ semanticAvailable ? '语义检索可用' : '语义检索未启用' }}
          </a-tag>
          <a-tag v-if="searchMode === 'semantic' && lastResults" color="purple" bordered>
            命中 {{ lastResults.length }} 条 · 耗时 {{ searchMs }}ms
          </a-tag>
          <a-tag v-else-if="searchMode === 'keyword'" bordered>
            关键词过滤
          </a-tag>
        </a-space>
      </div>
    </a-card>

    <!-- 类型筛选 -->
    <div class="filter-bar mt-16">
      <a-radio-group v-model="filterKind" type="button" size="small" @change="applyFilter">
        <a-radio value="all">全部</a-radio>
        <a-radio v-for="k in kindOptions" :key="k.value" :value="k.value">{{ k.label }}</a-radio>
      </a-radio-group>
    </div>

    <!-- 记忆列表 -->
    <a-list class="mt-16 memory-list" :data="displayItems" :loading="loading" :pagination="{ pageSize: 12 }">
      <template #item="{ item }">
        <a-list-item>
          <a-list-item-meta>
            <template #avatar>
              <a-avatar :style="{ background: kindColor(item.kind) }" shape="square">
                {{ kindLabel(item.kind).charAt(0) }}
              </a-avatar>
            </template>
            <template #title>
              <div class="mem-title-row">
                <span class="mem-body">{{ item.body }}</span>
                <a-space size="small">
                  <a-tag size="small" :color="kindTagColor(item.kind)">{{ kindLabel(item.kind) }}</a-tag>
                  <a-tag v-if="item.score != null" size="small" color="arcoblue">
                    相似度 {{ (item.score * 100).toFixed(0) }}%
                  </a-tag>
                </a-space>
              </div>
            </template>
            <template #description>
              <div class="mem-desc">
                <span><icon-user-group /> {{ item.source || 'unknown' }}</span>
                <span><icon-clock-circle /> {{ dayjs(item.createdAt).fromNow() }}</span>
                <span v-if="item.tags && item.tags.length"><icon-tag /> {{ item.tags.join(', ') }}</span>
                <span v-if="item.hitCount > 0"><icon-fire /> 命中 {{ item.hitCount }} 次</span>
              </div>
            </template>
          </a-list-item-meta>
          <template #actions>
            <a-space>
              <a-tooltip content="复制内容">
                <a-button size="small" shape="circle" @click="copyText(item.body)"><icon-copy /></a-button>
              </a-tooltip>
              <a-popconfirm content="确认删除？" @ok="onDelete(item.id)">
                <a-button size="small" shape="circle" status="danger"><icon-delete /></a-button>
              </a-popconfirm>
            </a-space>
          </template>
        </a-list-item>
      </template>
      <template #empty>
        <a-empty description="暂无记忆，点击右上角添加" />
      </template>
    </a-list>

    <!-- 添加记忆对话框 -->
    <a-modal :visible="visible" title="添加记忆" @cancel="visible = false" @ok="onSave" :ok-loading="saving" :width="560">
      <a-form :model="form" layout="vertical">
        <a-form-item label="内容" required>
          <a-textarea v-model="form.body" :auto-size="{ minRows: 3, maxRows: 8 }" placeholder="输入要记住的内容..." />
        </a-form-item>
        <a-form-item label="类型">
          <a-select v-model="form.kind">
            <a-option v-for="k in kindOptions" :key="k.value" :value="k.value">{{ k.label }}</a-option>
          </a-select>
        </a-form-item>
        <a-form-item label="标签（逗号分隔）">
          <a-input v-model="form.tagsStr" placeholder="如：运维, 紧急, 客户A" />
        </a-form-item>
        <a-form-item label="来源">
          <a-input v-model="form.source" placeholder="来源标识，如：admin、cron-xxx" />
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, markRaw } from 'vue';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/zh-cn';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import {
  IconBookmark, IconSearch, IconLoading, IconRefresh, IconPlus,
  IconCopy, IconDelete, IconFire, IconTag, IconClockCircle, IconUserGroup,
  IconStorage, IconRelation, IconCheckCircle,
} from '@arco-design/web-vue/es/icon';

dayjs.extend(relativeTime);
dayjs.locale('zh-cn');

// ---------- 类型定义 ----------
interface MemoryItem {
  id: number;
  body: string;
  kind: string;
  source?: string;
  createdAt: string;
  tags?: string[];
  score?: number;
  hitCount?: number;
}

const kindOptions = [
  { value: 'fact', label: '事实', color: '#8c57ff' },
  { value: 'preference', label: '偏好', color: '#16b1ff' },
  { value: 'instruction', label: '指令', color: '#ff4c51' },
  { value: 'procedure', label: '流程', color: '#56ca00' },
  { value: 'entity', label: '实体', color: '#ffb400' },
  { value: 'note', label: '笔记', color: '#6d6777' },
];

const kindLabelMap: Record<string, string> = Object.fromEntries(kindOptions.map((k) => [k.value, k.label]));
const kindColorMap: Record<string, string> = Object.fromEntries(kindOptions.map((k) => [k.value, k.color]));
const kindLabel = (k: string) => kindLabelMap[k] || k;
const kindColor = (k: string) => kindColorMap[k] || '#6d6777';
const kindTagColor = (k: string) => {
  const map: Record<string, string> = {
    fact: 'purple', preference: 'arcoblue', instruction: 'red',
    procedure: 'green', entity: 'orange', note: 'gray',
  };
  return map[k] || 'gray';
};

// ---------- 状态 ----------
const loading = ref(false);
const items = ref<MemoryItem[]>([]);
const q = ref('');
const filterKind = ref('all');
const searchMode = ref<'none' | 'keyword' | 'semantic'>('none');
const lastResults = ref<MemoryItem[] | null>(null);
const searchMs = ref(0);
const semanticAvailable = ref(false);
const semanticSearching = ref(false);

// 统计
const stats = ref({ total: 0, byKind: {} as Record<string, number>, avgHits: 0 });

// 对话框
const visible = ref(false);
const saving = ref(false);
const form = reactive({ body: '', kind: 'fact', tagsStr: '', source: 'admin' });

// ---------- 计算属性 ----------
const displayItems = computed(() => {
  if (searchMode.value === 'semantic' && lastResults.value) return lastResults.value;
  let list = items.value;
  if (filterKind.value !== 'all') list = list.filter((m) => m.kind === filterKind.value);
  if (q.value && searchMode.value !== 'semantic') {
    list = list.filter((m) => m.body.toLowerCase().includes(q.value.toLowerCase()));
  }
  return list;
});

const statCards = computed(() => [
  { title: '记忆总数', value: stats.value.total, icon: markRaw(IconStorage), bg: 'linear-gradient(135deg, #8c57ff, #7340e0)' },
  { title: '事实/知识', value: stats.value.byKind['fact'] || 0, icon: markRaw(IconBookmark), bg: 'linear-gradient(135deg, #56ca00, #82e040)' },
  { title: '关联实体', value: stats.value.byKind['entity'] || 0, icon: markRaw(IconRelation), bg: 'linear-gradient(135deg, #ffb400, #ffd040)' },
  { title: '平均命中', value: stats.value.avgHits, icon: markRaw(IconFire), bg: 'linear-gradient(135deg, #16b1ff, #56d0ff)' },
]);

// ---------- 方法 ----------
async function load() {
  loading.value = true;
  try {
    const res = await rpc.call<{ memories: MemoryItem[] }>('memory.list');
    items.value = res.memories || [];
    searchMode.value = 'none';
    lastResults.value = null;
  } catch (e: any) {
    Message.error(e.message);
  } finally {
    loading.value = false;
  }
}

async function loadStats() {
  try {
    const res = await rpc.call<{ stats: any; semanticAvailable: boolean }>('memory.stats');
    stats.value = res.stats || { total: items.value.length, byKind: {}, avgHits: 0 };
    semanticAvailable.value = !!res.semanticAvailable;
  } catch (e) {
    // 降级：本地聚合
    const byKind: Record<string, number> = {};
    let totalHits = 0;
    items.value.forEach((m) => {
      byKind[m.kind] = (byKind[m.kind] || 0) + 1;
      totalHits += m.hitCount || 0;
    });
    stats.value = {
      total: items.value.length,
      byKind,
      avgHits: items.value.length ? +(totalHits / items.value.length).toFixed(1) : 0,
    };
    semanticAvailable.value = false;
  }
}

async function onSearch(value: string) {
  if (!value || !value.trim()) {
    searchMode.value = 'none';
    lastResults.value = null;
    return;
  }
  // 优先语义检索（若可用）
  if (semanticAvailable.value) {
    semanticSearching.value = true;
    try {
      const t0 = Date.now();
      const res = await rpc.call<{ results: MemoryItem[] }>('memory.search', { query: value, topK: 20 });
      searchMs.value = Date.now() - t0;
      lastResults.value = res.results || [];
      searchMode.value = 'semantic';
    } catch (e: any) {
      Message.warning(`语义检索失败，降级为关键词：${e.message}`);
      searchMode.value = 'keyword';
    } finally {
      semanticSearching.value = false;
    }
  } else {
    searchMode.value = 'keyword';
  }
}

function applyFilter() {
  // 仅本地过滤
}

function openCreate() {
  form.body = '';
  form.kind = 'fact';
  form.tagsStr = '';
  form.source = 'admin';
  visible.value = true;
}

async function onSave() {
  if (!form.body.trim()) {
    Message.warning('请输入内容');
    return;
  }
  saving.value = true;
  try {
    const tags = form.tagsStr.split(',').map((s) => s.trim()).filter(Boolean);
    await rpc.call('memory.save', {
      body: form.body,
      kind: form.kind,
      tags,
      source: form.source || 'admin',
    });
    Message.success('已保存');
    visible.value = false;
    await Promise.all([load(), loadStats()]);
  } catch (e: any) {
    Message.error(e.message);
  } finally {
    saving.value = false;
  }
}

async function onDelete(id: number) {
  try {
    await rpc.call('memory.delete', { id });
    Message.success('已删除');
    await Promise.all([load(), loadStats()]);
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    Message.success('已复制');
  } catch {
    Message.warning('复制失败，请手动选择');
  }
}

onMounted(async () => {
  await Promise.all([load(), loadStats()]);
});
</script>

<style lang="less" scoped>
.stat-card {
  display: flex;
  align-items: center;
  gap: 14px;
  .stat-icon {
    width: 48px;
    height: 48px;
    border-radius: 10px;
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 22px;
    flex-shrink: 0;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }
  .stat-body {
    flex: 1;
    min-width: 0;
  }
  .stat-value {
    font-size: 26px;
    font-weight: 700;
    color: var(--color-text-1);
    line-height: 1.2;
  }
  .stat-label {
    font-size: 12px;
    color: var(--color-text-3);
    margin-top: 2px;
  }
}

.search-card {
  :deep(.arco-card-body) {
    padding: 16px 20px;
  }
}

.search-meta {
  margin-top: 10px;
}

.filter-bar {
  display: flex;
  align-items: center;
}

.mem-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  .mem-body {
    flex: 1;
    min-width: 0;
    font-weight: 500;
    color: var(--color-text-1);
  }
}

.mem-desc {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: var(--color-text-3);
  flex-wrap: wrap;
  span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
}
</style>
