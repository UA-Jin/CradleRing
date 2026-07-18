import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router';
import { rpc } from '@/api/rpc';

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'login',
    component: () => import('@/pages/login/index.vue'),
    meta: { requiresAuth: false },
  },
  {
    path: '/',
    component: () => import('@/layout/default-layout.vue'),
    redirect: '/dashboard',
    children: [
      // ==================== 核心功能 ====================
      { path: 'dashboard', name: 'DashboardOverview', component: () => import('@/pages/dashboard/overview.vue'), meta: { label: '仪表盘', icon: 'icon-dashboard', order: 1, group: 'core' } },
      { path: 'chat', name: 'Chat', component: () => import('@/pages/chat/index.vue'), meta: { label: '对话', icon: 'icon-message', order: 2, group: 'core' } },
      { path: 'sessions', name: 'Sessions', component: () => import('@/pages/sessions/index.vue'), meta: { label: '会话管理', icon: 'icon-storage', order: 3, group: 'core' } },

      // ==================== AI 能力 ====================
      {
        path: 'ai', redirect: '/memory', meta: { label: 'AI 能力', icon: 'icon-robot', order: 10, group: 'ai' },
        children: [
          { path: '/memory', name: 'Memory', component: () => import('@/pages/memory/index.vue'), meta: { label: '记忆库', icon: 'icon-bookmark' } },
          { path: '/agents', name: 'Agents', component: () => import('@/pages/agents/index.vue'), meta: { label: '角色 Agent', icon: 'icon-user-group' } },
          { path: '/workflows', name: 'Workflows', component: () => import('@/pages/workflows/index.vue'), meta: { label: '工作流引擎', icon: 'icon-mind-mapping' } },
          { path: '/skills', name: 'Skills', component: () => import('@/pages/skills/index.vue'), meta: { label: '技能', icon: 'icon-magic' } },
          { path: '/models', name: 'Models', component: () => import('@/pages/models/index.vue'), meta: { label: '模型', icon: 'icon-robot' } },
        ],
      },

      // ==================== 接入配置 ====================
      {
        path: 'integrations', redirect: '/channels', meta: { label: '接入配置', icon: 'icon-share-internal', order: 20, group: 'integration' },
        children: [
          { path: '/channels', name: 'Channels', component: () => import('@/pages/channels/index.vue'), meta: { label: '渠道', icon: 'icon-share-internal' } },
          { path: '/cron', name: 'Cron', component: () => import('@/pages/cron/index.vue'), meta: { label: '定时任务', icon: 'icon-history' } },
          { path: '/config', name: 'Config', component: () => import('@/pages/config/index.vue'), meta: { label: '系统配置', icon: 'icon-settings' } },
          { path: '/email-config', name: 'EmailConfig', component: () => import('@/pages/email-config/index.vue'), meta: { label: '邮件配置', icon: 'icon-email' } },
        ],
      },

      // ==================== 运维管理 ====================
      {
        path: 'ops', redirect: '/ops-dashboard', meta: { label: '运维管理', icon: 'icon-dashboard', order: 30, group: 'ops' },
        children: [
          { path: '/ops-dashboard', name: 'OpsDashboard', component: () => import('@/pages/ops-dashboard/index.vue'), meta: { label: '监控大屏', icon: 'icon-dashboard' } },
          { path: '/env', name: 'EnvDeploy', component: () => import('@/pages/env/index.vue'), meta: { label: '环境部署', icon: 'icon-common' } },
          { path: '/files', name: 'Files', component: () => import('@/pages/files/index.vue'), meta: { label: '文件管理', icon: 'icon-folder' } },
          { path: '/process', name: 'Process', component: () => import('@/pages/process/index.vue'), meta: { label: '进程管理', icon: 'icon-layers' } },
          { path: '/services', name: 'Services', component: () => import('@/pages/services/index.vue'), meta: { label: '服务管理', icon: 'icon-tool' } },
          { path: '/firewall', name: 'Firewall', component: () => import('@/pages/firewall/index.vue'), meta: { label: '防火墙', icon: 'icon-safe' } },
          { path: '/audit', name: 'Audit', component: () => import('@/pages/audit/index.vue'), meta: { label: '运维审计', icon: 'icon-safe' } },
        ],
      },

      // ==================== 审批中心 ====================
      {
        path: 'approvals', redirect: '/approvals/instances', meta: { label: '审批中心', icon: 'icon-check-circle', order: 40, group: 'approval' },
        children: [
          { path: 'instances', name: 'ApprovalInstances', component: () => import('@/pages/approvals/instances.vue'), meta: { label: '审批实例', icon: 'icon-list' } },
          { path: 'flows', name: 'ApprovalFlows', component: () => import('@/pages/approvals/flows.vue'), meta: { label: '审批流模板', icon: 'icon-mind-mapping' } },
        ],
      },

      // ==================== 系统管理 ====================
      {
        path: 'system', redirect: '/users', meta: { label: '系统管理', icon: 'icon-settings', order: 50, group: 'system' },
        children: [
          { path: '/users', name: 'Users', component: () => import('@/pages/users/index.vue'), meta: { label: '用户管理', icon: 'icon-user' } },
          { path: '/logs', name: 'Logs', component: () => import('@/pages/logs/index.vue'), meta: { label: '日志', icon: 'icon-file' } },
        ],
      },

      // ==================== 个人中心 ====================
      { path: 'profile', name: 'Profile', component: () => import('@/pages/profile/index.vue'), meta: { label: '个人中心', icon: 'icon-user', order: 60, group: 'profile' } },
    ],
  },
  { path: '/:pathMatch(.*)*', redirect: '/dashboard' },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.beforeEach((to, _from, next) => {
  const token = rpc.getToken();
  if (to.meta.requiresAuth === false) {
    if (to.name === 'login' && token) {
      next({ path: '/' });
    } else {
      next();
    }
    return;
  }
  if (!token) {
    next({ path: '/login', query: { redirect: to.fullPath } });
  } else {
    next();
  }
});

export default router;
