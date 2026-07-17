<template>
  <a-drawer
    :visible="visible"
    :width="300"
    :footer="false"
    placement="right"
    @cancel="$emit('update:visible', false)"
  >
    <template #title>主题设置</template>
    <div class="setting-panel">
      <a-form layout="vertical">
        <a-form-item label="暗色模式">
          <a-switch
            :model-value="appStore.isDark"
            @change="appStore.toggleDark()"
          />
        </a-form-item>
        <a-form-item label="主题色">
          <a-radio-group type="button" v-model="appStore.themeColor">
            <a-radio value="#165dff">蓝</a-radio>
            <a-radio value="#00b42a">绿</a-radio>
            <a-radio value="#ff7d00">橙</a-radio>
            <a-radio value="#f53f3f">红</a-radio>
            <a-radio value="#722ed1">紫</a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item label="侧边栏">
          <a-switch
            :model-value="!appStore.menuCollapse"
            @change="(v) => (appStore.menuCollapse = !v)"
          />
        </a-form-item>
      </a-form>
    </div>
  </a-drawer>
</template>

<script setup lang="ts">
import { useAppStore } from '@/stores/app';

defineProps<{ visible: boolean }>();
defineEmits<{ (e: 'update:visible', v: boolean): void }>();

const appStore = useAppStore();
</script>

<style lang="less" scoped>
.setting-panel {
  padding: 8px 0;
}
</style>
