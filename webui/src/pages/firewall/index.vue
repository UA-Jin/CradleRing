<template>
  <div class="page-container">
    <a-page-header title="安全中心" subtitle="IDS/IPS 入侵检测 · WAF Web 应用防火墙 · 系统防火墙" :show-back="false">
      <template #extra>
        <a-radio-group v-model="activeTab" type="button">
          <a-radio value="waf">WAF</a-radio>
          <a-radio value="ids">IDS/IPS</a-radio>
          <a-radio value="firewall">系统防火墙</a-radio>
        </a-radio-group>
      </template>
    </a-page-header>

    <!-- ==================== WAF ==================== -->
    <div v-if="activeTab === 'waf'">
      <a-row :gutter="24" class="mt-16">
        <a-col :xs="12" :lg="6"><a-card><a-statistic title="启用规则" :value="wafStats.enabled" :value-style="{ color: '#8c57ff' }" /></a-card></a-col>
        <a-col :xs="12" :lg="6"><a-card><a-statistic title="今日拦截" :value="wafStats.blockedToday" :value-style="{ color: '#ff4c51' }" /></a-card></a-col>
        <a-col :xs="12" :lg="6"><a-card><a-statistic title="总拦截" :value="wafStats.blockedTotal" :value-style="{ color: '#ffb400' }" /></a-card></a-col>
        <a-col :xs="12" :lg="6"><a-card><a-statistic title="命中率" :value="wafStats.hitRate" suffix="%" :value-style="{ color: '#56ca00' }" /></a-card></a-col>
      </a-row>

      <a-card class="mt-16">
        <template #title>WAF 规则（OWASP CRS）</template>
        <template #extra>
          <a-space>
            <a-input v-model="wafSearch" placeholder="搜索规则..." allow-clear style="width: 180px" />
            <a-button type="primary" size="small" @click="showWafAdd = true"><template #icon><icon-plus /></template>添加规则</a-button>
            <a-button size="small" @click="showWafImport = true"><template #icon><icon-upload /></template>导入</a-button>
            <a-button size="small" @click="wafTest"><template #icon><icon-thunderbolt /></template>测试</a-button>
          </a-space>
        </template>
        <a-table :data="filteredWafRules" :pagination="{ pageSize: 15 }" row-key="id">
          <template #columns>
            <a-table-column title="规则" :width="220">
              <template #cell="{ record }">
                <div class="rule-name">{{ record.name }}</div>
                <div class="rule-desc">{{ record.description }}</div>
              </template>
            </a-table-column>
            <a-table-column title="类型" :width="100">
              <template #cell="{ record }"><a-tag :color="wafTypeColor(record.type)" size="small">{{ record.type }}</a-tag></template>
            </a-table-column>
            <a-table-column title="动作" :width="80">
              <template #cell="{ record }">
                <a-tag :color="record.action === 'block' ? 'red' : record.action === 'log' ? 'orange' : 'green'" size="small">{{ record.action }}</a-tag>
              </template>
            </a-table-column>
            <a-table-column title="严重度" :width="90">
              <template #cell="{ record }">
                <a-tag :color="record.severity === 'critical' ? 'red' : record.severity === 'high' ? 'orange' : record.severity === 'medium' ? 'purple' : 'gray'" size="small">{{ record.severity }}</a-tag>
              </template>
            </a-table-column>
            <a-table-column title="命中" :width="80" data-index="hitCount" />
            <a-table-column title="启用" :width="70">
              <template #cell="{ record }">
                <a-switch :model-value="record.enabled" @change="toggleWafRule(record.id)" size="small" />
              </template>
            </a-table-column>
            <a-table-column title="操作" :width="100" fixed="right">
              <template #cell="{ record }">
                <a-space>
                  <a-button size="small" type="text" @click="editWafRule(record)"><icon-edit /></a-button>
                  <a-popconfirm content="确认删除？" @ok="deleteWafRule(record.id)"><a-button size="small" type="text" status="danger"><icon-delete /></a-button></a-popconfirm>
                </a-space>
              </template>
            </a-table-column>
          </template>
        </a-table>
      </a-card>

      <a-card title="最近拦截事件" class="mt-16">
        <a-table :data="wafEvents" :pagination="{ pageSize: 10 }" row-key="id">
          <template #columns>
            <a-table-column title="时间" :width="150"><template #cell="{ record }">{{ dayjs(record.ts).format('MM-DD HH:mm:ss') }}</template></a-table-column>
            <a-table-column title="规则" :width="150" data-index="ruleName" />
            <a-table-column title="来源 IP" :width="130" data-index="clientIp" />
            <a-table-column title="请求" data-index="request" ellipsis />
            <a-table-column title="动作" :width="80">
              <template #cell="{ record }"><a-tag :color="record.action === 'blocked' ? 'red' : 'orange'" size="small">{{ record.action }}</a-tag></template>
            </a-table-column>
          </template>
        </a-table>
      </a-card>
    </div>

    <!-- ==================== IDS/IPS ==================== -->
    <div v-if="activeTab === 'ids'">
      <a-row :gutter="24" class="mt-16">
        <a-col :xs="12" :lg="6"><a-card><a-statistic title="启用规则" :value="idsStats.enabled" :value-style="{ color: '#8c57ff' }" /></a-card></a-col>
        <a-col :xs="12" :lg="6"><a-card><a-statistic title="今日告警" :value="idsStats.alertsToday" :value-style="{ color: '#ffb400' }" /></a-card></a-col>
        <a-col :xs="12" :lg="6"><a-card><a-statistic title="已封禁 IP" :value="idsStats.bannedIps" :value-style="{ color: '#ff4c51' }" /></a-card></a-col>
        <a-col :xs="12" :lg="6"><a-card><a-statistic title="检测引擎" :value="idsStats.engine" :value-style="{ color: '#56ca00' }" /></a-card></a-col>
      </a-row>

      <a-card class="mt-16">
        <template #title>IDS 检测规则</template>
        <template #extra>
          <a-space>
            <a-button type="primary" size="small" @click="idsScan"><template #icon><icon-thunderbolt /></template>立即扫描</a-button>
          </a-space>
        </template>
        <a-table :data="idsRules" :pagination="{ pageSize: 10 }" row-key="id">
          <template #columns>
            <a-table-column title="规则" :width="250">
              <template #cell="{ record }">
                <div class="rule-name">{{ record.name }}</div>
                <div class="rule-desc">{{ record.description }}</div>
              </template>
            </a-table-column>
            <a-table-column title="类型" :width="120">
              <template #cell="{ record }"><a-tag :color="idsTypeColor(record.type)" size="small">{{ record.type }}</a-tag></template>
            </a-table-column>
            <a-table-column title="阈值" :width="80" data-index="threshold" />
            <a-table-column title="动作" :width="80">
              <template #cell="{ record }"><a-tag :color="record.action === 'ban' ? 'red' : 'orange'" size="small">{{ record.action }}</a-tag></template>
            </a-table-column>
            <a-table-column title="命中" :width="70" data-index="hitCount" />
            <a-table-column title="启用" :width="70">
              <template #cell="{ record }"><a-switch :model-value="record.enabled" @change="toggleIdsRule(record.id)" size="small" /></template>
            </a-table-column>
          </template>
        </a-table>
      </a-card>

      <a-card title="最近检测事件" class="mt-16">
        <a-table :data="idsEvents" :pagination="{ pageSize: 10 }" row-key="id">
          <template #columns>
            <a-table-column title="时间" :width="150"><template #cell="{ record }">{{ dayjs(record.ts).format('MM-DD HH:mm:ss') }}</template></a-table-column>
            <a-table-column title="类型" :width="100" data-index="type" />
            <a-table-column title="来源 IP" :width="130" data-index="srcIp" />
            <a-table-column title="详情" data-index="detail" ellipsis />
            <a-table-column title="动作" :width="80">
              <template #cell="{ record }"><a-tag :color="record.action === 'banned' ? 'red' : 'orange'" size="small">{{ record.action }}</a-tag></template>
            </a-table-column>
          </template>
        </a-table>
      </a-card>

      <a-card title="已封禁 IP" class="mt-16">
        <template #extra><a-button size="small" @click="loadBannedIps"><template #icon><icon-refresh /></template>刷新</a-button></template>
        <a-space wrap>
          <a-tag v-for="ip in bannedIps" :key="ip" color="red" closable @close="unbanIp(ip)">{{ ip }}</a-tag>
          <a-empty v-if="!bannedIps.length" description="暂无封禁 IP" />
        </a-space>
      </a-card>
    </div>

    <!-- ==================== 系统防火墙 ==================== -->
    <div v-if="activeTab === 'firewall'">
      <a-row :gutter="24" class="mt-16">
        <a-col :xs="24" :lg="8">
          <a-card class="status-card">
            <div class="status-icon" :class="{ active: fwStatus.active }"><icon-safe /></div>
            <div class="status-info">
              <div class="status-title">{{ fwStatus.active ? '已启用' : '未启用' }}</div>
              <div class="status-desc">{{ fwStatus.backend }} · {{ fwStatus.rules }} 条规则</div>
            </div>
            <a-switch :model-value="fwStatus.active" @change="toggleFirewall" :loading="toggling" />
          </a-card>
        </a-col>
        <a-col :xs="24" :lg="16">
          <a-card title="操作">
            <a-space wrap>
              <a-button type="primary" @click="showAddRule = true"><template #icon><icon-plus /></template>添加规则</a-button>
              <a-button @click="showImport = true"><template #icon><icon-upload /></template>导入规则</a-button>
              <a-button @click="loadFwStatus"><template #icon><icon-refresh /></template>刷新</a-button>
            </a-space>
          </a-card>
        </a-col>
      </a-row>

      <a-card title="iptables / ufw 规则" class="mt-16">
        <a-table :data="fwRules" :pagination="{ pageSize: 15 }" row-key="num" :loading="fwLoading">
          <template #columns>
            <a-table-column title="#" :width="60" data-index="num" />
            <a-table-column title="动作" :width="80">
              <template #cell="{ record }"><a-tag :color="record.action === 'ALLOW' || record.action === 'ACCEPT' ? 'green' : 'red'" size="small">{{ record.action }}</a-tag></template>
            </a-table-column>
            <a-table-column title="协议" :width="70" data-index="protocol" />
            <a-table-column title="来源" :width="150" data-index="source" />
            <a-table-column title="目标" :width="150" data-index="destination" />
            <a-table-column title="端口" :width="90" data-index="port" />
            <a-table-column title="操作" :width="80" fixed="right">
              <template #cell="{ record }">
                <a-popconfirm content="确认删除？" @ok="deleteFwRule(record.num)"><a-button size="small" status="danger" type="text"><icon-delete /></a-button></a-popconfirm>
              </template>
            </a-table-column>
          </template>
        </a-table>
      </a-card>
    </div>

    <!-- WAF 添加规则 -->
    <a-modal :visible="showWafAdd" title="添加 WAF 规则" @cancel="showWafAdd = false" @ok="saveWafRule" :ok-loading="savingWaf" :width="560">
      <a-form :model="wafForm" layout="vertical">
        <a-form-item label="规则名称" required><a-input v-model="wafForm.name" placeholder="如：SQL 注入防护" /></a-form-item>
        <a-row :gutter="12">
          <a-col :span="12"><a-form-item label="类型"><a-select v-model="wafForm.type">
            <a-option value="sqli">SQL 注入</a-option><a-option value="xss">XSS 跨站脚本</a-option>
            <a-option value="rfi">远程文件包含</a-option><a-option value="lfi">本地文件包含</a-option>
            <a-option value="cmd_inject">命令注入</a-option><a-option value="traversal">路径遍历</a-option>
            <a-option value="scanner">扫描器检测</a-option><a-option value="custom">自定义</a-option>
          </a-select></a-form-item></a-col>
          <a-col :span="12"><a-form-item label="动作"><a-select v-model="wafForm.action">
            <a-option value="block">拦截</a-option><a-option value="log">仅记录</a-option><a-option value="allow">放行</a-option>
          </a-select></a-form-item></a-col>
        </a-row>
        <a-form-item label="正则表达式"><a-input v-model="wafForm.pattern" placeholder="匹配规则的正则" /></a-form-item>
        <a-form-item label="严重度"><a-select v-model="wafForm.severity">
          <a-option value="critical">严重</a-option><a-option value="high">高</a-option>
          <a-option value="medium">中</a-option><a-option value="low">低</a-option>
        </a-select></a-form-item>
        <a-form-item label="描述"><a-textarea v-model="wafForm.description" :auto-size="{ minRows: 2 }" /></a-form-item>
      </a-form>
    </a-modal>

    <!-- WAF 导入 -->
    <a-modal :visible="showWafImport" title="导入 WAF 规则" @cancel="showWafImport = false" @ok="importWafRules" :width="560">
      <a-form layout="vertical">
        <a-form-item label="规则内容（JSON 数组）">
          <a-textarea v-model="wafImportText" :auto-size="{ minRows: 10 }" placeholder='[{"name":"...","type":"sqli","pattern":"...","action":"block","severity":"high"}]' style="font-family:monospace;font-size:12px" />
        </a-form-item>
      </a-form>
    </a-modal>

    <!-- iptables 添加规则 -->
    <a-modal :visible="showAddRule" title="添加防火墙规则" @cancel="showAddRule = false" @ok="addFwRule" :width="560">
      <a-form :model="ruleForm" layout="vertical">
        <a-row :gutter="12">
          <a-col :span="12"><a-form-item label="动作"><a-select v-model="ruleForm.action">
            <a-option value="ALLOW">ALLOW</a-option><a-option value="DROP">DROP</a-option><a-option value="REJECT">REJECT</a-option>
          </a-select></a-form-item></a-col>
          <a-col :span="12"><a-form-item label="协议"><a-select v-model="ruleForm.protocol">
            <a-option value="tcp">TCP</a-option><a-option value="udp">UDP</a-option><a-option value="all">全部</a-option>
          </a-select></a-form-item></a-col>
        </a-row>
        <a-row :gutter="12">
          <a-col :span="12"><a-form-item label="来源 IP"><a-input v-model="ruleForm.source" placeholder="0.0.0.0/0" /></a-form-item></a-col>
          <a-col :span="12"><a-form-item label="端口"><a-input v-model="ruleForm.port" placeholder="80" /></a-form-item></a-col>
        </a-row>
        <a-form-item label="备注"><a-input v-model="ruleForm.comment" /></a-form-item>
      </a-form>
    </a-modal>

    <!-- iptables 导入 -->
    <a-modal :visible="showImport" title="导入防火墙规则" @cancel="showImport = false" @ok="importFwRules" :width="640">
      <a-alert type="info" class="mb-16">支持 iptables-save 格式（-A INPUT ...），导入前自动备份</a-alert>
      <a-form layout="vertical">
        <a-form-item label="规则内容">
          <a-textarea v-model="importText" :auto-size="{ minRows: 12 }" placeholder="-A INPUT -p tcp --dport 22 -j ACCEPT" style="font-family:monospace;font-size:12px" />
        </a-form-item>
        <a-form-item><a-checkbox v-model="importReplace">替换模式（清空现有规则）</a-checkbox></a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import dayjs from 'dayjs';
import { Message } from '@arco-design/web-vue';
import { rpc } from '@/api/rpc';
import { IconPlus, IconUpload, IconDelete, IconSafe, IconThunderbolt, IconRefresh, IconEdit } from '@arco-design/web-vue/es/icon';

const activeTab = ref('waf');

// WAF
const wafStats = reactive({ enabled: 0, blockedToday: 0, blockedTotal: 0, hitRate: 0 });
const wafRules = ref<any[]>([]);
const wafEvents = ref<any[]>([]);
const wafSearch = ref('');
const showWafAdd = ref(false);
const showWafImport = ref(false);
const savingWaf = ref(false);
const wafImportText = ref('');
const wafForm = reactive({ name: '', type: 'sqli', action: 'block', pattern: '', severity: 'high', description: '' });

const filteredWafRules = computed(() =>
  wafRules.value.filter((r) => !wafSearch.value || r.name.toLowerCase().includes(wafSearch.value.toLowerCase()) || r.type.includes(wafSearch.value.toLowerCase())),
);

function wafTypeColor(t: string) {
  const map: Record<string, string> = { sqli: 'red', xss: 'orange', rfi: 'purple', lfi: 'purple', cmd_inject: 'red', traversal: 'orange', scanner: 'arcoblue', custom: 'gray' };
  return map[t] || 'gray';
}

async function loadWaf() {
  try {
    const [rules, events, stats] = await Promise.all([
      rpc.call<any>('waf.rules.list'),
      rpc.call<any>('waf.events.list', { limit: 20 }),
      rpc.call<any>('waf.stats'),
    ]);
    wafRules.value = rules.rules || [];
    wafEvents.value = events.events || [];
    Object.assign(wafStats, stats);
  } catch (e: any) { Message.error(e.message); }
}

async function toggleWafRule(id: string) { try { await rpc.call('waf.rules.toggle', { id }); } catch (e: any) { Message.error(e.message); } }
async function saveWafRule() {
  if (!wafForm.name || !wafForm.pattern) { Message.warning('请填写规则名称和正则'); return; }
  savingWaf.value = true;
  try { await rpc.call('waf.rules.create', { ...wafForm }); Message.success('规则已添加'); showWafAdd.value = false; await loadWaf(); }
  catch (e: any) { Message.error(e.message); } finally { savingWaf.value = false; }
}
function editWafRule(record: any) { Object.assign(wafForm, record); showWafAdd.value = true; }
async function deleteWafRule(id: string) { try { await rpc.call('waf.rules.delete', { id }); Message.success('已删除'); await loadWaf(); } catch (e: any) { Message.error(e.message); } }
async function importWafRules() {
  try { const rules = JSON.parse(wafImportText.value); await rpc.call('waf.rules.create', { rules }); Message.success('导入成功'); showWafImport.value = false; wafImportText.value = ''; await loadWaf(); }
  catch (e: any) { Message.error('JSON 格式错误: ' + e.message); }
}
async function wafTest() {
  try { const res = await rpc.call<any>('waf.check', { url: '/test', headers: { 'User-Agent': 'test-agent' }, body: 'test' }); Message.info(`WAF 检测: ${res.matched ? '命中 ' + res.rule : '未命中'}`); }
  catch (e: any) { Message.error(e.message); }
}

// IDS
const idsStats = reactive({ enabled: 0, alertsToday: 0, bannedIps: 0, engine: 'suricata-lite' });
const idsRules = ref<any[]>([]);
const idsEvents = ref<any[]>([]);
const bannedIps = ref<string[]>([]);

function idsTypeColor(t: string) {
  const map: Record<string, string> = { bruteforce: 'red', port_scan: 'orange', malware: 'purple', c2: 'red', ddos: 'orange' };
  return map[t] || 'gray';
}

async function loadIds() {
  try {
    const [rules, events, stats] = await Promise.all([
      rpc.call<any>('ids.rules.list'),
      rpc.call<any>('ids.events.list', { limit: 20 }),
      rpc.call<any>('ids.stats'),
    ]);
    idsRules.value = rules.rules || [];
    idsEvents.value = events.events || [];
    Object.assign(idsStats, stats);
  } catch (e: any) { Message.error(e.message); }
}

async function toggleIdsRule(id: string) { try { await rpc.call('ids.rules.update', { id }); } catch (e: any) { Message.error(e.message); } }
async function idsScan() { try { const res = await rpc.call<any>('ids.scan', { type: 'all' }); Message.info(`扫描完成: ${res.alerts || 0} 个告警`); await loadIds(); } catch (e: any) { Message.error(e.message); } }
async function loadBannedIps() { try { const res = await rpc.call<any>('ids.ban.list'); bannedIps.value = res.ips || []; } catch (e: any) { Message.error(e.message); } }
async function unbanIp(ip: string) { try { await rpc.call('ids.unban', { ip }); Message.success(`${ip} 已解封`); await loadBannedIps(); } catch (e: any) { Message.error(e.message); } }

// 系统防火墙
const fwStatus = reactive({ active: false, backend: 'iptables', rules: 0 });
const fwRules = ref<any[]>([]);
const fwLoading = ref(false);
const toggling = ref(false);
const showAddRule = ref(false);
const showImport = ref(false);
const importText = ref('');
const importReplace = ref(false);
const ruleForm = reactive({ action: 'ALLOW', protocol: 'tcp', source: '', port: '', comment: '' });

async function loadFwStatus() {
  fwLoading.value = true;
  try { const res = await rpc.call<any>('firewall.status'); Object.assign(fwStatus, res); fwRules.value = res.rulesList || []; }
  catch (e: any) { Message.error(e.message); } finally { fwLoading.value = false; }
}
async function toggleFirewall(active: boolean) {
  toggling.value = true;
  try { await rpc.call('firewall.toggle', { enabled: active }); Message.success(active ? '已启用' : '已禁用'); await loadFwStatus(); }
  catch (e: any) { Message.error(e.message); } finally { toggling.value = false; }
}
async function addFwRule() { try { await rpc.call('firewall.add', { ...ruleForm }); Message.success('规则已添加'); showAddRule.value = false; await loadFwStatus(); } catch (e: any) { Message.error(e.message); } }
async function deleteFwRule(num: number) { try { await rpc.call('firewall.delete', { ruleNum: num }); Message.success('已删除'); await loadFwStatus(); } catch (e: any) { Message.error(e.message); } }
async function importFwRules() {
  if (!importText.value.trim()) { Message.warning('请输入规则'); return; }
  try { const res = await rpc.call<any>('firewall.import', { rules: importText.value, replace: importReplace.value }); if (res.ok) { Message.success(`已导入 ${res.imported} 条`); showImport.value = false; await loadFwStatus(); } else { Message.error(res.error); } }
  catch (e: any) { Message.error(e.message); }
}

onMounted(() => { loadWaf(); loadIds(); loadFwStatus(); loadBannedIps(); });
</script>

<style lang="less" scoped>
.rule-name { font-weight: 500; color: var(--color-text-1); font-size: 13px; }
.rule-desc { font-size: 11px; color: var(--color-text-3); margin-top: 2px; }
.status-card {
  display: flex; align-items: center; gap: 16px;
  .status-icon {
    width: 48px; height: 48px; border-radius: 50%; display: flex;
    align-items: center; justify-content: center; font-size: 24px;
    background: #e0dce8; color: #6d6777; flex-shrink: 0;
    &.active { background: linear-gradient(135deg, #56ca00, #82e040); color: #fff; }
  }
  .status-title { font-size: 16px; font-weight: 600; color: var(--color-text-1); }
  .status-desc { font-size: 13px; color: var(--color-text-3); margin-top: 4px; }
}
.mb-16 { margin-bottom: 16px; }
</style>
