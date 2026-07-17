<template>
  <div class="chat-page">
    <!-- 会话侧栏 -->
    <div class="session-sidebar">
      <div class="sidebar-header">
        <a-button type="primary" long @click="newSession">
          <template #icon><icon-plus /></template>
          新建会话
        </a-button>
      </div>
      <a-input v-model="searchKey" placeholder="搜索会话..." allow-clear class="sidebar-search" />
      <div class="session-list">
        <div
          v-for="s in filteredSessions"
          :key="s.key"
          class="session-item"
          :class="{ active: s.key === currentKey }"
          @click="selectSession(s.key)"
        >
          <div class="si-title">{{ s.displayName || s.key }}</div>
          <div class="si-sub">{{ s.kind }} · {{ dayjs(s.updatedAt).fromNow() }}</div>
        </div>
        <a-empty v-if="!filteredSessions.length" description="暂无会话" />
      </div>
    </div>

    <!-- 主聊天区 -->
    <div class="chat-main">
      <div class="chat-header">
        <div>
          <a-tag :color="currentSession ? 'arcoblue' : 'gray'">{{ currentSession?.kind || '未选择' }}</a-tag>
          <span class="chat-title">{{ currentSession?.displayName || currentKey || '请选择或创建会话' }}</span>
        </div>
        <a-space>
          <a-tooltip content="压缩会话（无损摘要）">
            <a-button type="text" shape="circle" :disabled="!currentKey" @click="compact">
              <icon-compress />
            </a-button>
          </a-tooltip>
          <a-tooltip content="清空当前会话消息">
            <a-popconfirm content="确认清空该会话的所有消息？" @ok="clearMessages">
              <a-button type="text" shape="circle" :disabled="!currentKey"><icon-delete /></a-button>
            </a-popconfirm>
          </a-tooltip>
        </a-space>
      </div>

      <div class="messages" ref="messagesEl">
        <div v-for="m in messages" :key="m.ts" class="msg" :class="m.role">
          <a-avatar :size="32" :style="{ backgroundColor: m.role === 'user' ? '#165dff' : '#00b42a' }">
            {{ m.role === 'user' ? '我' : 'AI' }}
          </a-avatar>
          <div class="msg-body">
            <div class="msg-role">{{ m.role === 'user' ? '我' : '助手' }}</div>
            <div class="msg-content" v-html="renderMd(m.content)"></div>
          </div>
        </div>
        <div v-if="sending" class="msg assistant">
          <a-avatar :size="32" :style="{ backgroundColor: '#00b42a' }">AI</a-avatar>
          <div class="msg-body">
            <div class="msg-role">助手 <a-spin dot :size="14" style="display:inline-block;margin-left:8px" /></div>
            <div class="msg-content typing">思考中...</div>
          </div>
        </div>
      </div>

      <div class="input-area">
        <a-textarea
          v-model="input"
          :auto-size="{ minRows: 1, maxRows: 6 }"
          placeholder="输入消息，按 Enter 发送，Shift+Enter 换行"
          @keydown.enter.exact.prevent="send"
          :disabled="!currentKey || sending"
        />
        <a-button type="primary" :loading="sending" :disabled="!currentKey || !input.trim()" @click="send">
          <template #icon><icon-send /></template>
          发送
        </a-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import 'dayjs/locale/zh-cn';
import { rpc } from '@/api/rpc';
import { Message } from '@arco-design/web-vue';

dayjs.extend(relativeTime);
dayjs.locale('zh-cn');

interface SessionInfo { key: string; kind: string; displayName?: string; updatedAt: number; }
interface ChatMsg { role: string; content: string; ts: number; }

const sessions = ref<SessionInfo[]>([]);
const searchKey = ref('');
const currentKey = ref('');
const currentSession = computed(() => sessions.value.find((s) => s.key === currentKey.value));
const filteredSessions = computed(() =>
  sessions.value.filter((s) =>
    !searchKey.value || (s.displayName || s.key).toLowerCase().includes(searchKey.value.toLowerCase()),
  ),
);

const messages = ref<ChatMsg[]>([]);
const input = ref('');
const sending = ref(false);
const messagesEl = ref<HTMLElement>();

function renderMd(s: string): string {
  // 极简 markdown：代码块、粗体、换行
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/```(\w*)\n?([\s\S]*?)```/g, (_, lang, code) => `<pre class="code-block"><code>${code.trim()}</code></pre>`)
    .replace(/`([^`]+)`/g, '<code class="inline-code">$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\n/g, '<br>');
}

async function loadSessions() {
  try {
    const res = await rpc.call<{ sessions: SessionInfo[] }>('sessions.list');
    sessions.value = (res.sessions || []).sort((a, b) => b.updatedAt - a.updatedAt);
    if (!currentKey.value && sessions.value.length) {
      await selectSession(sessions.value[0].key);
    }
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function selectSession(key: string) {
  currentKey.value = key;
  try {
    const res = await rpc.call<{ messages: ChatMsg[] }>('sessions.messages', { sessionKey: key });
    messages.value = (res.messages || []).map((m: any) => ({ role: m.role, content: m.content, ts: m.timestamp }));
    await scrollBottom();
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function newSession() {
  const key = `web-${Date.now().toString(36)}`;
  try {
    await rpc.call('sessions.create', {
      key, kind: 'web', displayName: `新会话 ${dayjs().format('MM-DD HH:mm')}`, agentId: 'main',
    });
    await loadSessions();
    await selectSession(key);
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function send() {
  if (!currentKey.value || !input.value.trim()) return;
  const text = input.value.trim();
  input.value = '';
  sending.value = true;
  messages.value.push({ role: 'user', content: text, ts: Date.now() });
  await scrollBottom();

  try {
    const res = await rpc.call<{ assistantMessage?: string; reply?: string; error?: string }>(
      'sessions.chat',
      { sessionKey: currentKey.value, message: text },
    );
    const reply = res.assistantMessage || res.reply || res.error || '(无回复)';
    messages.value.push({ role: 'assistant', content: reply, ts: Date.now() });
    await scrollBottom();
  } catch (e: any) {
    messages.value.push({ role: 'assistant', content: `错误：${e.message}`, ts: Date.now() });
  } finally {
    sending.value = false;
  }
}

async function compact() {
  try {
    await rpc.call('sessions.compact', { sessionKey: currentKey.value });
    Message.success('已压缩');
    await selectSession(currentKey.value);
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function clearMessages() {
  try {
    await rpc.call('sessions.clear', { sessionKey: currentKey.value });
    messages.value = [];
    Message.success('已清空');
  } catch (e: any) {
    Message.error(e.message);
  }
}

async function scrollBottom() {
  await nextTick();
  if (messagesEl.value) messagesEl.value.scrollTop = messagesEl.value.scrollHeight;
}

let unsub: (() => void) | null = null;
onMounted(async () => {
  await loadSessions();
  // 订阅流式更新
  unsub = rpc.on('chat.stream.chunk', (p: any) => {
    if (p.sessionKey === currentKey.value) {
      // 追加到最新 assistant 消息
      const last = messages.value[messages.value.length - 1];
      if (last && last.role === 'assistant') {
        last.content += p.text || '';
      } else {
        messages.value.push({ role: 'assistant', content: p.text || '', ts: Date.now() });
      }
      scrollBottom();
    }
  });
});

watch(currentKey, () => scrollBottom());
</script>

<style lang="less" scoped>
.chat-page {
  display: flex;
  height: calc(100vh - var(--navbar-height));
}

.session-sidebar {
  width: 280px;
  background-color: var(--color-bg-1);
  border-right: 1px solid var(--color-border-1);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 12px;
}
.sidebar-search {
  margin: 0 12px 8px;
  width: calc(100% - 24px);
}

.session-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px 8px;
}

.session-item {
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  margin-bottom: 4px;
  &:hover { background-color: var(--color-bg-3); }
  &.active { background-color: rgb(var(--primary-1, 232, 243, 255)); }

  .si-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-1);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .si-sub {
    font-size: 11px;
    color: var(--color-text-3);
    margin-top: 2px;
  }
}

.chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.chat-header {
  padding: 12px 20px;
  border-bottom: 1px solid var(--color-border-1);
  background-color: var(--color-bg-1);
  display: flex;
  justify-content: space-between;
  align-items: center;
  .chat-title { margin-left: 8px; font-weight: 500; color: var(--color-text-1); }
}

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.msg {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
  max-width: 80%;
  &.user { margin-left: auto; flex-direction: row-reverse; }
}

.msg-body {
  flex: 1;
  min-width: 0;
}

.msg-role {
  font-size: 12px;
  color: var(--color-text-3);
  margin-bottom: 4px;
}

.msg-content {
  background-color: var(--color-bg-1);
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 14px;
  line-height: 1.6;
  color: var(--color-text-1);
  word-break: break-word;
  .user & {
    background-color: rgb(var(--primary-6, 20, 120, 252));
    color: #fff;
  }
  :deep(.code-block) {
    background-color: var(--color-bg-3);
    padding: 12px;
    border-radius: 4px;
    overflow-x: auto;
    margin: 8px 0;
  }
  :deep(.inline-code) {
    background-color: var(--color-bg-3);
    padding: 2px 6px;
    border-radius: 3px;
    font-family: monospace;
  }
}

.typing {
  color: var(--color-text-3);
  font-style: italic;
}

.input-area {
  padding: 16px 20px;
  border-top: 1px solid var(--color-border-1);
  background-color: var(--color-bg-1);
  display: flex;
  gap: 12px;
  align-items: flex-end;
}
</style>
