<template>
  <div class="page-container">
    <a-page-header title="定时任务" subtitle="Cron 调度任务" :show-back="false">
      <template #extra>
        <a-button type="primary" @click="visible = true"><template #icon><icon-plus /></template>新建任务</a-button>
      </template>
    </a-page-header>
    <a-table class="mt-16" :data="jobs" :loading="loading" :pagination="{ pageSize: 15 }" row-key="id">
      <template #columns>
        <a-table-column title="名称" data-index="name" :width="160">
          <template #cell="{ record }"><a-link @click="openRun(record)">{{ record.name }}</a-link></template>
        </a-table-column>
        <a-table-column title="Cron 表达式" :width="160">
          <template #cell="{ record }"><a-tag>{{ record.cron }}</a-tag></template>
        </a-table-column>
        <a-table-column title="Prompt" :width="300" ellipsis tooltip>
          <template #cell="{ record }">{{ record.prompt }}</template>
        </a-table-column>
        <a-table-column title="会话" data-index="sessionKey" :width="140" />
        <a-table-column title="启用" :width="80">
          <template #cell="{ record }"><a-switch v-model="record.enabled" @change="toggle(record)" /></template>
        </a-table-column>
        <a-table-column title="下次执行" :width="160">
          <template #cell="{ record }">{{ record.nextRunAt ? dayjs(record.nextRunAt).format('MM-DD HH:mm') : '-' }}</template>
        </a-table-column>
        <a-table-column title="操作" :width="140" fixed="right">
          <template #cell="{ record }">
            <a-space>
              <a-button size="small" @click="openRun(record)">执行</a-button>
              <a-popconfirm @ok="onDelete(record.id)"><a-button size="small" status="danger">删除</a-button></a-popconfirm>
            </a-space>
          </template>
        </a-table-column>
      </template>
    </a-table>
    <a-modal :visible="visible" title="新建定时任务" @cancel="visible = false" @ok="onSave" :ok-loading="saving" :width="560">
      <a-form :model="form" layout="vertical">
        <a-form-item label="名称"><a-input v-model="form.name" /></a-form-item>
        <a-form-item label="Cron 表达式"><a-input v-model="form.cron" placeholder="如：0 9 * * * （每天 9 点）" /></a-form-item>
        <a-form-item label="会话"><a-input v-model="form.sessionKey" placeholder="main" /></a-form-item>
        <a-form-item label="Prompt"><a-textarea v-model="form.prompt" :auto-size="{ minRows: 3 }" /></a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import dayjs from 'dayjs';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';

const loading = ref(false);
const jobs = ref<any[]>([]);
const visible = ref(false);
const saving = ref(false);
const form = reactive({ name: '', cron: '', sessionKey: 'main', prompt: '', enabled: true });

async function load() {
  loading.value = true;
  try {
    const res = await rpc.call<{ jobs: any[] }>('cron.list');
    jobs.value = res.jobs || [];
  } catch (e: any) { Message.error(e.message); }
  finally { loading.value = false; }
}

async function onSave() {
  saving.value = true;
  try {
    await rpc.call('cron.create', form);
    Message.success('已创建');
    visible.value = false;
    await load();
  } catch (e: any) { Message.error(e.message); }
  finally { saving.value = false; }
}

async function toggle(j: any) {
  try { await rpc.call('cron.update', { id: j.id, enabled: j.enabled }); } catch (e: any) { Message.error(e.message); }
}

async function openRun(j: any) {
  try { await rpc.call('cron.run', { id: j.id }); Message.success('已触发'); } catch (e: any) { Message.error(e.message); }
}

async function onDelete(id: string) {
  try { await rpc.call('cron.delete', { id }); Message.success('已删除'); await load(); } catch (e: any) { Message.error(e.message); }
}

onMounted(load);
</script>
