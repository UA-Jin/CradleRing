/**
 * CradleRing RPC 客户端
 * - REST 登录（/api/login）
 * - WebSocket JSON-RPC（后端实际协议：connect.challenge 事件 → 直接发 RPC）
 * - 事件订阅（broadcast 推送）
 *
 * 后端协议：
 *   连接后端会发送 {"type":"event","event":"connect.challenge","payload":{nonce,ts}}
 *   客户端发送 {"method":"<rpc>","id":"<str>","params":{...}}
 *   后端响应 {"type":"res","id":"<str>","ok":true,"payload":<result>}
 *   后端推送 {"type":"event","event":"<name>","payload":{...}}
 */

export interface User {
  id: string;
  username: string;
  displayName: string;
  email?: string;
  role: string;
  scopes: string[];
  agentId: string;
  enabled: boolean;
  approvalEnabled: boolean;
  createdAt: number;
  lastLogin?: number;
}

type EventHandler = (payload: any) => void;

class RpcClient {
  private ws: WebSocket | null = null;
  private nextId = 1;
  private pending = new Map<string, { resolve: (v: any) => void; reject: (e: any) => void; timer: any }>();
  private eventHandlers = new Map<string, Set<EventHandler>>();
  private token = '';
  private connected = false;
  private connecting: Promise<void> | null = null;
  private reconnectAttempts = 0;
  private ready = false;
  private authenticated = false;

  /** REST 登录 */
  async login(username: string, password: string): Promise<{ token: string; user: User }> {
    const res = await fetch('/api/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password }),
    });
    const data = await res.json().catch(() => ({}));
    if (!data.ok) {
      throw new Error(data.error?.message || '登录失败');
    }
    this.token = data.token;
    localStorage.setItem('cradle_token', this.token);
    localStorage.setItem('cradle_user', JSON.stringify(data.user));
    // 关键修复：登录成功后，若 WS 已连接（之前以空 token hello 失败），重新认证
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.reauthenticate().catch(() => {});
    }
    return { token: data.token, user: data.user as User };
  }

  /** 重新认证（token 更新后重新 hello） */
  async reauthenticate(): Promise<boolean> {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return false;
    const gwToken = await this.fetchGatewayToken();
    const id = `hello-${this.nextId++}`;
    return new Promise<boolean>((resolve) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        resolve(false);
      }, 5000);
      this.pending.set(id, {
        resolve: () => {
          clearTimeout(timer);
          this.authenticated = true;
          resolve(true);
        },
        reject: () => {
          clearTimeout(timer);
          this.authenticated = false;
          resolve(false);
        },
        timer,
      });
      this.ws!.send(
        JSON.stringify({
          jsonrpc: '2.0',
          method: 'hello',
          params: { token: gwToken, auth_token: this.getToken(), client: 'cradle-webui', version: '1.0' },
          id,
        }),
      );
    });
  }

  logout() {
    this.token = '';
    localStorage.removeItem('cradle_token');
    localStorage.removeItem('cradle_user');
    if (this.ws) {
      try { this.ws.close(); } catch { /* ignore */ }
      this.ws = null;
    }
    this.connected = false;
    this.ready = false;
    this.authenticated = false;
  }

  getToken(): string {
    if (!this.token) {
      this.token = localStorage.getItem('cradle_token') || '';
    }
    return this.token;
  }

  getStoredUser(): User | null {
    const raw = localStorage.getItem('cradle_user');
    if (!raw) return null;
    try {
      return JSON.parse(raw) as User;
    } catch {
      return null;
    }
  }

  /** 获取网关 token（用于标识网关，非鉴权） */
  private async fetchGatewayToken(): Promise<string> {
    try {
      const res = await fetch('/api/token');
      const data = await res.json();
      return data.token || '';
    } catch {
      return '';
    }
  }

  /** 连接 WebSocket */
  async connect(): Promise<void> {
    if (this.ready && this.ws?.readyState === WebSocket.OPEN) return;
    if (this.connecting) return this.connecting;
    this.connecting = this._doConnect();
    try {
      await this.connecting;
    } finally {
      this.connecting = null;
    }
  }

  private async _doConnect(): Promise<void> {
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${proto}//${location.host}/ws`;

    return new Promise((resolve, reject) => {
      let opened = false;
      const ws = new WebSocket(wsUrl);
      this.ws = ws;
      ws.binaryType = 'arraybuffer';

      ws.onopen = () => {
        opened = true;
        this.connected = true;
        // 发送 hello（作为 pending 调用，可追踪认证结果）
        this.reauthenticate().then((ok) => {
          this.authenticated = ok;
          if (!ok && this.getToken()) {
            // 本地有 token 但认证失败 → token 失效，清除并跳登录
            this.token = '';
            localStorage.removeItem('cradle_token');
            localStorage.removeItem('cradle_user');
            if (location.pathname !== '/login') {
              location.href = '/login';
            }
          }
        });
        // 标记就绪（不等待 hello-ok，RPC 遇到 UNAUTHORIZED 会自动重试）
        this.ready = true;
        this.reconnectAttempts = 0;
        resolve();
      };

      ws.onmessage = (ev) => {
        let data: any;
        try {
          if (ev.data instanceof ArrayBuffer) {
            data = JSON.parse(new TextDecoder().decode(ev.data));
          } else {
            data = JSON.parse(ev.data);
          }
        } catch {
          return;
        }
        this.handleMessage(data);
      };

      ws.onerror = () => {
        if (!opened) reject(new Error('WebSocket 连接失败，请检查网关是否运行'));
      };

      ws.onclose = () => {
        this.connected = false;
        this.ready = false;
        this.ws = null;
        // 自动重连（指数退避，最多 30 秒）
        this.reconnectAttempts++;
        const delay = Math.min(1000 * 2 ** Math.min(this.reconnectAttempts, 5), 30000);
        setTimeout(() => this.connect().catch(() => {}), delay);
      };
    });
  }

  private handleMessage(data: any) {
    // 广播事件：{"type":"event","event":"<name>","payload":{...}}
    if (data.type === 'event' && data.event) {
      const handlers = this.eventHandlers.get(data.event);
      if (handlers) handlers.forEach((h) => h(data.payload));
      this.eventHandlers.get('*')?.forEach((h) => h({ event: data.event, payload: data.payload }));
      return;
    }
    // RPC 响应：{"type":"res","id":"<str>","ok":true,"payload":<result>}
    if (data.type === 'res' && data.id && this.pending.has(String(data.id))) {
      const p = this.pending.get(String(data.id))!;
      this.pending.delete(String(data.id));
      clearTimeout(p.timer);
      if (data.ok === false || data.error) {
        p.reject(new Error(data.error?.message || data.payload?.error?.message || 'RPC 错误'));
      } else {
        p.resolve(data.payload);
      }
    }
  }

  /** 全局 sudo 密码缓存（用户输入一次后后续请求自动带上） */
  private sudoPassword = '';

  /** 弹窗请求 sudo 密码（用 Arco 的 Modal.prompt 不可用，改用原生 prompt + 动态 import） */
  private async requestSudoPassword(): Promise<string> {
    // 用 Arco Design 的 Modal 输入框
    const { Modal, Input } = await import('@arco-design/web-vue');
    return new Promise<string>((resolve) => {
      let inputValue = '';
      const app = document.createElement('div');
      document.body.appendChild(app);
      const modal = Modal.open({
        title: '🔐 需要管理员密码',
        content: '此操作需要 sudo 权限，请输入管理员密码：',
        okText: '确认',
        cancelText: '取消',
        modalClass: 'sudo-modal',
        onOk: () => {
          resolve(inputValue);
          modal.close();
        },
        onCancel: () => {
          resolve('');
          modal.close();
        },
      });
      // 动态插入密码输入框
      setTimeout(() => {
        const body = document.querySelector('.sudo-modal .arco-modal-body');
        if (body) {
          const wrapper = document.createElement('div');
          wrapper.style.marginTop = '12px';
          const input = document.createElement('input');
          input.type = 'password';
          input.placeholder = 'sudo 密码';
          input.style.width = '100%';
          input.style.padding = '8px 12px';
          input.style.borderRadius = '6px';
          input.style.border = '1px solid var(--color-border-2)';
          input.style.fontSize = '14px';
          input.style.background = 'var(--color-bg-2)';
          input.style.color = 'var(--color-text-1)';
          input.addEventListener('input', (e) => { inputValue = (e.target as HTMLInputElement).value; });
          input.addEventListener('keydown', (e) => { if (e.key === 'Enter') { resolve(inputValue); modal.close(); } });
          wrapper.appendChild(input);
          body.appendChild(wrapper);
          input.focus();
        }
      }, 100);
    });
  }

  /** 调用 RPC 方法（遇到 UNAUTHORIZED 自动重新认证；遇到 NEED_SUDO_PASSWORD 弹窗输入密码重试） */
  async call<T = any>(method: string, params: Record<string, any> = {}): Promise<T> {
    try {
      // 如果有缓存的 sudo 密码，自动带上
      const finalParams = this.sudoPassword ? { ...params, sudoPassword: this.sudoPassword } : params;
      return await this._callOnce<T>(method, finalParams);
    } catch (e: any) {
      const msg = String(e?.message || '');
      // sudo 密码请求：弹窗输入后重试
      if (msg.includes('NEED_SUDO_PASSWORD') || msg.includes('管理员密码')) {
        const pwd = await this.requestSudoPassword();
        if (pwd) {
          this.sudoPassword = pwd;
          return await this._callOnce<T>(method, { ...params, sudoPassword: pwd });
        }
        throw new Error('操作已取消（需要 sudo 密码）');
      }
      // 认证失败：重新认证后重试
      if (msg.includes('未认证') || msg.includes('UNAUTHORIZED') || msg.includes('AUTH_FAILED')) {
        const ok = await this.reauthenticate();
        if (ok) {
          return await this._callOnce<T>(method, params);
        }
        if (!this.getToken() && location.pathname !== '/login') {
          location.href = '/login';
        }
      }
      throw e;
    }
  }

  private async _callOnce<T = any>(method: string, params: Record<string, any> = {}): Promise<T> {
    await this.connect();
    const id = String(this.nextId++);
    return new Promise<T>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`RPC 超时: ${method}`));
      }, 60000);
      this.pending.set(id, {
        resolve: (v) => resolve(v as T),
        reject,
        timer,
      });
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        clearTimeout(timer);
        this.pending.delete(id);
        reject(new Error('WebSocket 未连接'));
        return;
      }
      this.ws.send(JSON.stringify({ jsonrpc: '2.0', method, params, id }));
    });
  }

  /** 订阅事件 */
  on(event: string, handler: EventHandler): () => void {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set());
    }
    this.eventHandlers.get(event)!.add(handler);
    return () => {
      this.eventHandlers.get(event)?.delete(handler);
    };
  }

  get isConnected() {
    return this.ready;
  }
}

export const rpc = new RpcClient();
