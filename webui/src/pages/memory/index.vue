<template>
  <div class="page-container">
    <a-page-header title="记忆库" subtitle="Cache-First · 向量检索 · 时序知识图谱" :show-back="false">
      <template #extra>
        <a-space>
          <a-button @click="loadStats">
            <template #icon><icon-refresh /></template>
            刷新
          </a-button>
          <a-button @click="openImport">
            <template #icon><icon-upload /></template>
            导入数据集
          </a-button>
          <a-button v-if="v2Count > 0" status="warning" @click="openMigrate">
            <template #icon><icon-swap /></template>
            迁移旧版数据 ({{ v2Count }})
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

    <!-- 导入数据集对话框 -->
    <a-modal :visible="importVisible" title="导入数据集" @cancel="importVisible = false" @ok="onImport" :ok-loading="importing" :width="680">
      <a-form :model="importForm" layout="vertical">
        <a-form-item label="数据格式">
          <a-radio-group v-model="importForm.format" type="button">
            <a-radio value="json">JSON 数组</a-radio>
            <a-radio value="jsonl">JSONL</a-radio>
            <a-radio value="csv">CSV</a-radio>
            <a-radio value="txt">纯文本</a-radio>
          </a-radio-group>
        </a-form-item>

        <a-form-item>
          <template #label>
            <span>数据内容</span>
            <a-tooltip content="也可以直接粘贴文本，或选择文件自动填充">
              <icon-question-circle class="ml-8" />
            </a-tooltip>
          </template>
          <a-upload
            :auto-upload="false"
            :show-file-list="false"
            accept=".json,.jsonl,.csv,.txt"
            @change="onImportFile"
          >
            <template #upload-button>
              <a-button size="small" class="mb-8">
                <template #icon><icon-folder-add /></template>
                选择文件（自动读取到下方）
              </a-button>
            </template>
          </a-upload>
          <a-textarea
            v-model="importForm.data"
            :auto-size="{ minRows: 10, maxRows: 16 }"
            :placeholder="importPlaceholder"
            style="font-family: monospace; font-size: 12px"
          />
        </a-form-item>

        <a-row :gutter="12">
          <a-col :span="8">
            <a-form-item label="默认类型">
              <a-select v-model="importForm.defaultKind">
                <a-option v-for="k in kindOptions" :key="k.value" :value="k.value">{{ k.label }}</a-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="默认来源">
              <a-input v-model="importForm.defaultSource" placeholder="import" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="默认标签（逗号分隔）">
              <a-input v-model="importForm.defaultTagsStr" placeholder="可选" />
            </a-form-item>
          </a-col>
        </a-row>

        <!-- 导入结果 -->
        <a-alert v-if="importResult" :type="importResult.ok ? 'success' : 'error'" class="mt-8">
          <template v-if="importResult.ok">
            导入完成：成功 {{ importResult.imported }} 条
            <template v-if="importResult.failed">，失败 {{ importResult.failed }} 条</template>
            <template v-if="importResult.parseErrors">，解析错误 {{ importResult.parseErrors }} 条</template>
          </template>
          <template v-else>
            导入失败：{{ importResult.error }}
          </template>
        </a-alert>
      </a-form>
    </a-modal>

    <!-- 迁移旧版数据对话框 -->
    <a-modal :visible="migrateVisible" title="迁移旧版数据到 V3" @cancel="migrateVisible = false" @ok="onMigrate" :ok-loading="migrating" :width="520">
      <a-alert type="info" class="mb-16">
        检测到旧版记忆库（V2）有 <b>{{ v2Count }}</b> 条数据。迁移会把它们导入 V3 向量库（生成向量 + 图谱），旧数据保留不动。
      </a-alert>
      <a-form layout="vertical">
        <a-form-item>
          <a-checkbox v-model="migrateOverwrite">
            覆盖模式（不跳过 V3 已存在的相同内容）
          </a-checkbox>
          <div class="migrate-hint">默认跳过 V3 已有的相同内容，避免重复</div>
        </a-form-item>
      </a-form>
      <a-alert v-if="migrateResult" :type="migrateResult.ok ? 'success' : 'error'" class="mt-8">
        <template v-if="migrateResult.ok">
          迁移完成：导入 {{ migrateResult.migrated }} 条，跳过 {{ migrateResult.skipped }} 条
          <template v-if="migrateResult.failed">，失败 {{ migrateResult.failed }} 条</template>
        </template>
        <template v-else>
          迁移失败：{{ migrateResult.error }}
        </template>
      </a-alert>
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
  IconStorage, IconRelation, IconCheckCircle, IconUpload, IconSwap,
  IconFolderAdd, IconQuestionCircle,
} from '@arco-design/web-vue/es/icon';

dayjs.extend(relativeTime);
dayjs.locale('zh-cn');

// ---------- 类型定义 ----------
interface MemoryItem {
  id: string | number;  // V3 是字符串 id，V2 遗留是数字 id
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

// ---------- 导入数据集 ----------
const importVisible = ref(false);
const importing = ref(false);
const importResult = ref<any>(null);
const importForm = reactive({
  format: 'json',
  data: '',
  defaultKind: 'fact',
  defaultSource: 'import',
  defaultTagsStr: '',
});

const importPlaceholder = computed(() => {
  switch (importForm.format) {
    case 'json':
      return '[\n  { "body": "记忆内容", "kind": "fact", "source": "import", "tags": ["标签"] },\n  ...\n]';
    case 'jsonl':
      return '{"body": "每行一个 JSON 对象", "kind": "fact"}\n{"body": "第二条", "kind": "preference"}';
    case 'csv':
      return 'body,kind,source,tags\n"记忆内容,含逗号",fact,import,标签1;标签2';
    default:
      return '每行一条记忆...\n第二行是第二条记忆...';
  }
});

// ---------- 迁移旧版 ----------
const migrateVisible = ref(false);
const migrating = ref(false);
const migrateOverwrite = ref(false);
const migrateResult = ref<any>(null);
const v2Count = ref(0);

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

async function onDelete(id: string | number) {
  try {
    await rpc.call('memory.delete', { id: String(id) });
    Message.success('已删除');
    await Promise.all([load(), loadStats(), loadV2Count()]);
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

// ---------- 导入数据集方法 ----------
function openImport() {
  importForm.data = '';
  importResult.value = null;
  importVisible.value = true;
}

function onImportFile(_fileList: any, fileItem: any) {
  const file = fileItem?.file;
  if (!file) return;
  const reader = new FileReader();
  reader.onload = (e) => {
    importForm.data = String(e.target?.result || '');
    // 按扩展名自动识别格式
    const name = (file.name || '').toLowerCase();
    if (name.endsWith('.jsonl')) importForm.format = 'jsonl';
    else if (name.endsWith('.csv')) importForm.format = 'csv';
    else if (name.endsWith('.txt')) importForm.format = 'txt';
    else importForm.format = 'json';
    Message.success(`已读取 ${file.name}（${(file.size / 1024).toFixed(1)}KB），格式识别为 ${importForm.format.toUpperCase()}`);
  };
  reader.readAsText(file);
}

async function onImport() {
  if (!importForm.data.trim()) {
    Message.warning('请先粘贴数据或选择文件');
    return;
  }
  importing.value = true;
  importResult.value = null;
  try {
    const defaultTags = importForm.defaultTagsStr
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean);
    const res = await rpc.call<any>('memory2.import', {
      format: importForm.format,
      data: importForm.data,
      defaultKind: importForm.defaultKind,
      defaultSource: importForm.defaultSource || 'import',
      defaultTags,
    });
    importResult.value = res;
    if (res.ok) {
      Message.success(`成功导入 ${res.imported} 条记忆`);
      await Promise.all([load(), loadStats()]);
    }
  } catch (e: any) {
    importResult.value = { ok: false, error: e.message };
  } finally {
    importing.value = false;
  }
}

// ---------- 迁移旧版数据方法 ----------
function openMigrate() {
  migrateResult.value = null;
  migrateOverwrite.value = false;
  migrateVisible.value = true;
}

async function onMigrate() {
  migrating.value = true;
  migrateResult.value = null;
  try {
    const res = await rpc.call<any>('memory2.migrate_v2', { overwrite: migrateOverwrite.value });
    migrateResult.value = res;
    if (res.ok) {
      Message.success(`迁移完成：${res.migrated} 条`);
      await Promise.all([load(), loadStats(), loadV2Count()]);
    }
  } catch (e: any) {
    migrateResult.value = { ok: false, error: e.message };
  } finally {
    migrating.value = false;
  }
}

async function loadV2Count() {
  try {
    const res = await rpc.call<any>('memory.stats');
    v2Count.value = res.stats?.v2_total || 0;
  } catch {
    v2Count.value = 0;
  }
}

onMounted(async () => {
  await Promise.all([load(), loadStats(), loadV2Count()]);
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

.migrate-hint {
  font-size: 12px;
  color: var(--color-text-4);
  margin-top: 4px;
}

.mb-8 { margin-bottom: 8px; }
.mb-16 { margin-bottom: 16px; }
.ml-8 { margin-left: 8px; }
</style>
