<template>
  <div class="page-container">
    <a-page-header title="记忆库" subtitle="Agent 的长期记忆" :show-back="false">
      <template #extra>
        <a-button type="primary" @click="openCreate"><template #icon><icon-plus /></template>添加记忆</a-button>
      </template>
    </a-page-header>
    <a-input v-model="q" placeholder="搜索记忆..." allow-clear class="mt-16" style="width: 320px" />
    <a-list class="mt-16" :data="filtered" :loading="loading">
      <template #item="{ item }">
        <a-list-item>
          <a-list-item-meta :title="item.body" :description="`类型：${item.kind} · 来源：${item.source} · ${dayjs(item.createdAt).fromNow()}`">
            <template #avatar><a-avatar><icon-bookmark /></a-avatar></template>
          </a-list-item-meta>
          <template #actions>
            <a-popconfirm content="确认删除？" @ok="onDelete(item.id)">
              <a-button size="small" status="danger">删除</a-button>
            </a-popconfirm>
          </template>
        </a-list-item>
      </template>
    </a-list>

    <a-modal :visible="visible" title="添加记忆" @cancel="visible = false" @ok="onSave" :ok-loading="saving">
      <a-form :model="form" layout="vertical">
        <a-form-item label="内容" required>
          <a-textarea v-model="form.body" :auto-size="{ minRows: 3 }" placeholder="记忆内容..." />
        </a-form-item>
        <a-form-item label="类型">
          <a-select v-model="form.kind">
            <a-option value="fact">事实</a-option>
            <a-option value="preference">偏好</a-option>
            <a-option value="instruction">指令</a-option>
            <a-option value="note">笔记</a-option>
          </a-select>
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/zh-cn';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import { IconBookmark } from '@arco-design/web-vue/es/icon';

dayjs.extend(relativeTime);
dayjs.locale('zh-cn');

const loading = ref(false);
const items = ref<any[]>([]);
const q = ref('');
const filtered = computed(() => items.value.filter((m) => !q.value || m.body.toLowerCase().includes(q.value.toLowerCase())));

const visible = ref(false);
const saving = ref(false);
const form = reactive({ body: '', kind: 'fact' });

async function load() {
  loading.value = true;
  try {
    const res = await rpc.call<{ memories: any[] }>('memory.list');
    items.value = res.memories || [];
  } catch (e: any) { Message.error(e.message); }
  finally { loading.value = false; }
}

function openCreate() { form.body = ''; form.kind = 'fact'; visible.value = true; }

async function onSave() {
  if (!form.body.trim()) { Message.warning('请输入内容'); return; }
  saving.value = true;
  try {
    await rpc.call('memory.save', form);
    Message.success('已保存');
    visible.value = false;
    await load();
  } catch (e: any) { Message.error(e.message); }
  finally { saving.value = false; }
}

async function onDelete(id: number) {
  try {
    await rpc.call('memory.delete', { id });
    Message.success('已删除');
    await load();
  } catch (e: any) { Message.error(e.message); }
}

onMounted(load);
</script>
