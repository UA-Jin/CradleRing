<template>
  <div class="page-container">
    <a-page-header title="Sequential 流水线" subtitle="对标 CrewAI：多个角色化 Agent 按顺序接力执行" :show-back="false">
      <template #extra>
        <a-button type="primary" @click="openCreate"><template #icon><icon-plus /></template>新建流水线</a-button>
      </template>
    </a-page-header>

    <a-table class="mt-16" :data="pipelines" :pagination="{ pageSize: 10 }" row-key="id">
      <template #columns>
        <a-table-column title="名称" data-index="name" :width="180">
          <template #cell="{ record }"><a-link @click="openEdit(record)">{{ record.name }}</a-link></template>
        </a-table-column>
        <a-table-column title="阶段数" :width="80">
          <template #cell="{ record }">{{ record.stages?.length || 0 }}</template>
        </a-table-column>
        <a-table-column title="阶段流程" :width="400">
          <template #cell="{ record }">
            <a-steps size="mini">
              <a-step v-for="(s, i) in record.stages" :key="i" :title="roleName(s.agentRoleId)" />
            </a-steps>
          </template>
        </a-table-column>
        <a-table-column title="传递" :width="80">
          <template #cell="{ record }"><a-tag :color="record.passThrough ? 'green' : 'gray'">{{ record.passThrough ? '是' : '否' }}</a-tag></template>
        </a-table-column>
        <a-table-column title="操作" :width="200" fixed="right">
          <template #cell="{ record }">
            <a-space>
              <a-button size="small" type="primary" @click="openRun(record)">运行</a-button>
              <a-button size="small" @click="openEdit(record)">编辑</a-button>
              <a-popconfirm @ok="onDelete(record.id)"><a-button size="small" status="danger">删除</a-button></a-popconfirm>
            </a-space>
          </template>
        </a-table-column>
      </template>
    </a-table>

    <a-drawer :visible="visible" :width="600" @cancel="visible = false" @ok="onSave" :ok-loading="saving">
      <template #title>{{ form.id ? '编辑流水线' : '新建流水线' }}</template>
      <a-form :model="form" layout="vertical">
        <a-form-item label="名称" required><a-input v-model="form.name" /></a-form-item>
        <a-form-item label="描述"><a-input v-model="form.description" /></a-form-item>
        <a-form-item label="前阶段输出传入后阶段"><a-switch v-model="form.passThrough" /></a-form-item>
        <a-divider orientation="left">阶段</a-divider>
        <div v-for="(s, idx) in form.stages" :key="idx" class="stage-card">
          <div class="stage-head">
            <strong>第 {{ idx + 1 }} 阶段</strong>
            <a-space>
              <a-button size="mini" @click="moveStage(idx, -1)" :disabled="idx === 0">↑</a-button>
              <a-button size="mini" @click="moveStage(idx, 1)" :disabled="idx === form.stages.length - 1">↓</a-button>
              <a-button size="mini" status="danger" @click="form.stages.splice(idx, 1)">删</a-button>
            </a-space>
          </div>
          <a-form-item label="角色 Agent">
            <a-select v-model="s.agentRoleId">
              <a-option v-for="r in roles" :key="r.id" :value="r.id">{{ r.name }} ({{ r.role }})</a-option>
            </a-select>
          </a-form-item>
          <a-form-item label="任务模板（支持 ${input} ${prev_output}）">
            <a-textarea v-model="s.taskTemplate" :auto-size="{ minRows: 2 }" />
          </a-form-item>
        </div>
        <a-button long @click="addStage"><template #icon><icon-plus /></template>添加阶段</a-button>
      </a-form>
    </a-drawer>

    <a-modal :visible="runVisible" :title="`运行流水线 - ${runPipeline?.name}`" :width="640" @cancel="runVisible = false" :footer="false">
      <a-space direction="vertical" fill style="width: 100%">
        <a-textarea v-model="runInput" :auto-size="{ minRows: 2 }" placeholder="输入流水线的初始任务..." />
        <a-button type="primary" :loading="running" @click="doRun">运行</a-button>
        <div v-if="runResult">
          <h4>最终输出</h4>
          <pre class="result-pre">{{ runResult.finalOutput }}</pre>
          <a-collapse :default-active-key="[0]" v-for="(out, i) in runResult.stageOutputs" :key="i">
            <a-collapse-item :header="`阶段 ${i + 1} 输出`" :key="i">
              <pre class="result-pre">{{ out }}</pre>
            </a-collapse-item>
          </a-collapse>
        </div>
      </a-space>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';

const pipelines = ref<any[]>([]);
const roles = ref<any[]>([]);
const visible = ref(false);
const saving = ref(false);
const runVisible = ref(false);
const runPipeline = ref<any>(null);
const runInput = ref('');
const runResult = ref<any>(null);
const running = ref(false);

const form = reactive<any>({ id: '', name: '', description: '', passThrough: true, stages: [] });

function roleName(id: string) {
  return roles.value.find((r) => r.id === id)?.name || id;
}

async function load() {
  try {
    const [p, r] = await Promise.all([
      rpc.call<{ pipelines: any[] }>('pipelines.list'),
      rpc.call<{ roles: any[] }>('agent_roles.list'),
    ]);
    pipelines.value = p.pipelines || [];
    roles.value = r.roles || [];
  } catch { /* ignore */ }
}

function openCreate() {
  Object.assign(form, { id: '', name: '', description: '', passThrough: true, stages: [] });
  visible.value = true;
}

function openEdit(p: any) {
  Object.assign(form, { ...p, stages: p.stages.map((s: any) => ({ ...s })) });
  visible.value = true;
}

function addStage() {
  form.stages.push({ order: form.stages.length + 1, agentRoleId: roles.value[0]?.id || '', taskTemplate: '${input}' });
}
function moveStage(idx: number, dir: number) {
  const t = idx + dir;
  if (t < 0 || t >= form.stages.length) return;
  const tmp = form.stages[idx];
  form.stages[idx] = form.stages[t];
  form.stages[t] = tmp;
  form.stages.forEach((s: any, i: number) => (s.order = i + 1));
}

async function onSave() {
  if (!form.name.trim()) { Message.warning('请输入名称'); return; }
  if (!form.stages.length) { Message.warning('至少 1 个阶段'); return; }
  saving.value = true;
  try {
    const payload = { name: form.name, description: form.description, passThrough: form.passThrough, stages: form.stages.map((s: any, i: number) => ({ order: i + 1, agentRoleId: s.agentRoleId, taskTemplate: s.taskTemplate })) };
    if (form.id) await rpc.call('pipelines.update', { id: form.id, ...payload });
    else await rpc.call('pipelines.create', payload);
    Message.success(form.id ? '已更新' : '已创建');
    visible.value = false;
    await load();
  } catch (e: any) { Message.error(e.message); }
  finally { saving.value = false; }
}

async function onDelete(id: string) {
  try { await rpc.call('pipelines.delete', { id }); Message.success('已删除'); await load(); }
  catch (e: any) { Message.error(e.message); }
}

function openRun(p: any) { runPipeline.value = p; runInput.value = ''; runResult.value = null; runVisible.value = true; }

async function doRun() {
  if (!runInput.value.trim()) { Message.warning('请输入任务'); return; }
  running.value = true;
  runResult.value = null;
  try {
    const res = await rpc.call('pipelines.run', { id: runPipeline.value.id, input: runInput.value });
    if (res.ok) runResult.value = res.result;
    else Message.error(res.error?.message || '运行失败');
  } catch (e: any) { Message.error(e.message); }
  finally { running.value = false; }
}

onMounted(async () => {
  await load();
  if (pipelines.value.length === 0 && roles.value.length >= 2) {
    // 创建示例流水线：研究员 → 工程师 → 审阅者
    const researcher = roles.value.find((r) => r.name === '研究员');
    const engineer = roles.value.find((r) => r.name === '工程师');
    const reviewer = roles.value.find((r) => r.name === '审阅者');
    if (researcher && engineer && reviewer) {
      try {
        await rpc.call('pipelines.create', {
          name: '调研-开发-审阅 流水线',
          description: '研究员调研 → 工程师实现 → 审阅者审核',
          passThrough: true,
          stages: [
            { order: 1, agentRoleId: researcher.id, taskTemplate: '请调研以下需求并给出方案：\n${input}' },
            { order: 2, agentRoleId: engineer.id, taskTemplate: '基于以下调研方案，实现代码：\n${prev_output}' },
            { order: 3, agentRoleId: reviewer.id, taskTemplate: '请审阅以下实现，指出问题：\n${prev_output}' },
          ],
        });
        await load();
      } catch { /* ignore */ }
    }
  }
});
</script>

<style lang="less" scoped>
.stage-card { border: 1px solid var(--color-border-1); border-radius: 6px; padding: 12px; margin-bottom: 12px; background: var(--color-bg-2); }
.stage-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.result-pre { background: var(--color-bg-3); padding: 8px; border-radius: 4px; font-size: 12px; white-space: pre-wrap; max-height: 200px; overflow: auto; }
</style>
