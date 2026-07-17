<template>
  <div class="page-container">
    <a-page-header title="审批流模板" subtitle="配置多级审批工作流：主管 → 领导 → 执行，支持 IM 渠道通知" :show-back="false">
      <template #extra>
        <a-button type="primary" @click="openCreate">
          <template #icon><icon-plus /></template>
          新建审批流
        </a-button>
      </template>
    </a-page-header>

    <a-table
      class="mt-16"
      :data="store.flows"
      :loading="store.loading"
      :pagination="{ pageSize: 10 }"
      row-key="id"
    >
      <template #columns>
        <a-table-column title="名称" data-index="name" :width="180">
          <template #cell="{ record }">
            <a-link @click="openEdit(record)">{{ record.name }}</a-link>
          </template>
        </a-table-column>
        <a-table-column title="适用工具" :width="120">
          <template #cell="{ record }">
            <a-tag v-for="k in record.kinds" :key="k" size="small" color="arcoblue">{{ k }}</a-tag>
          </template>
        </a-table-column>
        <a-table-column title="触发关键词" :width="200" ellipsis tooltip>
          <template #cell="{ record }">
            <a-tag v-for="p in record.triggerPatterns" :key="p" size="small">{{ p }}</a-tag>
            <span v-if="!record.triggerPatterns.length" class="muted">所有命令</span>
          </template>
        </a-table-column>
        <a-table-column title="步骤" :width="280">
          <template #cell="{ record }">
            <a-steps :current="record.steps.length" size="mini">
              <a-step v-for="(s, i) in record.steps" :key="i" :title="`第${s.order}步:${s.name || s.approverRole}`" />
            </a-steps>
          </template>
        </a-table-column>
        <a-table-column title="状态" :width="80">
          <template #cell="{ record }">
            <a-tag :color="record.enabled ? 'green' : 'gray'">{{ record.enabled ? '启用' : '禁用' }}</a-tag>
          </template>
        </a-table-column>
        <a-table-column title="操作" :width="160" fixed="right">
          <template #cell="{ record }">
            <a-space>
              <a-button size="small" @click="openEdit(record)">编辑</a-button>
              <a-popconfirm content="确认删除该审批流？" @ok="onDelete(record.id)">
                <a-button size="small" status="danger">删除</a-button>
              </a-popconfirm>
            </a-space>
          </template>
        </a-table-column>
      </template>
    </a-table>

    <!-- 编辑抽屉 -->
    <a-drawer :visible="visible" :width="640" @cancel="visible = false" @ok="onSave" :ok-loading="saving">
      <template #title>{{ form.id ? '编辑审批流' : '新建审批流' }}</template>
      <a-form :model="form" layout="vertical">
        <a-form-item label="名称" required>
          <a-input v-model="form.name" placeholder="如：运维变更审批" />
        </a-form-item>
        <a-form-item label="适用工具类型">
          <a-checkbox-group v-model="form.kinds">
            <a-checkbox value="exec">命令执行</a-checkbox>
            <a-checkbox value="write_file">文件写入</a-checkbox>
            <a-checkbox value="*">所有操作</a-checkbox>
          </a-checkbox-group>
        </a-form-item>
        <a-form-item label="触发关键词（命中任一即触发；留空表示匹配全部）">
          <a-input-tag v-model="form.triggerPatterns" placeholder="输入关键词后回车，如 rm -rf、shutdown" allow-clear />
        </a-form-item>
        <a-form-item label="启用">
          <a-switch v-model="form.enabled" />
        </a-form-item>

        <a-divider orientation="left">审批步骤</a-divider>
        <div v-for="(step, idx) in form.steps" :key="idx" class="step-card">
          <div class="step-head">
            <strong>第 {{ step.order }} 步</strong>
            <a-space>
              <a-button size="mini" @click="moveStep(idx, -1)" :disabled="idx === 0">上移</a-button>
              <a-button size="mini" @click="moveStep(idx, 1)" :disabled="idx === form.steps.length - 1">下移</a-button>
              <a-button size="mini" status="danger" @click="removeStep(idx)">删除</a-button>
            </a-space>
          </div>
          <a-row :gutter="8">
            <a-col :span="12">
              <a-form-item label="步骤名称">
                <a-input v-model="step.name" placeholder="如：部门主管" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="审批角色">
                <a-select v-model="step.approverRole" placeholder="选择角色">
                  <a-option value="admin">管理员</a-option>
                  <a-option value="manager">经理</a-option>
                  <a-option value="supervisor">主管</a-option>
                </a-select>
              </a-form-item>
            </a-col>
          </a-row>
          <a-form-item label="指定审批人（用户名，精确匹配）">
            <a-input-tag v-model="step.approverIds" placeholder="留空则按角色匹配" />
          </a-form-item>
          <a-row :gutter="8">
            <a-col :span="12">
              <a-form-item label="通知渠道">
                <a-select v-model="step.notifyChannels" multiple placeholder="选择 IM 渠道">
                  <a-option value="dingtalk">钉钉</a-option>
                  <a-option value="feishu">飞书</a-option>
                  <a-option value="telegram">Telegram</a-option>
                  <a-option value="discord">Discord</a-option>
                  <a-option value="wecom">企业微信</a-option>
                  <a-option value="slack">Slack</a-option>
                </a-select>
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="通知目标（chat_id / 群ID，与渠道顺序对应）">
                <a-input-tag v-model="step.notifyTargets" placeholder="如：12345678" />
              </a-form-item>
            </a-col>
          </a-row>
          <a-row :gutter="8">
            <a-col :span="12">
              <a-form-item label="超时自动通过（秒，0=不自动）">
                <a-input-number v-model="step.autoApproveAfterSecs" :min="0" placeholder="如 3600" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="审批方式">
                <a-radio-group v-model="step.requireAll">
                  <a-radio :value="false">任一审批人</a-radio>
                  <a-radio :value="true">全部审批人</a-radio>
                </a-radio-group>
              </a-form-item>
            </a-col>
          </a-row>
        </div>
        <a-button long @click="addStep" class="mt-8">
          <template #icon><icon-plus /></template>
          添加步骤
        </a-button>
      </a-form>
    </a-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { Message } from '@arco-design/web-vue';
import { useApprovalStore, type ApprovalStep } from '@/stores/approval';

const store = useApprovalStore();

const visible = ref(false);
const saving = ref(false);

interface FlowForm {
  id?: string;
  name: string;
  kinds: string[];
  triggerPatterns: string[];
  enabled: boolean;
  steps: ApprovalStep[];
}

const form = reactive<FlowForm>({
  name: '',
  kinds: ['exec'],
  triggerPatterns: [],
  enabled: true,
  steps: [],
});

function resetForm() {
  form.id = undefined;
  form.name = '';
  form.kinds = ['exec'];
  form.triggerPatterns = [];
  form.enabled = true;
  form.steps = [];
}

function openCreate() {
  resetForm();
  // 预置 2 步示例
  form.steps = [
    { order: 1, name: '部门主管', approverRole: 'supervisor', approverIds: [], notifyChannels: [], notifyTargets: [], autoApproveAfterSecs: undefined, requireAll: false },
    { order: 2, name: '团队领导', approverRole: 'manager', approverIds: [], notifyChannels: [], notifyTargets: [], autoApproveAfterSecs: undefined, requireAll: false },
  ];
  visible.value = true;
}

function openEdit(r: any) {
  form.id = r.id;
  form.name = r.name;
  form.kinds = [...r.kinds];
  form.triggerPatterns = [...r.triggerPatterns];
  form.enabled = r.enabled;
  form.steps = r.steps.map((s: any) => ({ ...s }));
  visible.value = true;
}

function addStep() {
  form.steps.push({
    order: form.steps.length + 1,
    name: '',
    approverRole: 'supervisor',
    approverIds: [],
    notifyChannels: [],
    notifyTargets: [],
    autoApproveAfterSecs: undefined,
    requireAll: false,
  });
  reindex();
}

function removeStep(idx: number) {
  form.steps.splice(idx, 1);
  reindex();
}

function moveStep(idx: number, dir: number) {
  const target = idx + dir;
  if (target < 0 || target >= form.steps.length) return;
  const tmp = form.steps[idx];
  form.steps[idx] = form.steps[target];
  form.steps[target] = tmp;
  reindex();
}

function reindex() {
  form.steps.forEach((s, i) => (s.order = i + 1));
}

async function onSave() {
  if (!form.name.trim()) {
    Message.warning('请输入名称');
    return;
  }
  if (!form.steps.length) {
    Message.warning('至少需要 1 个审批步骤');
    return;
  }
  saving.value = true;
  try {
    const payload = {
      ...form,
      autoApproveAfterSecs: undefined, // 顶层不传，每步独立
    };
    if (form.id) {
      await store.updateFlow(form.id, payload as any);
    } else {
      await store.createFlow(payload as any);
    }
    Message.success(form.id ? '已更新' : '已创建');
    visible.value = false;
  } catch (e: any) {
    Message.error(e.message);
  } finally {
    saving.value = false;
  }
}

async function onDelete(id: string) {
  try {
    await store.deleteFlow(id);
    Message.success('已删除');
  } catch (e: any) {
    Message.error(e.message);
  }
}

onMounted(() => store.loadFlows());
</script>

<style lang="less" scoped>
.muted { color: var(--color-text-3); }
.step-card {
  border: 1px solid var(--color-border-1);
  border-radius: 6px;
  padding: 12px;
  margin-bottom: 12px;
  background-color: var(--color-bg-2);
}
.step-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
</style>
