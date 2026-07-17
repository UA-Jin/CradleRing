<template>
  <div class="page-container">
    <a-page-header title="工作台" subtitle="快速开始 · 最近任务 · 快捷操作" :show-back="false" />

    <a-row :gutter="16" class="mt-16">
      <a-col :xs="24" :md="8">
        <a-card hoverable @click="$router.push('/chat')">
          <div class="quick-card">
            <a-avatar :size="48" :style="{ backgroundColor: '#165dff' }"><icon-message /></a-avatar>
            <div>
              <div class="qc-title">开始对话</div>
              <div class="qc-desc">与 AI Agent 智能对话</div>
            </div>
          </div>
        </a-card>
      </a-col>
      <a-col :xs="24" :md="8">
        <a-card hoverable @click="$router.push('/approvals/instances')">
          <div class="quick-card">
            <a-avatar :size="48" :style="{ backgroundColor: '#ff7d00' }"><icon-check-circle /></a-avatar>
            <div>
              <div class="qc-title">审批中心</div>
              <div class="qc-desc">处理待审批任务</div>
            </div>
          </div>
        </a-card>
      </a-col>
      <a-col :xs="24" :md="8">
        <a-card hoverable @click="$router.push('/channels')">
          <div class="quick-card">
            <a-avatar :size="48" :style="{ backgroundColor: '#00b42a' }"><icon-share-internal /></a-avatar>
            <div>
              <div class="qc-title">渠道管理</div>
              <div class="qc-desc">配置 IM 渠道接入</div>
            </div>
          </div>
        </a-card>
      </a-col>
    </a-row>

    <a-row :gutter="16" class="mt-16">
      <a-col :span="24">
        <a-card title="快捷工具">
          <a-space wrap size="large">
            <a-button v-for="t in quickTools" :key="t.name" @click="runTool(t.name)">
              <template #icon><component :is="t.icon" /></template>
              {{ t.label }}
            </a-button>
          </a-space>
        </a-card>
      </a-col>
    </a-row>
  </div>
</template>

<script setup lang="ts">
import { markRaw } from 'vue';
import { useRouter } from 'vue-router';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import {
  IconSearch, IconCode, IconBulb, IconFile, IconImage, IconRecord, IconBranch,
  IconMessage, IconCheckCircle, IconShareInternal,
} from '@arco-design/web-vue/es/icon';

const router = useRouter();

const quickTools = [
  { name: 'web_search', label: '网络搜索', icon: markRaw(IconSearch) },
  { name: 'exec', label: '执行命令', icon: markRaw(IconCode) },
  { name: 'run_code', label: '运行代码', icon: markRaw(IconBulb) },
  { name: 'read_document', label: '解析文档', icon: markRaw(IconFile) },
  { name: 'analyze_image', label: '图像分析', icon: markRaw(IconImage) },
  { name: 'transcribe_audio', label: '语音转写', icon: markRaw(IconRecord) },
  { name: 'dns_lookup', label: 'DNS 查询', icon: markRaw(IconBranch) },
];

async function runTool(name: string) {
  // 跳转到对话页并通过 query 传递快捷工具
  router.push({ path: '/chat', query: { tool: name } });
  Message.info(`已选择工具：${name}，请在对话中使用`);
}
</script>

<style lang="less" scoped>
.quick-card {
  display: flex;
  align-items: center;
  gap: 16px;
  cursor: pointer;
  .qc-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-1);
  }
  .qc-desc {
    font-size: 13px;
    color: var(--color-text-3);
    margin-top: 4px;
  }
}
</style>
