<template>
  <div class="page-container">
    <a-page-header title="角色化 Agent" subtitle="对标 CrewAI：role / goal / backstory / 工具白名单 / 委派" :show-back="false">
      <template #extra>
        <a-space>
          <a-button @click="$router.push('/workflows')">工作流引擎</a-button>
          <a-button type="primary" @click="openCreate"><template #icon><icon-plus /></template>新建角色</a-button>
        </a-space>
      </template>
    </a-page-header>

    <a-row :gutter="16" class="mt-16">
      <a-col v-for="r in roles" :key="r.id" :xs="24" :sm="12" :md="8" :lg="6">
        <a-card hoverable class="role-card">
          <div class="role-head">
            <a-avatar :size="40" :style="{ backgroundColor: avatarColor(r.role) }">{{ r.name.charAt(0) }}</a-avatar>
            <div>
              <div class="role-name">{{ r.name }}</div>
              <div class="role-role">{{ r.role }}</div>
            </div>
          </div>
          <div class="role-goal"><strong>目标：</strong>{{ r.goal || '-' }}</div>
          <div class="role-backstory" v-if="r.backstory"><strong>背景：</strong>{{ r.backstory }}</div>
          <div class="role-tags">
            <a-tag v-for="t in (r.tools || [])" :key="t" size="small">{{ t }}</a-tag>
            <a-tag v-if="!r.tools?.length" size="small" color="gray">全部工具</a-tag>
            <a-tag v-if="r.allowDelegation" size="small" color="green">可委派</a-tag>
            <a-tag size="small" color="arcoblue">{{ r.maxIterations }} 轮</a-tag>
          </div>
          <template #actions>
            <a-button size="small" @click="openTest(r)">测试</a-button>
            <a-button size="small" @click="openEdit(r)">编辑</a-button>
            <a-popconfirm @ok="onDelete(r.id)"><a-button size="small" status="danger">删除</a-button></a-popconfirm>
          </template>
        </a-card>
      </a-col>
    </a-row>

    <!-- 编辑抽屉 -->
    <a-drawer :visible="visible" :width="560" @cancel="visible = false" @ok="onSave" :ok-loading="saving">
      <template #title>{{ form.id ? '编辑角色' : '新建角色' }}</template>
      <a-form :model="form" layout="vertical">
        <a-form-item label="显示名称" required><a-input v-model="form.name" placeholder="如：研究员" /></a-form-item>
        <a-form-item label="角色" required><a-input v-model="form.role" placeholder="如：资深数据分析师" /></a-form-item>
        <a-form-item label="目标"><a-textarea v-model="form.goal" :auto-size="{ minRows: 2 }" placeholder="如：产出高质量的数据分析报告" /></a-form-item>
        <a-form-item label="背景故事"><a-textarea v-model="form.backstory" :auto-size="{ minRows: 3 }" placeholder="增强 agent 人格的背景描述" /></a-form-item>
        <a-row :gutter="8">
          <a-col :span="12"><a-form-item label="模型覆盖"><a-input v-model="form.model" placeholder="留空用默认" /></a-form-item></a-col>
          <a-col :span="6"><a-form-item label="最大轮次"><a-input-number v-model="form.maxIterations" :min="1" :max="30" /></a-form-item></a-col>
          <a-col :span="6"><a-form-item label="允许委派"><a-switch v-model="form.allowDelegation" /></a-form-item></a-col>
        </a-row>
        <a-form-item label="工具白名单（留空=全部；每行一个工具名）">
          <a-textarea v-model="toolsText" :auto-size="{ minRows: 3 }" placeholder="web_search&#10;read_file&#10;write_file" />
        </a-form-item>
        <a-form-item label="自定义 system prompt 模板（可选，支持 ${role} ${goal} ${backstory}）">
          <a-textarea v-model="form.systemPromptTemplate" :auto-size="{ minRows: 3 }" placeholder="留空使用默认模板" />
        </a-form-item>
      </a-form>
    </a-drawer>

    <!-- 测试对话框 -->
    <a-modal :visible="testVisible" :title="`测试角色 - ${testRole?.name}`" :width="640" @cancel="testVisible = false" :footer="false">
      <a-space direction="vertical" fill style="width: 100%">
        <a-textarea v-model="testTask" :auto-size="{ minRows: 2 }" placeholder="输入测试任务..." />
        <a-button type="primary" :loading="testing" @click="runTest">运行</a-button>
        <div v-if="testOutput" class="test-output">
          <pre>{{ testOutput }}</pre>
        </div>
      </a-space>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';

const roles = ref<any[]>([]);
const visible = ref(false);
const saving = ref(false);
const toolsText = ref('');
const testVisible = ref(false);
const testRole = ref<any>(null);
const testTask = ref('');
const testOutput = ref('');
const testing = ref(false);

const form = reactive<any>({
  id: '', name: '', role: '', goal: '', backstory: '',
  tools: [], model: '', maxIterations: 10, allowDelegation: false, systemPromptTemplate: '',
});

function avatarColor(role: string) {
  const colors = ['#8c57ff', '#56ca00', '#ffb400', '#ff4c51', '#7340e0', '#16b1ff'];
  let hash = 0;
  for (const c of role) hash = (hash * 31 + c.charCodeAt(0)) | 0;
  return colors[Math.abs(hash) % colors.length];
}

async function load() {
  try {
    const res = await rpc.call<{ roles: any[] }>('agent_roles.list');
    roles.value = res.roles || [];
  } catch { /* ignore */ }
}

function openCreate() {
  Object.assign(form, { id: '', name: '', role: '', goal: '', backstory: '', tools: [], model: '', maxIterations: 10, allowDelegation: false, systemPromptTemplate: '' });
  toolsText.value = '';
  visible.value = true;
}

function openEdit(r: any) {
  Object.assign(form, { ...r });
  toolsText.value = (r.tools || []).join('\n');
  visible.value = true;
}

async function onSave() {
  if (!form.name.trim() || !form.role.trim()) { Message.warning('请输入名称和角色'); return; }
  saving.value = true;
  try {
    const tools = toolsText.value.split('\n').map((s) => s.trim()).filter(Boolean);
    const payload = { ...form, tools: tools.length ? tools : undefined };
    if (form.id) {
      await rpc.call('agent_roles.update', payload);
    } else {
      await rpc.call('agent_roles.create', payload);
    }
    Message.success(form.id ? '已更新' : '已创建');
    visible.value = false;
    await load();
  } catch (e: any) { Message.error(e.message); }
  finally { saving.value = false; }
}

async function onDelete(id: string) {
  try { await rpc.call('agent_roles.delete', { id }); Message.success('已删除'); await load(); }
  catch (e: any) { Message.error(e.message); }
}

function openTest(r: any) {
  testRole.value = r;
  testTask.value = '';
  testOutput.value = '';
  testVisible.value = true;
}

async function runTest() {
  if (!testTask.value.trim()) { Message.warning('请输入测试任务'); return; }
  testing.value = true;
  testOutput.value = '';
  try {
    const res = await rpc.call('agent_roles.test', { id: testRole.value.id, task: testTask.value });
    if (res.ok) {
      testOutput.value = res.output;
    } else {
      Message.error(res.error?.message || '测试失败');
    }
  } catch (e: any) { Message.error(e.message); }
  finally { testing.value = false; }
}

onMounted(async () => {
  await load();
  // 首次访问若无角色，自动创建几个示例
  if (roles.value.length === 0) {
    await createDemoRoles();
  }
});

async function createDemoRoles() {
  const demos = [
    { name: '研究员', role: '资深研究分析师', goal: '产出深入、准确、有数据支撑的研究报告', backstory: '拥有 10 年行业研究经验，擅长快速拆解复杂问题、多角度分析。' },
    { name: '工程师', role: '资深全栈工程师', goal: '编写高质量、可维护、经过测试的代码', backstory: '精通多种编程语言，注重代码质量、性能和可读性。', tools: ['exec', 'read_file', 'write_file', 'run_code'], allowDelegation: true },
    { name: '审阅者', role: '严谨的技术审阅专家', goal: '发现潜在问题、给出建设性改进意见', backstory: '以挑剔著称，能发现别人忽略的边界情况和安全隐患。', tools: ['read_file', 'web_search'] },
  ];
  for (const d of demos) {
    try { await rpc.call('agent_roles.create', d); } catch { /* ignore */ }
  }
  await load();
}
</script>

<style lang="less" scoped>
.role-card { margin-bottom: 16px; }
.role-head { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
.role-name { font-size: 16px; font-weight: 600; color: var(--color-text-1); }
.role-role { font-size: 13px; color: var(--color-text-3); }
.role-goal { font-size: 13px; color: var(--color-text-2); margin-bottom: 8px; }
.role-backstory { font-size: 12px; color: var(--color-text-3); margin-bottom: 8px; max-height: 60px; overflow: hidden; }
.role-tags { display: flex; flex-wrap: wrap; gap: 4px; }
.test-output { background: var(--color-bg-3); padding: 12px; border-radius: 4px; max-height: 360px; overflow: auto; }
.test-output pre { margin: 0; white-space: pre-wrap; font-size: 13px; }
</style>
