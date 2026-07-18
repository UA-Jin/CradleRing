<template>
  <div class="page-container">
    <a-page-header title="安全中心" subtitle="WAF · IDS/IPS · IP 名单 · 实时阻断 · 事件日志" :show-back="false">
      <template #extra>
        <a-space>
          <a-button @click="loadAll"><template #icon><icon-refresh /></template>刷新</a-button>
          <a-button type="primary" status="danger" @click="showBanIp = true"><template #icon><icon-stop /></template>封禁 IP</a-button>
        </a-space>
      </template>
    </a-page-header>

    <!-- 安全总览卡 -->
    <a-row :gutter="24" class="mt-16">
      <a-col :xs="12" :lg="6" v-for="s in overviewCards" :key="s.title">
        <a-card class="ov-card">
          <div class="ov-icon" :style="{ background: s.bg }"><component :is="s.icon" /></div>
          <div class="ov-info">
            <div class="ov-num">{{ s.value }}</div>
            <div class="ov-label">{{ s.title }}</div>
          </div>
          <a-tag :color="s.tagColor" size="small" class="ov-tag">{{ s.tag }}</a-tag>
        </a-card>
      </a-col>
    </a-row>

    <!-- 标签页 -->
    <a-tabs v-model:active-key="activeTab" type="rounded" class="mt-24">
      <!-- WAF 规则 -->
      <a-tab-pane key="waf" title="WAF 规则">
        <a-card>
          <template #extra>
            <a-space>
              <a-input v-model="wafSearch" placeholder="搜索规则..." allow-clear style="width: 180px" />
              <a-button type="primary" size="small" @click="showWafCreate = true"><template #icon><icon-plus /></template>自定义规则</a-button>
            </a-space>
          </template>
          <a-table :data="filteredWafRules" :pagination="{ pageSize: 15 }" row-key="id" :loading="wafLoading">
            <template #columns>
              <a-table-column title="规则" :width="220">
                <template #cell="{ record }">
                  <div>
                    <div class="rule-name">{{ record.name || record.id }}</div>
                    <div class="rule-pattern">{{ (record.pattern || '').slice(0, 60) }}</div>
                  </div>
                </template>
              </a-table-column>
              <a-table-column title="类型" :width="100">
                <template #cell="{ record }">
                  <a-tag :color="ruleTypeColor(record.ruleType)" size="small">{{ record.ruleType }}</a-tag>
                </template>
              </a-table-column>
              <a-table-column title="动作" :width="90">
                <template #cell="{ record }">
                  <a-tag :color="record.action === 'block' ? 'red' : 'orange'" size="small">{{ record.action }}</a-tag>
                </template>
              </a-table-column>
              <a-table-column title="严重度" :width="90">
                <template #cell="{ record }">
                  <a-tag :color="severityColor(record.severity)" size="small">{{ record.severity }}</a-tag>
                </template>
              </a-table-column>
              <a-table-column title="命中" :width="80" data-index="hitCount" />
              <a-table-column title="来源" :width="80">
                <template #cell="{ record }">
                  <a-tag :color="record.builtin ? 'arcoblue' : 'green'" size="small">{{ record.builtin ? '内置' : '自定义' }}</a-tag>
                </template>
              </a-table-column>
              <a-table-column title="启用" :width="80">
                <template #cell="{ record }">
                  <a-switch :model-value="record.enabled !== false" @change="toggleWafRule(record)" size="small" />
                </template>
              </a-table-column>
              <a-table-column title="操作" :width="80" fixed="right">
                <template #cell="{ record }">
                  <a-popconfirm v-if="!record.builtin" content="确认删除？" @ok="deleteWafRule(record.id)">
                    <a-button size="small" status="danger" type="text"><icon-delete /></a-button>
                  </a-popconfirm>
                </template>
              </a-table-column>
            </template>
          </a-table>
        </a-card>
      </a-tab-pane>

      <!-- IDS 规则 -->
      <a-tab-pane key="ids" title="IDS/IPS 规则">
        <a-card>
          <template #extra>
            <a-button type="primary" size="small" @click="showIdsCreate = true"><template #icon><icon-plus /></template>自定义规则</a-button>
          </template>
          <a-table :data="idsRules" :pagination="{ pageSize: 15 }" row-key="id" :loading="idsLoading">
            <template #columns>
              <a-table-column title="规则" :width="240">
                <template #cell="{ record }">
                  <div>
                    <div class="rule-name">{{ record.name || record.id }}</div>
                    <div class="rule-pattern">{{ record.description || '' }}</div>
                  </div>
                </template>
              </a-table-column>
              <a-table-column title="类型" :width="110">
                <template #cell="{ record }">
                  <a-tag :color="idsTypeColor(record.ruleType)" size="small">{{ record.ruleType }}</a-tag>
                </template>
              </a-table-column>
              <a-table-column title="阈值" :width="100">
                <template #cell="{ record }">{{ record.threshold || '-' }}</template>
              </a-table-column>
              <a-table-column title="动作" :width="90">
                <template #cell="{ record }">
                  <a-tag :color="record.action === 'ban' ? 'red' : 'orange'" size="small">{{ record.action }}</a-tag>
                </template>
              </a-table-column>
              <a-table-column title="启用" :width="80">
                <template #cell="{ record }">
                  <a-switch :model-value="record.enabled !== false" @change="toggleIdsRule(record)" size="small" />
                </template>
              </a-table-column>
              <a-table-column title="操作" :width="80" fixed="right">
                <template #cell="{ record }">
                  <a-popconfirm v-if="!record.builtin" content="确认删除？" @ok="deleteIdsRule(record.id)">
                    <a-button size="small" status="danger" type="text"><icon-delete /></a-button>
                  </a-popconfirm>
                </template>
              </a-table-column>
            </template>
          </a-table>
        </a-card>
      </a-tab-pane>

      <!-- IP 名单 -->
      <a-tab-pane key="ip" title="IP 名单">
        <a-row :gutter="24">
          <a-col :xs="24" :lg="12">
            <a-card title="黑名单（自动封禁 + 手动封禁）">
              <template #extra>
                <a-button size="small" @click="showBanIp = true"><template #icon><icon-plus /></template>添加</a-button>
              </template>
              <a-list :data="bannedIps" :pagination="{ pageSize: 10 }">
                <template #item="{ item }">
                  <a-list-item>
                    <a-list-item-meta :title="item.ip" :description="`${item.reason} · ${dayjs(item.banned_at).format('MM-DD HH:mm')}`">
                      <template #avatar><a-avatar :style="{ backgroundColor: '#ff4c51' }"><icon-stop /></a-avatar></template>
                    </a-list-item-meta>
                    <template #actions>
                      <a-button size="small" @click="unbanIp(item.ip)">解封</a-button>
                    </template>
                  </a-list-item>
                </template>
              </a-list>
            </a-card>
          </a-col>
          <a-col :xs="24" :lg="12">
            <a-card title="白名单（优先放行）">
              <template #extra>
                <a-button size="small" @click="showWhiteIp = true"><template #icon><icon-plus /></template>添加</a-button>
              </template>
              <a-list :data="whiteIps" :pagination="{ pageSize: 10 }">
                <template #item="{ item }">
                  <a-list-item>
                    <a-list-item-meta :title="item.ip" :description="item.reason || '手动添加'">
                      <template #avatar><a-avatar :style="{ backgroundColor: '#56ca00' }"><icon-check-circle /></a-avatar></template>
                    </a-list-item-meta>
                    <template #actions>
                      <a-button size="small" status="danger" @click="removeWhiteIp(item.ip)">移除</a-button>
                    </template>
                  </a-list-item>
                </template>
              </a-list>
            </a-card>
          </a-col>
        </a-row>
      </a-tab-pane>

      <!-- 事件日志 -->
      <a-tab-pane key="events" title="安全事件">
        <a-card>
          <template #extra>
            <a-space>
              <a-select v-model="eventFilter" style="width: 140px" @change="loadEvents">
                <a-option value="">全部类型</a-option>
                <a-option value="waf_block">WAF 拦截</a-option>
                <a-option value="ids_detect">IDS 检测</a-option>
                <a-option value="ids_ban">IDS 封禁</a-option>
                <a-option value="ip_block">IP 阻断</a-option>
                <a-option value="rate_limit">速率限制</a-option>
              </a-select>
              <a-button size="small" @click="exportEvents"><template #icon><icon-download /></template>导出</a-button>
            </a-space>
          </template>
          <a-table :data="filteredEvents" :pagination="{ pageSize: 20 }" row-key="ts" :loading="eventsLoading">
            <template #columns>
              <a-table-column title="时间" :width="140">
                <template #cell="{ record }">{{ dayjs(record.ts).format('MM-DD HH:mm:ss') }}</template>
              </a-table-column>
              <a-table-column title="类型" :width="100">
                <template #cell="{ record }">
                  <a-tag :color="eventKindColor(record.kind)" size="small">{{ eventKindLabel(record.kind) }}</a-tag>
                </template>
              </a-table-column>
              <a-table-column title="来源" :width="140" data-index="source" ellipsis tooltip />
              <a-table-column title="目标" :width="160" data-index="target" ellipsis tooltip />
              <a-table-column title="严重度" :width="90">
                <template #cell="{ record }">
                  <a-tag :color="severityColor(record.severity)" size="small">{{ record.severity }}</a-tag>
                </template>
              </a-table-column>
              <a-table-column title="动作" :width="80">
                <template #cell="{ record }">
                  <a-tag :color="record.action === 'block' || record.action === 'ban' ? 'red' : 'orange'" size="small">{{ record.action }}</a-tag>
                </template>
              </a-table-column>
              <a-table-column title="详情" data-index="detail" ellipsis tooltip />
            </template>
          </a-table>
        </a-card>
      </a-tab-pane>
    </a-tabs>

    <!-- 创建 WAF 规则对话框 -->
    <a-modal :visible="showWafCreate" title="创建 WAF 规则" @cancel="showWafCreate = false" @ok="createWafRule" :width="560">
      <a-form :model="wafForm" layout="vertical">
        <a-form-item label="规则名称" required><a-input v-model="wafForm.name" placeholder="自定义 XSS 检测" /></a-form-item>
        <a-row :gutter="12">
          <a-col :span="12">
            <a-form-item label="规则类型" required>
              <a-select v-model="wafForm.ruleType">
                <a-option value="sqli">SQL 注入</a-option>
                <a-option value="xss">XSS 跨站脚本</a-option>
                <a-option value="rfi">远程文件包含</a-option>
                <a-option value="lfi">本地文件包含</a-option>
                <a-option value="cmd_inject">命令注入</a-option>
                <a-option value="traversal">路径遍历</a-option>
                <a-option value="scanner">扫描器</a-option>
                <a-option value="custom">自定义</a-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="动作">
              <a-select v-model="wafForm.action">
                <a-option value="block">阻断</a-option>
                <a-option value="log">仅记录</a-option>
              </a-select>
            </a-form-item>
          </a-col>
        </a-row>
        <a-form-item label="匹配模式（正则）" required>
          <a-textarea v-model="wafForm.pattern" :auto-size="{ minRows: 3 }" placeholder="(?i)(union.*select|select.*from)" style="font-family:monospace" />
        </a-form-item>
        <a-form-item label="严重度">
          <a-radio-group v-model="wafForm.severity" type="button">
            <a-radio value="critical">严重</a-radio>
            <a-radio value="high">高</a-radio>
            <a-radio value="medium">中</a-radio>
            <a-radio value="low">低</a-radio>
          </a-radio-group>
        </a-form-item>
      </a-form>
    </a-modal>

    <!-- 创建 IDS 规则对话框 -->
    <a-modal :visible="showIdsCreate" title="创建 IDS 规则" @cancel="showIdsCreate = false" @ok="createIdsRule" :width="560">
      <a-form :model="idsForm" layout="vertical">
        <a-form-item label="规则名称" required><a-input v-model="idsForm.name" placeholder="自定义端口扫描检测" /></a-form-item>
        <a-row :gutter="12">
          <a-col :span="12">
            <a-form-item label="检测类型" required>
              <a-select v-model="idsForm.ruleType">
                <a-option value="bruteforce">暴力破解</a-option>
                <a-option value="portscan">端口扫描</a-option>
                <a-option value="malware">恶意软件</a-option>
                <a-option value="c2">C2 外联</a-option>
                <a-option value="custom">自定义</a-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="阈值"><a-input v-model="idsForm.threshold" placeholder="如 5 次/分钟" /></a-form-item>
          </a-col>
        </a-row>
        <a-form-item label="动作">
          <a-select v-model="idsForm.action">
            <a-option value="ban">自动封禁</a-option>
            <a-option value="log">仅记录</a-option>
          </a-select>
        </a-form-item>
        <a-form-item label="描述"><a-textarea v-model="idsForm.description" :auto-size="{ minRows: 2 }" /></a-form-item>
      </a-form>
    </a-modal>

    <!-- 封禁 IP 对话框 -->
    <a-modal :visible="showBanIp" title="手动封禁 IP" @cancel="showBanIp = false" @ok="banIp" :width="420">
      <a-form layout="vertical">
        <a-form-item label="IP 地址" required><a-input v-model="banIpForm.ip" placeholder="192.168.1.100 或 10.0.0.0/8" /></a-form-item>
        <a-form-item label="封禁时长">
          <a-select v-model="banIpForm.duration">
            <a-option value="3600">1 小时</a-option>
            <a-option value="86400">24 小时</a-option>
            <a-option value="604800">7 天</a-option>
            <a-option value="0">永久</a-option>
          </a-select>
        </a-form-item>
        <a-form-item label="原因"><a-input v-model="banIpForm.reason" placeholder="手动封禁" /></a-form-item>
      </a-form>
    </a-modal>

    <!-- 添加白名单对话框 -->
    <a-modal :visible="showWhiteIp" title="添加白名单 IP" @cancel="showWhiteIp = false" @ok="addWhiteIp" :width="420">
      <a-form layout="vertical">
        <a-form-item label="IP 地址" required><a-input v-model="whiteIpForm.ip" placeholder="192.168.1.100 或 10.0.0.0/8" /></a-form-item>
        <a-form-item label="备注"><a-input v-model="whiteIpForm.reason" placeholder="可信 IP" /></a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, markRaw } from 'vue';
import dayjs from 'dayjs';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import {
  IconRefresh, IconStop, IconPlus, IconDelete, IconDownload,
  IconSafe, IconFire, IconCheckCircle, IconThunderbolt,
} from '@arco-design/web-vue/es/icon';

const activeTab = ref('waf');
const wafLoading = ref(false);
const idsLoading = ref(false);
const eventsLoading = ref(false);
const wafSearch = ref('');
const eventFilter = ref('');

const showWafCreate = ref(false);
const showIdsCreate = ref(false);
const showBanIp = ref(false);
const showWhiteIp = ref(false);

const wafRules = ref<any[]>([]);
const idsRules = ref<any[]>([]);
const bannedIps = ref<any[]>([]);
const whiteIps = ref<any[]>([]);
const events = ref<any[]>([]);

const wafForm = reactive({ name: '', ruleType: 'xss', action: 'block', pattern: '', severity: 'high' });
const idsForm = reactive({ name: '', ruleType: 'portscan', action: 'ban', threshold: '', description: '' });
const banIpForm = reactive({ ip: '', duration: '86400', reason: '' });
const whiteIpForm = reactive({ ip: '', reason: '' });

// 总览卡
const overviewCards = computed(() => [
  { title: 'WAF 拦截（今日）', value: overview.wafBlocked || 0, icon: markRaw(IconSafe), bg: 'linear-gradient(135deg, #ff4c51, #ff8083)', tag: 'WAF', tagColor: 'red' },
  { title: 'IDS 检测（今日）', value: overview.idsDetected || 0, icon: markRaw(IconSafe), bg: 'linear-gradient(135deg, #ffb400, #ffd040)', tag: 'IDS', tagColor: 'orange' },
  { title: '封禁 IP', value: bannedIps.value.length, icon: markRaw(IconStop), bg: 'linear-gradient(135deg, #8c57ff, #a785fa)', tag: '封禁', tagColor: 'purple' },
  { title: '风险评分', value: overview.riskScore || 0, icon: markRaw(IconThunderbolt), bg: overview.riskScore >= 70 ? 'linear-gradient(135deg, #ff4c51, #ff8083)' : 'linear-gradient(135deg, #56ca00, #82e040)', tag: overview.riskScore >= 70 ? '高危' : '安全', tagColor: overview.riskScore >= 70 ? 'red' : 'green' },
]);

const overview = reactive({ wafBlocked: 0, idsDetected: 0, riskScore: 0 });

const filteredWafRules = computed(() =>
  wafRules.value.filter((r) => !wafSearch.value || JSON.stringify(r).toLowerCase().includes(wafSearch.value.toLowerCase())),
);

const filteredEvents = computed(() =>
  events.value.filter((e) => !eventFilter.value || e.kind === eventFilter.value),
);

function ruleTypeColor(t: string) {
  const map: Record<string, string> = { sqli: 'red', xss: 'orange', rfi: 'purple', lfi: 'purple', cmd_inject: 'red', traversal: 'orange', scanner: 'gray', custom: 'arcoblue' };
  return map[t] || 'gray';
}
function idsTypeColor(t: string) {
  const map: Record<string, string> = { bruteforce: 'red', portscan: 'orange', malware: 'red', c2: 'purple', custom: 'arcoblue' };
  return map[t] || 'gray';
}
function severityColor(s: string) {
  const map: Record<string, string> = { critical: 'red', high: 'orange', medium: 'gold', low: 'green' };
  return map[s] || 'gray';
}
function eventKindColor(k: string) {
  const map: Record<string, string> = { waf_block: 'red', waf_log: 'orange', ids_detect: 'orange', ids_ban: 'red', ip_block: 'red', rate_limit: 'purple' };
  return map[k] || 'gray';
}
function eventKindLabel(k: string) {
  const map: Record<string, string> = { waf_block: 'WAF 拦截', waf_log: 'WAF 记录', ids_detect: 'IDS 检测', ids_ban: 'IDS 封禁', ip_block: 'IP 阻断', rate_limit: '速率限制' };
  return map[k] || k;
}

async function loadAll() {
  await Promise.all([loadWafRules(), loadIdsRules(), loadBannedIps(), loadWhiteIps(), loadEvents(), loadOverview()]);
}

async function loadOverview() {
  try {
    const res = await rpc.call<any>('security.overview');
    overview.wafBlocked = res.waf?.blockedToday || 0;
    overview.idsDetected = res.ids?.detectedToday || 0;
    overview.riskScore = res.riskScore || 0;
  } catch { /* ignore */ }
}

async function loadWafRules() {
  wafLoading.value = true;
  try {
    const res = await rpc.call<any>('security.waf.rules.list');
    wafRules.value = res.rules || [];
  } catch (e: any) { Message.error(e.message); }
  finally { wafLoading.value = false; }
}

async function loadIdsRules() {
  idsLoading.value = true;
  try {
    const res = await rpc.call<any>('security.ids.rules.list');
    idsRules.value = res.rules || [];
  } catch (e: any) { Message.error(e.message); }
  finally { idsLoading.value = false; }
}

async function loadBannedIps() {
  try {
    const res = await rpc.call<any>('security.ids.banned_list');
    bannedIps.value = res.banned || [];
  } catch { /* ignore */ }
}

async function loadWhiteIps() {
  try {
    const res = await rpc.call<any>('security.ip.whitelist.list');
    whiteIps.value = res.list || [];
  } catch { /* ignore */ }
}

async function loadEvents() {
  eventsLoading.value = true;
  try {
    const res = await rpc.call<any>('security.ids.events', { limit: 100 });
    events.value = (res.events || []).reverse();
  } catch (e: any) { Message.error(e.message); }
  finally { eventsLoading.value = false; }
}

async function toggleWafRule(rule: any) {
  try {
    await rpc.call('security.waf.rules.toggle', { id: rule.id, enabled: !rule.enabled });
    await loadWafRules();
  } catch (e: any) { Message.error(e.message); }
}

async function toggleIdsRule(rule: any) {
  try {
    await rpc.call('security.ids.rules.toggle', { id: rule.id, enabled: !rule.enabled });
    await loadIdsRules();
  } catch (e: any) { Message.error(e.message); }
}

async function createWafRule() {
  if (!wafForm.name || !wafForm.pattern) { Message.warning('请填写规则名称和匹配模式'); return; }
  try {
    await rpc.call('security.waf.rules.create', { ...wafForm });
    Message.success('规则已创建');
    showWafCreate.value = false;
    Object.assign(wafForm, { name: '', ruleType: 'xss', action: 'block', pattern: '', severity: 'high' });
    await loadWafRules();
  } catch (e: any) { Message.error(e.message); }
}

async function deleteWafRule(id: string) {
  try {
    await rpc.call('security.waf.rules.delete', { id });
    Message.success('已删除');
    await loadWafRules();
  } catch (e: any) { Message.error(e.message); }
}

async function createIdsRule() {
  if (!idsForm.name) { Message.warning('请填写规则名称'); return; }
  try {
    await rpc.call('security.ids.rules.create', { ...idsForm });
    Message.success('规则已创建');
    showIdsCreate.value = false;
    Object.assign(idsForm, { name: '', ruleType: 'portscan', action: 'ban', threshold: '', description: '' });
    await loadIdsRules();
  } catch (e: any) { Message.error(e.message); }
}

async function deleteIdsRule(id: string) {
  try {
    await rpc.call('security.ids.rules.delete', { id });
    Message.success('已删除');
    await loadIdsRules();
  } catch (e: any) { Message.error(e.message); }
}

async function banIp() {
  if (!banIpForm.ip) { Message.warning('请输入 IP'); return; }
  try {
    await rpc.call('security.ids.ban', { ip: banIpForm.ip, duration: parseInt(banIpForm.duration), reason: banIpForm.reason || '手动封禁' });
    Message.success('已封禁');
    showBanIp.value = false;
    Object.assign(banIpForm, { ip: '', duration: '86400', reason: '' });
    await loadBannedIps();
  } catch (e: any) { Message.error(e.message); }
}

async function unbanIp(ip: string) {
  try {
    await rpc.call('security.ids.unban', { ip });
    Message.success('已解封');
    await loadBannedIps();
  } catch (e: any) { Message.error(e.message); }
}

async function addWhiteIp() {
  if (!whiteIpForm.ip) { Message.warning('请输入 IP'); return; }
  try {
    await rpc.call('security.ip.whitelist.add', { ip: whiteIpForm.ip, reason: whiteIpForm.reason });
    Message.success('已添加');
    showWhiteIp.value = false;
    Object.assign(whiteIpForm, { ip: '', reason: '' });
    await loadWhiteIps();
  } catch (e: any) { Message.error(e.message); }
}

async function removeWhiteIp(ip: string) {
  try {
    await rpc.call('security.ip.whitelist.remove', { ip });
    Message.success('已移除');
    await loadWhiteIps();
  } catch (e: any) { Message.error(e.message); }
}

function exportEvents() {
  const data = JSON.stringify(events.value, null, 2);
  const blob = new Blob([data], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `security-events-${dayjs().format('YYYYMMDD-HHmm')}.json`;
  a.click();
  URL.revokeObjectURL(url);
  Message.success(`已导出 ${events.value.length} 条事件`);
}

onMounted(loadAll);
</script>

<style lang="less" scoped>
.ov-card {
  display: flex;
  align-items: center;
  gap: 14px;
  position: relative;
  .ov-icon {
    width: 44px;
    height: 44px;
    border-radius: 10px;
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 20px;
    flex-shrink: 0;
    box-shadow: var(--shadow-xs);
  }
  .ov-num {
    font-size: 22px;
    font-weight: 700;
    color: var(--color-text-1);
  }
  .ov-label {
    font-size: 12px;
    color: var(--color-text-3);
    margin-top: 2px;
  }
  .ov-tag {
    position: absolute;
    top: 12px;
    right: 12px;
  }
}
.rule-name {
  font-weight: 500;
  color: var(--color-text-1);
  font-size: 13px;
}
.rule-pattern {
  font-size: 11px;
  color: var(--color-text-4);
  font-family: monospace;
  margin-top: 2px;
}
</style>
