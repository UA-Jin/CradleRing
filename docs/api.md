# API 文档

WebSocket JSON-RPC 197+ 方法。

## 连接

```
ws://127.0.0.1:18800/ws
```

## 认证

```
POST /api/login
{"username": "admin_xxx", "password": "xxx"}
```

## 主要方法

- `sessions.*` - 会话管理
- `chat.*` - 对话
- `workflow.*` - 工作流
- `approval.*` - 审批
- `users.*` - 用户
- `roles.*` - 角色
- `waf.*` - WAF 安全
- `dashboard.*` - 大屏指标
