<template>
  <div class="page-container">
    <a-page-header title="文件管理" subtitle="远程节点文件浏览与操作" :show-back="false">
      <template #extra>
        <a-space>
          <a-select v-model="currentNodeId" style="width: 200px" @change="loadFiles" size="small">
            <a-option value="local">本机</a-option>
            <a-option v-for="n in nodes" :key="n.id" :value="n.id">{{ n.name }}</a-option>
          </a-select>
          <a-button @click="loadFiles"><template #icon><icon-refresh /></template></a-button>
          <a-button type="primary" @click="showMkdir = true"><template #icon><icon-folder-add /></template>新建目录</a-button>
        </a-space>
      </template>
    </a-page-header>

    <!-- 路径面包屑 -->
    <a-card class="mt-16">
      <div class="path-bar">
        <a-button size="small" @click="goUp" :disabled="currentPath === '/'"><icon-arrow-up /></a-button>
        <a-breadcrumb class="path-breadcrumb">
          <a-breadcrumb-item @click="goTo('/')">根目录</a-breadcrumb-item>
          <a-breadcrumb-item v-for="(seg, i) in pathSegments" :key="i" @click="goTo(seg.path)">
            {{ seg.name }}
          </a-breadcrumb-item>
        </a-breadcrumb>
        <a-input v-model="currentPath" style="width: 300px; margin-left: auto" @keydown.enter="loadFiles" placeholder="/home" />
      </div>
    </a-card>

    <!-- 文件列表 -->
    <a-card class="mt-16">
      <a-table :data="files" :loading="loading" :pagination="{ pageSize: 20 }" row-key="name">
        <template #columns>
          <a-table-column title="名称" :width="280">
            <template #cell="{ record }">
              <a-space>
                <icon-folder v-if="record.isDir" style="color: #ffb400" />
                <icon-file v-else style="color: var(--brand-info)" />
                <a-link v-if="record.isDir" @click="enterDir(record)">{{ record.name }}</a-link>
                <a-link v-else @click="viewFile(record)">{{ record.name }}</a-link>
              </a-space>
            </template>
          </a-table-column>
          <a-table-column title="大小" :width="100">
            <template #cell="{ record }">{{ record.isDir ? '-' : formatSize(record.size) }}</template>
          </a-table-column>
          <a-table-column title="修改时间" :width="160">
            <template #cell="{ record }">{{ record.modified ? dayjs(record.modified).format('MM-DD HH:mm') : '-' }}</template>
          </a-table-column>
          <a-table-column title="操作" :width="160" fixed="right">
            <template #cell="{ record }">
              <a-space>
                <a-button v-if="!record.isDir" size="small" @click="viewFile(record)">查看</a-button>
                <a-button size="small" @click="downloadFile(record)">下载</a-button>
                <a-popconfirm content="确认删除？" @ok="deleteFile(record)">
                  <a-button size="small" status="danger">删除</a-button>
                </a-popconfirm>
              </a-space>
            </template>
          </a-table-column>
        </template>
      </a-table>
    </a-card>

    <!-- 查看文件对话框 -->
    <a-modal :visible="viewVisible" :title="viewingFile?.name" @cancel="viewVisible = false" :footer="false" :width="800">
      <pre class="file-content">{{ fileContent }}</pre>
    </a-modal>

    <!-- 新建目录对话框 -->
    <a-modal :visible="showMkdir" title="新建目录" @cancel="showMkdir = false" @ok="createDir" :width="420">
      <a-input v-model="newDirName" placeholder="目录名" />
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import dayjs from 'dayjs';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import { IconRefresh, IconFolderAdd, IconArrowUp, IconFolder, IconFile } from '@arco-design/web-vue/es/icon';

const currentNodeId = ref('local');
const nodes = ref<any[]>([]);
const currentPath = ref('/home');
const files = ref<any[]>([]);
const loading = ref(false);
const viewVisible = ref(false);
const viewingFile = ref<any>(null);
const fileContent = ref('');
const showMkdir = ref(false);
const newDirName = ref('');

const pathSegments = computed(() => {
  const parts = currentPath.value.split('/').filter(Boolean);
  return parts.map((name, i) => ({
    name,
    path: '/' + parts.slice(0, i + 1).join('/'),
  }));
});

async function loadFiles() {
  loading.value = true;
  try {
    const res = await rpc.call<any>('files.list', { path: currentPath.value, nodeId: currentNodeId.value });
    files.value = (res.files || []).sort((a: any, b: any) => (b.isDir ? 1 : 0) - (a.isDir ? 1 : 0) || a.name.localeCompare(b.name));
  } catch (e: any) {
    Message.error(e.message);
  } finally {
    loading.value = false;
  }
}

async function loadNodes() {
  try {
    const res = await rpc.call<any>('nodes.list');
    nodes.value = (res.nodes || []).filter((n: any) => n.status === 'online');
  } catch { /* ignore */ }
}

function enterDir(record: any) {
  currentPath.value = record.path;
  loadFiles();
}

function goTo(path: string) {
  currentPath.value = path;
  loadFiles();
}

function goUp() {
  const parent = currentPath.value.split('/').slice(0, -1).join('/') || '/';
  currentPath.value = parent;
  loadFiles();
}

async function viewFile(record: any) {
  viewingFile.value = record;
  try {
    const res = await rpc.call<any>('files.read', { path: record.path, nodeId: currentNodeId.value });
    fileContent.value = res.content || '(空文件)';
  } catch (e: any) {
    fileContent.value = `读取失败: ${e.message}`;
  }
  viewVisible.value = true;
}

async function downloadFile(record: any) {
  try {
    const res = await rpc.call<any>('files.read', { path: record.path, nodeId: currentNodeId.value, raw: true });
    const blob = new Blob([res.content || ''], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = record.name;
    a.click();
    URL.revokeObjectURL(url);
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function deleteFile(record: any) {
  try {
    await rpc.call('files.delete', { path: record.path, nodeId: currentNodeId.value });
    Message.success('已删除');
    await loadFiles();
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function createDir() {
  if (!newDirName.value.trim()) return;
  try {
    const path = currentPath.value === '/' ? `/${newDirName.value}` : `${currentPath.value}/${newDirName.value}`;
    await rpc.call('files.mkdir', { path, nodeId: currentNodeId.value });
    Message.success('已创建');
    showMkdir.value = false;
    newDirName.value = '';
    await loadFiles();
  } catch (e: any) {
    Message.error(e.message);
  }
}

function formatSize(bytes: number): string {
  if (bytes >= 1073741824) return (bytes / 1073741824).toFixed(1) + 'G';
  if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + 'M';
  if (bytes >= 1024) return (bytes / 1024).toFixed(1) + 'K';
  return bytes + 'B';
}

onMounted(() => {
  loadNodes();
  loadFiles();
});
</script>

<style lang="less" scoped>
.path-bar {
  display: flex;
  align-items: center;
  gap: 12px;
}
.path-breadcrumb {
  flex: 1;
  :deep(.arco-breadcrumb-item) {
    cursor: pointer;
    color: var(--brand-primary);
    &:hover { text-decoration: underline; }
  }
}
.file-content {
  max-height: 500px;
  overflow: auto;
  background: var(--color-bg-3);
  padding: 16px;
  border-radius: 6px;
  font-family: monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
