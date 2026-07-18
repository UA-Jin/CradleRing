<template>
  <div class="page-container">
    <a-page-header title="邮件配置" subtitle="SMTP 发信配置（用于二步验证邮件 / 通知邮件）" :show-back="false" />
    <a-card class="mt-16">
      <a-form :model="emailForm" layout="vertical" style="max-width:640px">
        <a-form-item label="SMTP 服务器"><a-input v-model="emailForm.host" placeholder="smtp.gmail.com" /></a-form-item>
        <a-row :gutter="16">
          <a-col :span="12"><a-form-item label="端口"><a-input-number v-model="emailForm.port" :min="1" :max="65535" /></a-form-item></a-col>
          <a-col :span="12"><a-form-item label="加密方式">
            <a-select v-model="emailForm.encryption">
              <a-option value="tls">TLS</a-option><a-option value="ssl">SSL</a-option><a-option value="none">无</a-option>
            </a-select>
          </a-form-item></a-col>
        </a-row>
        <a-row :gutter="16">
          <a-col :span="12"><a-form-item label="发信邮箱"><a-input v-model="emailForm.username" placeholder="user@gmail.com" /></a-form-item></a-col>
          <a-col :span="12"><a-form-item label="密码 / 授权码"><a-input-password v-model="emailForm.password" /></a-form-item></a-col>
        </a-row>
        <a-form-item label="发件人名称"><a-input v-model="emailForm.fromName" placeholder="CradleRing 通知" /></a-form-item>
        <a-space>
          <a-button type="primary" :loading="saving" @click="saveConfig">保存配置</a-button>
          <a-button :loading="testing" @click="testEmail">
            <template #icon><icon-thunderbolt /></template>发送测试邮件
          </a-button>
        </a-space>
        <a-form-item label="测试收件人" style="margin-top:16px"><a-input v-model="testTo" placeholder="test@example.com" style="width:300px" /></a-form-item>
        <div v-if="testResult" class="test-result" :class="{ ok: testResult.ok, fail: !testResult.ok }">{{ testResult.message }}</div>
      </a-form>
    </a-card>
  </div>
</template>
<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import { IconThunderbolt } from '@arco-design/web-vue/es/icon';

const saving = ref(false); const testing = ref(false); const testTo = ref('');
const testResult = ref<{ok:boolean;message:string}|null>(null);
const emailForm = reactive({ host:'', port:587, encryption:'tls', username:'', password:'', fromName:'CradleRing' });

async function loadConfig() {
  try {
    const res = await rpc.call<any>('email.config.get');
    const cfg = res.config || {};
    emailForm.host = cfg.host || ''; emailForm.port = cfg.port || 587;
    emailForm.encryption = cfg.encryption || 'tls'; emailForm.username = cfg.username || '';
    emailForm.password = cfg.password || ''; emailForm.fromName = cfg.fromName || 'CradleRing';
  } catch { /* ignore */ }
}
async function saveConfig() {
  saving.value = true;
  try { await rpc.call('email.config.set', { config: emailForm }); Message.success('已保存'); }
  catch (e:any) { Message.error(e.message); } finally { saving.value = false; }
}
async function testEmail() {
  if (!testTo.value) { Message.warning('请输入测试收件人'); return; }
  testing.value = true; testResult.value = null;
  try {
    const res = await rpc.call<any>('email.test', { to: testTo.value });
    testResult.value = { ok: res.ok, message: res.ok ? '测试邮件发送成功' : (res.error || '发送失败') };
    if (res.ok) Message.success('发送成功'); else Message.error(res.error);
  } catch (e:any) { testResult.value = { ok:false, message: e.message }; Message.error(e.message); }
  finally { testing.value = false; }
}
onMounted(loadConfig);
</script>
<style lang="less" scoped>
.test-result { margin-top:12px; padding:8px 12px; border-radius:6px; font-size:13px;
  &.ok { background: rgba(86,202,0,0.1); color: var(--brand-success); }
  &.fail { background: rgba(255,76,81,0.1); color: var(--brand-danger); }
}
</style>
