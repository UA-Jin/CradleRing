<template>
  <div class="page-container">
    <a-page-header title="工作流引擎" subtitle="对标 LangGraph：有状态图 + 检查点 + 回放 + interrupt + 并行扇出" :show-back="false">
      <template #extra>
        <a-radio-group v-model="tab" type="button" @change="load">
          <a-radio value="graphs">图模板</a-radio>
          <a-radio value="runs">运行实例</a-radio>
          <a-radio value="trace">执行轨迹</a-radio>
        </a-radio-group>
      </template>
    </a-page-header>

    <!-- 图模板列表 -->
    <div v-if="tab === 'graphs'" class="mt-16">
      <a-space class="mb-16">
        <a-button type="primary" @click="openCreateGraph"><template #icon><icon-plus /></template>新建工作流</a-button>
        <a-button @click="createDemoGraph">生成示例图</a-button>
      </a-space>
      <a-table :data="store.graphs" :loading="store.loading" :pagination="{ pageSize: 10 }" row-key="id">
        <template #columns>
          <a-table-column title="名称" data-index="name" :width="180">
            <template #cell="{ record }"><a-link @click="openEditGraph(record)">{{ record.name }}</a-link></template>
          </a-table-column>
          <a-table-column title="节点数" :width="80">
            <template #cell="{ record }">{{ record.nodes?.length || 0 }}</template>
          </a-table-column>
          <a-table-column title="入口" data-index="entryNode" :width="120" />
          <a-table-column title="描述" data-index="description" ellipsis />
          <a-table-column title="状态" :width="80">
            <template #cell="{ record }"><a-tag :color="record.enabled ? 'green' : 'gray'">{{ record.enabled ? '启用' : '禁用' }}</a-tag></template>
          </a-table-column>
          <a-table-column title="操作" :width="240" fixed="right">
            <template #cell="{ record }">
              <a-space>
                <a-button size="small" type="primary" @click="quickRun(record)">运行</a-button>
                <a-button size="small" @click="openEditGraph(record)">编辑</a-button>
                <a-popconfirm @ok="deleteGraph(record.id)"><a-button size="small" status="danger">删除</a-button></a-popconfirm>
              </a-space>
            </template>
          </a-table-column>
        </template>
      </a-table>
    </div>

    <!-- 运行实例 -->
    <div v-if="tab === 'runs'" class="mt-16">
      <a-button class="mb-16" @click="load"><template #icon><icon-refresh /></template>刷新</a-button>
      <a-table :data="store.runs" :pagination="{ pageSize: 15 }" row-key="id">
        <template #columns>
          <a-table-column title="工作流" data-index="graphName" :width="160" />
          <a-table-column title="状态" :width="120">
            <template #cell="{ record }">
              <a-tag :color="statusColor(record.status)">{{ statusLabel(record.status) }}</a-tag>
            </template>
          </a-table-column>
          <a-table-column title="当前节点" data-index="currentNode" :width="140" />
          <a-table-column title="检查点" :width="80">
            <template #cell="{ record }">{{ record.checkpointsCount }}</template>
          </a-table-column>
          <a-table-column title="开始时间" :width="160">
            <template #cell="{ record }">{{ dayjs(record.startedAt).format('MM-DD HH:mm:ss') }}</template>
          </a-table-column>
          <a-table-column title="操作" :width="280" fixed="right">
            <template #cell="{ record }">
              <a-space>
                <a-button v-if="record.status === 'paused_interrupt'" size="small" type="primary" @click="openResume(record)">恢复</a-button>
                <a-button size="small" @click="openTrace(record)">轨迹</a-button>
                <a-button size="small" @click="openRewind(record)">回滚</a-button>
                <a-button v-if="record.status === 'running' || record.status.startsWith('paused')" size="small" status="danger" @click="cancelRun(record.id)">取消</a-button>
              </a-space>
            </template>
          </a-table-column>
        </template>
      </a-table>
    </div>

    <!-- 执行轨迹 -->
    <div v-if="tab === 'trace'" class="mt-16">
      <a-empty v-if="!traceData" description="请从「运行实例」点击「轨迹」查看" />
      <div v-else>
        <a-card :title="`执行轨迹 - ${traceData.name}`">
          <span slot="extra">耗时 {{ traceData.durationMs || 0 }}ms · 状态 {{ traceData.status }}</span>
          <trace-tree :span="traceData" :depth="0" />
        </a-card>
      </div>
    </div>

    <!-- 图编辑抽屉 -->
    <a-drawer :visible="graphVisible" :width="720" @cancel="graphVisible = false" @ok="saveGraph" :ok-loading="saving">
      <template #title>{{ graphForm.id ? '编辑工作流' : '新建工作流' }}</template>
      <a-form :model="graphForm" layout="vertical">
        <a-form-item label="名称" required><a-input v-model="graphForm.name" placeholder="如：调研分析流水线" /></a-form-item>
        <a-form-item label="描述"><a-input v-model="graphForm.description" /></a-form-item>
        <a-form-item label="入口节点 id"><a-input v-model="graphForm.entryNode" placeholder="start" /></a-form-item>
        <a-form-item label="状态字段（逗号分隔）"><a-input v-model="stateSchemaStr" placeholder="topic,result" /></a-form-item>
        <a-divider orientation="left">节点列表</a-divider>
        <div v-for="(n, idx) in graphForm.nodes" :key="idx" class="node-card">
          <div class="node-head">
            <strong>{{ n.name || n.id }}</strong>
            <a-space>
              <a-button size="mini" @click="moveNode(idx, -1)" :disabled="idx === 0">↑</a-button>
              <a-button size="mini" @click="moveNode(idx, 1)" :disabled="idx === graphForm.nodes.length - 1">↓</a-button>
              <a-button size="mini" status="danger" @click="graphForm.nodes.splice(idx, 1)">删</a-button>
            </a-space>
          </div>
          <a-row :gutter="8">
            <a-col :span="8"><a-form-item label="节点 ID"><a-input v-model="n.id" /></a-form-item></a-col>
            <a-col :span="8"><a-form-item label="名称"><a-input v-model="n.name" /></a-form-item></a-col>
            <a-col :span="8">
              <a-form-item label="类型">
                <a-select v-model="n.nodeType">
                  <a-option value="llm">LLM 调用</a-option>
                  <a-option value="tool">工具</a-option>
                  <a-option value="agent">角色化 Agent</a-option>
                  <a-option value="condition">条件分支</a-option>
                  <a-option value="parallel">并行扇出</a-option>
                  <a-option value="interrupt">暂停(interrupt)</a-option>
                  <a-option value="end">终止</a-option>
                </a-select>
              </a-form-item>
            </a-col>
          </a-row>
          <a-form-item v-if="n.nodeType === 'llm' || n.nodeType === 'agent'" label="提示模板（支持 ${state.x}）">
            <a-textarea v-model="n.promptTemplate" :auto-size="{ minRows: 2 }" />
          </a-form-item>
          <a-form-item v-if="n.nodeType === 'agent'" label="角色 Agent">
            <a-select v-model="n.agentRole" allow-clear placeholder="选择角色">
              <a-option v-for="r in store.roles" :key="r.id" :value="r.id">{{ r.name }} ({{ r.role }})</a-option>
            </a-select>
          </a-form-item>
          <a-row v-if="n.nodeType === 'tool'" :gutter="8">
            <a-col :span="8"><a-form-item label="工具名"><a-input v-model="n.toolName" placeholder="web_search" /></a-form-item></a-col>
            <a-col :span="16"><a-form-item label="参数(JSON模板)"><a-textarea v-model="n.toolArgsStr" :auto-size="{ minRows: 1 }" placeholder='{"query":"${vars.topic}"}' /></a-form-item></a-col>
          </a-row>
          <a-row v-if="n.nodeType === 'parallel'" :gutter="8">
            <a-col :span="8"><a-form-item label="扇出字段"><a-input v-model="n.fanOutField" placeholder="input" /></a-form-item></a-col>
            <a-col :span="8"><a-form-item label="角色"><a-select v-model="n.fanOutRole" allow-clear><a-option v-for="r in store.roles" :key="r.id" :value="r.id">{{ r.name }}</a-option></a-select></a-form-item></a-col>
            <a-col :span="4"><a-form-item label="并发"><a-input-number v-model="n.maxConcurrent" :min="1" :max="20" /></a-form-item></a-col>
            <a-col :span="4"><a-form-item label="归并"><a-select v-model="n.reduceMode"><a-option value="concat">拼接</a-option><a-option value="join">分隔</a-option><a-option value="summary">摘要</a-option></a-select></a-form-item></a-col>
          </a-row>
          <a-row v-if="n.nodeType === 'condition'" :gutter="8">
            <a-col :span="16"><a-form-item label="条件分支(JSON)"><a-textarea v-model="n.branchesStr" :auto-size="{ minRows: 2 }" placeholder='[{"expr":"${vars.score} > 80","target":"pass"}]' /></a-form-item></a-col>
            <a-col :span="8"><a-form-item label="默认出口"><a-input v-model="n.defaultEdge" placeholder="fail" /></a-form-item></a-col>
          </a-row>
          <a-form-item label="输出字段（写入 state 的 key）"><a-input v-model="n.outputField" placeholder="output" /></a-form-item>
        </div>
        <a-button long @click="addNode"><template #icon><icon-plus /></template>添加节点</a-button>
        <a-divider orientation="left">边列表（from → to）</a-divider>
        <div v-for="(e, idx) in graphForm.edges" :key="'e'+idx" class="edge-row">
          <a-input v-model="e.from" placeholder="from" style="width:120px" />
          <span>→</span>
          <a-input v-model="e.to" placeholder="to" style="width:120px" />
          <a-input v-model="e.condition" placeholder="条件(可选)" style="width:200px" />
          <a-button size="small" status="danger" @click="graphForm.edges.splice(idx, 1)">删</a-button>
        </div>
        <a-button long class="mt-8" @click="addEdge"><template #icon><icon-plus /></template>添加边</a-button>
      </a-form>
    </a-drawer>

    <!-- 恢复对话框 -->
    <a-modal :visible="resumeVisible" title="恢复工作流" @cancel="resumeVisible = false" @ok="doResume">
      <a-form layout="vertical">
        <a-form-item label="人工输入（写入 state.human_input）">
          <a-textarea v-model="resumeInput" :auto-size="{ minRows: 3 }" placeholder="输入补充信息..." />
        </a-form-item>
      </a-form>
    </a-modal>

    <!-- 回滚对话框 -->
    <a-modal :visible="rewindVisible" title="回滚到检查点" @cancel="rewindVisible = false" @ok="doRewind">
      <a-form layout="vertical">
        <a-form-item label="检查点序号">
          <a-input-number v-model="rewindIndex" :min="0" :max="rewindMax" style="width: 200px" />
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, h, onMounted, defineComponent } from 'vue';
import dayjs from 'dayjs';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import { useWorkflowStore } from '@/stores/workflow';

const store = useWorkflowStore();
const tab = ref('graphs');
const graphVisible = ref(false);
const saving = ref(false);
const resumeVisible = ref(false);
const rewindVisible = ref(false);
const resumeInput = ref('');
const resumeId = ref('');
const rewindId = ref('');
const rewindIndex = ref(0);
const rewindMax = ref(0);
const traceData = ref<any>(null);
const stateSchemaStr = ref('');

interface GraphForm {
  id?: string; name: string; description: string; entryNode: string;
  nodes: any[]; edges: any[];
}
const graphForm = reactive<GraphForm>({ name: '', description: '', entryNode: 'start', nodes: [], edges: [] });

// 轨迹树组件（递归渲染 span）
const TraceTree = defineComponent({
  name: 'TraceTree',
  props: { span: Object, depth: Number },
  setup(props) {
    return () => {
      const s = props.span;
      if (!s) return null;
      const color = s.status === 'error' ? 'red' : s.status === 'running' ? 'orange' : 'green';
      const children = (s.children || []).map((c: any, i: number) =>
        h(TraceTree, { span: c, depth: props.depth + 1, key: i }),
      );
      return h('div', { class: 'trace-node', style: { marginLeft: (props.depth * 20) + 'px' } }, [
        h('div', { class: 'trace-line' }, [
          h('a-tag', { color }, `${s.kind}: ${s.name}`),
          h('span', { class: 'trace-meta' }, ` ${s.durationMs || 0}ms ${s.tokensIn ? '· in:' + s.tokensIn : ''} ${s.tokensOut ? ' out:' + s.tokensOut : ''} ${s.costUsd ? ' $' + s.costUsd.toFixed(4) : ''}`),
          s.error ? h('span', { class: 'trace-err' }, ' ❌ ' + s.error) : null,
        ]),
        children.length ? h('div', children) : null,
      ]);
    };
  },
});

async function load() {
  if (tab.value === 'graphs') await store.loadGraphs();
  else if (tab.value === 'runs') await store.loadRuns();
  if (store.roles.length === 0) await store.loadRoles();
}

function statusLabel(s: string) {
  return ({ running: '运行中', paused_interrupt: '已暂停(等待输入)', paused_review: '已暂停(等待审核)', completed: '已完成', failed: '失败', cancelled: '已取消' } as any)[s] || s;
}
function statusColor(s: string) {
  return ({ running: 'arcoblue', paused_interrupt: 'orange', paused_review: 'orange', completed: 'green', failed: 'red', cancelled: 'gray' } as any)[s] || 'gray';
}

function openCreateGraph() {
  Object.assign(graphForm, { id: undefined, name: '', description: '', entryNode: 'start', nodes: [], edges: [] });
  stateSchemaStr.value = '';
  graphVisible.value = true;
}

function openEditGraph(g: any) {
  graphForm.id = g.id;
  graphForm.name = g.name;
  graphForm.description = g.description;
  graphForm.entryNode = g.entryNode;
  graphForm.nodes = (g.nodes || []).map((n: any) => ({
    ...n,
    toolArgsStr: n.toolArgsTemplate ? JSON.stringify(n.toolArgsTemplate) : '',
    branchesStr: n.branches ? JSON.stringify(n.branches) : '',
  }));
  graphForm.edges = (g.edges || []).map((e: any) => ({ ...e }));
  stateSchemaStr.value = (g.stateSchema || []).join(',');
  graphVisible.value = true;
}

function addNode() {
  graphForm.nodes.push({ id: `node${graphForm.nodes.length + 1}`, name: '', nodeType: 'llm', outputField: 'output' });
}
function addEdge() {
  graphForm.edges.push({ from: '', to: '', condition: '' });
}
function moveNode(idx: number, dir: number) {
  const t = idx + dir;
  if (t < 0 || t >= graphForm.nodes.length) return;
  const tmp = graphForm.nodes[idx];
  graphForm.nodes[idx] = graphForm.nodes[t];
  graphForm.nodes[t] = tmp;
}

async function saveGraph() {
  if (!graphForm.name.trim()) { Message.warning('请输入名称'); return; }
  saving.value = true;
  try {
    const nodes = graphForm.nodes.map((n) => {
      const out: any = { id: n.id, name: n.name, nodeType: n.nodeType, outputField: n.outputField };
      if (n.promptTemplate) out.promptTemplate = n.promptTemplate;
      if (n.agentRole) out.agentRole = n.agentRole;
      if (n.toolName) out.toolName = n.toolName;
      if (n.toolArgsStr) { try { out.toolArgsTemplate = JSON.parse(n.toolArgsStr); } catch { /* ignore */ } }
      if (n.fanOutField) out.fanOutField = n.fanOutField;
      if (n.fanOutRole) out.fanOutRole = n.fanOutRole;
      if (n.maxConcurrent) out.maxConcurrent = n.maxConcurrent;
      if (n.reduceMode) out.reduceMode = n.reduceMode;
      if (n.branchesStr) { try { out.branches = JSON.parse(n.branchesStr); } catch { /* ignore */ } }
      if (n.defaultEdge) out.defaultEdge = n.defaultEdge;
      return out;
    });
    const edges = graphForm.edges.filter((e) => e.from && e.to).map((e, i) => ({ id: `e${i}`, from: e.from, to: e.to, condition: e.condition || undefined }));
    const payload: any = {
      name: graphForm.name, description: graphForm.description, entryNode: graphForm.entryNode,
      nodes, edges, stateSchema: stateSchemaStr.value.split(',').map((s) => s.trim()).filter(Boolean),
    };
    if (graphForm.id) {
      await rpc.call('workflow.graphs.update', { id: graphForm.id, ...payload });
    } else {
      await rpc.call('workflow.graphs.create', payload);
    }
    Message.success(graphForm.id ? '已更新' : '已创建');
    graphVisible.value = false;
    await store.loadGraphs();
  } catch (e: any) { Message.error(e.message); }
  finally { saving.value = false; }
}

async function deleteGraph(id: string) {
  try { await rpc.call('workflow.graphs.delete', { id }); Message.success('已删除'); await store.loadGraphs(); }
  catch (e: any) { Message.error(e.message); }
}

async function quickRun(g: any) {
  const input = prompt(`输入工作流「${g.name}」的初始输入：`, '');
  if (input === null) return;
  try {
    const res = await rpc.call('workflow.runs.start', { graphId: g.id, input, sessionKey: 'workflow' });
    if (res.ok) { Message.success(`已启动: ${res.runId}`); tab.value = 'runs'; await load(); }
    else Message.error(res.error?.message || '启动失败');
  } catch (e: any) { Message.error(e.message); }
}

async function openResume(r: any) { resumeId.value = r.id; resumeInput.value = ''; resumeVisible.value = true; }
async function doResume() {
  try { await rpc.call('workflow.runs.resume', { id: resumeId.value, input: resumeInput.value }); Message.success('已恢复'); resumeVisible.value = false; await load(); }
  catch (e: any) { Message.error(e.message); }
}

async function openRewind(r: any) {
  rewindId.value = r.id; rewindIndex.value = 0;
  try {
    const res = await rpc.call('workflow.runs.checkpoints', { id: r.id });
    rewindMax.value = Math.max(0, (res.checkpoints?.length || 1) - 1);
  } catch { rewindMax.value = 0; }
  rewindVisible.value = true;
}
async function doRewind() {
  try { await rpc.call('workflow.runs.rewind', { id: rewindId.value, checkpointIndex: rewindIndex.value }); Message.success('已回滚'); rewindVisible.value = false; await load(); }
  catch (e: any) { Message.error(e.message); }
}

async function openTrace(r: any) {
  try {
    const res = await rpc.call('workflow.runs.trace', { id: r.id });
    traceData.value = res.trace;
    tab.value = 'trace';
  } catch (e: any) { Message.error(e.message); }
}

async function cancelRun(id: string) {
  try { await rpc.call('workflow.runs.cancel', { id }); Message.success('已取消'); await load(); }
  catch (e: any) { Message.error(e.message); }
}

// 生成示例图：LLM 调研 → 条件分支 → (高分并行深入 / 低分重做) → End
async function createDemoGraph() {
  try {
    await rpc.call('workflow.graphs.create', {
      name: '调研分析示例',
      description: 'LLM 调研 → 条件分支 → 并行深入 → 汇总 → End',
      entryNode: 'research',
      stateSchema: ['topic', 'score'],
      nodes: [
        { id: 'research', name: '初步调研', nodeType: 'llm', promptTemplate: '请调研主题：${input}，给出简要分析。', outputField: 'vars.research' },
        { id: 'judge', name: '评分判断', nodeType: 'llm', promptTemplate: '基于以下调研给出 0-100 质量分数，只输出数字：\n${vars.research}', outputField: 'vars.score' },
        { id: 'branch', name: '分数分支', nodeType: 'condition', branches: [{ expr: '${vars.score} > 70', target: 'deepen' }], defaultEdge: 'redo' },
        { id: 'deepen', name: '并行深入', nodeType: 'parallel', fanOutField: 'input', maxConcurrent: 3, reduceMode: 'summary', outputField: 'output' },
        { id: 'redo', name: '重新调研', nodeType: 'llm', promptTemplate: '之前的调研质量不够，请重新调研：${input}', outputField: 'vars.research' },
        { id: 'end', name: '完成', nodeType: 'end' },
      ],
      edges: [
        { id: 'e1', from: 'research', to: 'judge' },
        { id: 'e2', from: 'judge', to: 'branch' },
        { id: 'e3', from: 'deepen', to: 'end' },
        { id: 'e4', from: 'redo', to: 'judge' },
      ],
    });
    Message.success('已创建示例工作流');
    await store.loadGraphs();
  } catch (e: any) { Message.error(e.message); }
}

onMounted(load);
</script>

<style lang="less" scoped>
.node-card { border: 1px solid var(--color-border-1); border-radius: 6px; padding: 12px; margin-bottom: 12px; background: var(--color-bg-2); }
.node-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.edge-row { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
:deep(.trace-node) { margin-bottom: 4px; }
:deep(.trace-line) { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; }
:deep(.trace-meta) { font-size: 12px; color: var(--color-text-3); }
:deep(.trace-err) { font-size: 12px; color: var(--color-danger); }
</style>
