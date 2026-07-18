# CradleRing

> 企业级 AI Agent 协作平台 — 多 Agent 编排、多级审批、40+ IM 渠道、IDS/IPS + WAF 安全防护、可视化工作流、环境一键部署、节点集群管理

[![License](https://img.shields.io/badge/license-商业源码许可-blue)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.0.2-green)](https://github.com/UA-Jin/CradleRing/releases)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)](https://github.com/UA-Jin/CradleRing)

## 🚀 快速部署

### 一键安装（推荐）

**Linux / macOS（bash）：**

```bash
curl -fsSL https://raw.githubusercontent.com/UA-Jin/CradleRing/main/install.sh | bash
```

**Windows（PowerShell）：**

```powershell
irm https://raw.githubusercontent.com/UA-Jin/CradleRing/main/install.ps1 | iex
```

一键安装脚本自动完成：
- **Rust 工具链**：自动通过 rustup 安装（Windows 自动下载 rustup-init.exe）
- **C 编译器**：Linux/macOS 自动安装（apt/yum/dnf/apk/pacman）；Windows 自动下载 WinLibs MinGW
- **源码下载**：本地无源码时自动从 GitHub 下载（支持 git clone 或 curl tarball）
- **前端构建**：自动安装 Node.js/pnpm 并构建（npm.cmd 绕过 PowerShell 执行策略）
- **交互式配置**：模型 Provider（20 个国产+国际）/ Embedding（本地 bge-small 或硅基流动 Qwen3-VL）/ 绑定地址 / IM 渠道
- **自启动**：Linux（systemd）、macOS（launchd）、Windows（Task Scheduler）自动注册

全程无需手动安装任何依赖，适合全新服务器一键部署。

### 开放外网访问（绑 0.0.0.0）

默认网关只绑定 `127.0.0.1`（本机访问）。如需局域网/公网访问：

```bash
# 编辑配置
nano ~/.cradle-ring/cradle-ring.json

# 把 "gateway.bind" 改为 "0.0.0.0"（或 "all"）
{
  "gateway": {
    "bind": "0.0.0.0",
    "port": 18800
  }
}

# 重启网关
cradle-ring gateway start
```

⚠️ **安全提示**：开放外网后请确保登录密码强度足够，并配置防火墙只放行可信 IP。建议同时开启登录二步验证。

### 手动安装

```bash
# 1. 克隆仓库
git clone https://github.com/UA-Jin/CradleRing.git
cd CradleRing

# 2. 编译（需要 Rust 1.70+）
cargo build --release

# 3. 安装
cp target/release/cradle-ring ~/.local/bin/

# 4. 启动
cradle-ring gateway start

# 5. 浏览器访问
# http://127.0.0.1:18800
```

### 一键更新 / 卸载

```bash
# 更新到最新版
cradle-ring self-update

# 停止服务
cradle-ring gateway stop

# 卸载（删除二进制 + UI + 数据）
cradle-ring uninstall
```

## ✨ 功能特点

### 🤖 多 Agent 编排引擎

- **角色化 Agent**：7 个预置运维专家角色（DevOps/SRE/DBA/Security/Network/Cloud/Compliance），自定义角色（system prompt + 工具白名单 + 模型覆盖）
- **LangGraph 工作流**：状态图 + 检查点 + 回滚，支持分支/循环/并行/人工审批节点
- **CrewAI 流水线**：多角色 Agent 顺序接力，支持上下文传递
- **子 Agent 并行**：spawn_subagent 并行执行独立任务，自动聚合结果
- **无损上下文压缩**：超长会话自动摘要，保留关键信息不丢失

### 🔒 安全防护（IDS/IPS + WAF）

- **WAF（Web 应用防火墙）**：50+ OWASP CRS 规则（SQL 注入/XSS/路径遍历/命令注入/扫描器/敏感文件/Log4j JNDI），自定义规则 CRUD + 启用禁用 + 事件日志
- **IDS/IPS（入侵检测/防御）**：SSH 暴力破解、端口扫描、恶意软件、C2 外联检测，自动封禁 + 手动封禁/解封 + 规则管理
- **IP 黑白名单**：CIDR 格式，白名单优先放行
- **速率限制**：按 IP 限流，防暴力破解和 DDoS
- **命令审批**：多级审批工作流，危险命令自动触发审批，支持审批流模板（按风险等级路由）
- **登录安全**：失败 5 次锁定 5 分钟，可选二步验证（TOTP 谷歌身份验证器 / 邮件验证码）
- **Webhook 密钥**：渠道回调 URL 带密钥，防伪造消息注入

### 🌐 全渠道接入（40+ IM）

- **40+ 渠道真实连接**：飞书/钉钉/企微/Telegram/Discord/Slack/WhatsApp/Signal/QQ/Matrix/Teams/IRC/Nostr/Twitch/LINE/Mattermost/Nextcloud/Synology/Tlon/Zalo/Google Chat/Rocket.Chat/Zulip/Gitter/XMPP/Mastodon/Twitter/SMS/Viber/KakaoTalk/Threads/Bluesky/Misskey/Wire/Keybase/Threema/Session/Blogger
- **渠道测试真实化**：按渠道类型调平台 API 验证凭据（飞书 tenant_token / Telegram getMe / Discord @me / 钉钉 gettoken / Slack auth.test / WhatsApp），不再假通过
- **Webhook 密钥**：可选启用，回调 URL 带密钥防伪造
- **消息格式**：文本/图片/文件/卡片/按钮，支持 Markdown 渲染

### 📊 可视化运维（节点集群）

- **监控大屏**：世界地图设备分布、延迟趋势、风险排行、主机监控（CPU/内存/磁盘/负载/网络 IO 实时仪表）
- **节点选择**：本机 + 远程 SSH 节点切换监控，离线自动回退本机
- **节点管理**：SSH 节点添加/测试/删除，一键安装代码生成（自动注册 + 心跳上报）
- **环境一键部署**：9 种环境（PHP/NodeJS/Python/Go/Java/Nginx/Redis/MySQL/Docker）自动检测/安装/卸载，支持本地/远程节点
- **文件管理**：远程节点文件浏览/查看/下载/删除/新建目录
- **进程管理**：实时进程列表（CPU/内存排序），支持各节点
- **服务管理**：systemd 服务启停/日志，支持各节点
- **防火墙**：iptables/ufw 规则管理 + 规则导入（自动备份）+ 状态详情

### 🧠 记忆系统 V3（Cache-First + 向量检索 + 时序知识图谱 + 级联路由）

- **Cache-First**：相同问题第二次直接命中缓存（0 成本），L1 精确 + L2 语义（相似度 >0.92）+ L4 向量检索
- **向量检索**：纯 Rust 余弦相似度 + 元数据过滤，Embedding 可选本地（bge-small-zh）或硅基流动（Qwen3-VL-Embedding-8B）
- **时序知识图谱**：实体 + 关系 + 时间维度，多跳推理 BFS，自动从对话抽取实体关系（中文否定检测）
- **级联路由（RouteLLM）**：简单问题走小模型（降本 85%），复杂问题走大模型，质量升级机制
- **多后端冗余**：可选接入 Obsidian/思源笔记/Hindsight/Zep，RRF 结果融合
- **数据集导入**：JSON/JSONL/CSV/TXT 四种格式批量导入，V2→V3 一键迁移

### 🛠️ 内置工具（28+）

- **命令执行**：exec（审批控制）/ read_file / write_file / run_code
- **网络搜索**：15+ 搜索引擎（Google/Bing/Brave/DuckDuckGo/SearXNG/Baidu/360/搜狗/知乎/微信/头条/小红书/B站/维基/微博）
- **文档解析**：read_document（PDF/DOCX/PPTX/XLSX/CSV/TXT/Markdown）
- **图像分析**：analyze_image（多模态视觉模型）
- **语音转写**：transcribe_audio（Whisper API / 本地）
- **网页浏览**：browse / fetch_latest_info / web_search
- **安全工具**：WAF 检测 / 漏洞扫描 / SQLi 扫描 / XSS 扫描 / 端口扫描 / 暴露面分析 / DNS 查询 / SSL 检查 / 子域名枚举
- **运维工具**：文件管理 / 进程管理 / 服务管理 / 防火墙 / SSL 证书 / Docker / Git / 备份 / 主机负载 / 日志分析 / 性能分析

### 👥 多账号权限

- **多用户**：独立账号/密码/角色，细粒度 scopes（chat/sessions/memory/tools/approval/channels/config/users/workflows/logs/admin）
- **角色管理**：预置角色（admin/manager/supervisor/operator/viewer）+ 自定义角色（名称/描述/颜色/权限树）
- **审批工作流**：按角色路由审批（主管→经理→管理员），超时自动拒绝
- **个人中心**：资料/安全（改密码+二步验证）/偏好（主题/语言/折叠）/API Token
- **首次引导**：类似 OpenClaw，首次启用问名字/角色/偏好，自动保存到记忆库

### 🎨 Materialize 设计系统

- **紫色主色**：#8c57ff（Materialize Bootstrap 5 模板完整移植）
- **侧栏**：白色底 + 靶心小圆点切换折叠（双环=展开/单环=折叠），hover 浮出展开，滚轮滑动
- **顶栏**：语言切换 / 主题三态（浅/深/跟随系统）/ 快捷入口 / 真实通知（待审批+系统事件）/ 用户头像下拉
- **卡片**：6px 圆角 + 柔和阴影 + hover 浮起，统计图标彩色圆角方块
- **暗色主题**：深紫黑 #28243d + 毛玻璃 backdrop-filter

## 🏗️ 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                      CradleRing Gateway                     │
├─────────────────────────────────────────────────────────────┤
│  WebSocket JSON-RPC (200+ methods) + HTTP REST              │
│  ├─ WS 认证（hello/connect + JWT/网关 token）              │
│  ├─ Origin 检查（防跨站 WS 攻击）                           │
│  └─ 30s 未认证超时断开                                      │
├─────────────────────────────────────────────────────────────┤
│  Agent Loop                                                 │
│  ├─ Cache-First（L1 精确 → L2 语义 → L4 向量）            │
│  ├─ 记忆召回注入 system prompt                              │
│  ├─ 级联路由（简单→小模型，复杂→大模型）                   │
│  ├─ 工具并行执行（FuturesUnordered）                        │
│  └─ 回答写缓存（下次 0 成本）                               │
├─────────────────────────────────────────────────────────────┤
│  Memory Engine V3                                           │
│  ├─ Embedding（本地哈希 / SiliconFlow API）                │
│  ├─ Vector Store（纯 Rust 余弦 + JSONL 持久化）            │
│  ├─ Temporal Knowledge Graph（实体/关系/时间）              │
│  ├─ Cascading Router（RouteLLM 启发式难度评估）             │
│  └─ Multi-Backend RRF（Obsidian/思源/Hindsight/Zep）        │
├─────────────────────────────────────────────────────────────┤
│  Security Center                                            │
│  ├─ WAF（50+ OWASP CRS 规则 + 自定义 CRUD + 事件日志）      │
│  ├─ IDS/IPS（bruteforce/portscan/malware/C2 + 自动封禁）    │
│  ├─ IP 黑白名单（CIDR）                                     │
│  ├─ Rate Limit（按 IP 限流）                                │
│  └─ Login 2FA（TOTP / 邮件验证码）                          │
├─────────────────────────────────────────────────────────────┤
│  Channel Manager（40+ IM 渠道真实连接 + Webhook 密钥）       │
├─────────────────────────────────────────────────────────────┤
│  Node Manager（SSH 节点 + 一键安装代码 + 心跳上报）           │
├─────────────────────────────────────────────────────────────┤
│  Workflow Engine（LangGraph 状态图 + 检查点 + 回滚）          │
├─────────────────────────────────────────────────────────────┤
│  Approval Engine（多级审批 + 审批流模板 + 超时拒绝）           │
└─────────────────────────────────────────────────────────────┘
```

### 技术栈

- **后端**：Rust 1.70+（单二进制，零外部服务依赖）
- **前端**：Vue 3 + Arco Design Pro + Vite + ECharts
- **存储**：JSON 文件（sessions/messages/memory/config/users/roles）+ JSONL（messages/security_events）
- **向量检索**：纯 Rust 余弦相似度（可选 Qdrant embedded 后续接入）
- **Embedding**：本地哈希降维（零依赖）/ SiliconFlow Qwen3-VL-Embedding-8B
- **TOTP**：纯 Rust Base32 + HMAC-SHA1 + TOTP（零 crate）
- **SMTP**：纯 TCP 手工协议（EHLO/AUTH LOGIN/MAIL FROM/RCPT TO/DATA）

## 📦 安装后配置

### 首次登录

安装完成后，浏览器打开 `http://127.0.0.1:18800`，使用安装时生成的随机凭据登录（保存在 `~/.cradle-ring/data/.admin_credentials`）。

首次登录后进入**首次设置向导**（OpenClaw 风格）：
1. 问名字（怎么称呼你）
2. 问角色（运维/开发/安全/其他）
3. 问偏好（重点帮你做什么）

自动保存到用户资料和记忆库，后续对话会记住你的偏好。

### 配置大模型

**方式一：交互式配置（推荐）**

安装时自动进入配置向导，选择模型 Provider（20 个国产+国际预设），填 API Key，测试连接，保存。

**方式二：配置文件**

编辑 `~/.cradle-ring/cradle-ring.json`：

```json
{
  "providers": {
    "deepseek": {
      "apiKey": "sk-...",
      "baseUrl": "https://api.deepseek.com/v1",
      "model": "deepseek-chat"
    }
  },
  "models": { "primary": "deepseek-chat" }
}
```

**方式三：网关页面**

登录后进入「接入配置 → 系统配置 → 大模型」，20 个服务商预设网格，点击即添加自动填充，只填 Key，测试连接，保存。

### 配置 Embedding（记忆系统）

**本地模式（零成本）**：默认启用，无需配置。首次使用自动下载 bge-small-zh-v1.5（~100MB）。

**API 模式（硅基流动）**：

```json
{
  "memory": {
    "embedding": {
      "provider": "siliconflow",
      "model": "Qwen/Qwen3-VL-Embedding-8B",
      "baseUrl": "https://api.siliconflow.cn/v1",
      "apiKey": "sk-..."
    }
  }
}
```

### 配置 IM 渠道

登录后进入「接入配置 → 渠道」，点击渠道卡片配置：
- **飞书**：App ID + App Secret + Verification Token
- **Telegram**：Bot Token（@BotFather 创建）
- **Discord**：Bot Token
- **钉钉**：App Key + App Secret
- **企业微信**：Corp ID + Agent ID + Secret
- **Slack**：Bot Token + Signing Secret
- **WhatsApp**：Access Token + Phone Number ID
- **其他 30+**：按提示填写

**测试连接**：点击「测试连接」真实调平台 API 验证凭据（不再假通过）。

**Webhook 密钥**：可选启用，回调 URL 带密钥防伪造消息注入。

### 配置二步验证（可选）

登录后进入「个人中心 → 安全 → 二步验证」：
- **TOTP 谷歌身份验证器**：生成密钥 + 扫码添加 → 输入 6 位验证码启用
- **邮件验证码**：配置 SMTP 后，登录时发送 6 位验证码到邮箱

### 配置节点集群

登录后进入「运维管理 → 节点管理」：
1. 添加 SSH 节点（名称/主机/端口/用户/密钥或密码）
2. 测试连接
3. 生成一键安装代码（复制到目标机器执行，自动注册 + 心跳上报）
4. 监控大屏/环境部署/文件管理/进程管理/服务管理均可切换节点查看

## 🔐 安全说明

- **WS 认证**：WebSocket 首帧必须 hello/connect 携带 JWT 或网关 token，未认证拒绝一切 RPC
- **Origin 检查**：浏览器跨站 WS 攻击防护
- **登录限流**：失败 5 次锁定 5 分钟
- **命令注入防护**：所有外部命令参数化调用，输入字符白名单
- **路径穿越防护**：文件管理 canonicalize 真实路径 + 白名单
- **SSRF 防护**：出站请求 DNS 解析后 IP 检查（防云元数据窃取）
- **Webhook 密钥**：渠道回调防伪造注入
- **密码哈希**：salted SHA-256 + 常数时间比较
- **JWT**：手工 HMAC-SHA256，7 天有效，密钥 OsRng 生成
- **敏感信息**：启动日志脱敏，配置不回显密码

## 📄 License

商业源码许可（见 LICENSE 文件）

## 🙏 致谢

- [Materialize](https://themeselection.com/item/materio-dashboard-pro-bootstrap/) — UI 设计系统
- [OpenClaw](https://github.com/openclaw/openclaw) — 产品灵感（本项目是 Rust 1:1 功能复刻）
- [Qdrant](https://qdrant.tech/) — 向量数据库（后续接入）
- [Zep/Graphiti](https://arxiv.org/abs/2501.13956) — 时序知识图谱灵感
- [RouteLLM](https://github.com/lm-sys/RouteLLM) — 级联路由方案
