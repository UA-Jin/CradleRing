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
    redirect: '/dashboard/overview',
    children: [
      {
        path: 'dashboard',
        name: 'dashboard',
        redirect: '/dashboard/overview',
        meta: { label: '仪表盘', icon: 'icon-dashboard' },
        children: [
          { path: 'overview', name: 'DashboardOverview', component: () => import('@/pages/dashboard/overview.vue'), meta: { label: '概览', icon: 'icon-dashboard' } },
          { path: 'workplace', name: 'DashboardWorkplace', component: () => import('@/pages/dashboard/workplace.vue'), meta: { label: '工作台', icon: 'icon-common' } },
        ],
      },
      { path: 'chat', name: 'Chat', component: () => import('@/pages/chat/index.vue'), meta: { label: '对话', icon: 'icon-message' } },
      { path: 'sessions', name: 'Sessions', component: () => import('@/pages/sessions/index.vue'), meta: { label: '会话管理', icon: 'icon-storage' } },
      { path: 'memory', name: 'Memory', component: () => import('@/pages/memory/index.vue'), meta: { label: '记忆库', icon: 'icon-bookmark' } },
      { path: 'channels', name: 'Channels', component: () => import('@/pages/channels/index.vue'), meta: { label: '渠道', icon: 'icon-share-internal' } },
      { path: 'cron', name: 'Cron', component: () => import('@/pages/cron/index.vue'), meta: { label: '定时任务', icon: 'icon-history' } },
      { path: 'skills', name: 'Skills', component: () => import('@/pages/skills/index.vue'), meta: { label: '技能', icon: 'icon-magic' } },
      { path: 'models', name: 'Models', component: () => import('@/pages/models/index.vue'), meta: { label: '模型', icon: 'icon-robot' } },
      { path: 'agents', name: 'Agents', component: () => import('@/pages/agents/index.vue'), meta: { label: '角色 Agent', icon: 'icon-user-group' } },
      { path: 'workflows', name: 'Workflows', component: () => import('@/pages/workflows/index.vue'), meta: { label: '工作流引擎', icon: 'icon-mind-mapping' } },
      { path: 'pipelines', name: 'Pipelines', component: () => import('@/pages/pipelines/index.vue'), meta: { label: '流水线', icon: 'icon-link' } },
      { path: 'audit', name: 'Audit', component: () => import('@/pages/audit/index.vue'), meta: { label: '运维审计', icon: 'icon-safe' } },
      { path: 'ops-dashboard', name: 'OpsDashboard', component: () => import('@/pages/ops-dashboard/index.vue'), meta: { label: '运维大屏', icon: 'icon-dashboard' } },
      {
        path: 'approvals',
        name: 'Approvals',
        redirect: '/approvals/instances',
        meta: { label: '审批中心', icon: 'icon-check-circle' },
        children: [
          { path: 'instances', name: 'ApprovalInstances', component: () => import('@/pages/approvals/instances.vue'), meta: { label: '审批实例', icon: 'icon-list' } },
          { path: 'flows', name: 'ApprovalFlows', component: () => import('@/pages/approvals/flows.vue'), meta: { label: '审批流模板', icon: 'icon-mind-mapping' } },
        ],
      },
      { path: 'users', name: 'Users', component: () => import('@/pages/users/index.vue'), meta: { label: '用户管理', icon: 'icon-user' } },
      { path: 'config', name: 'Config', component: () => import('@/pages/config/index.vue'), meta: { label: '配置', icon: 'icon-settings' } },
      { path: 'logs', name: 'Logs', component: () => import('@/pages/logs/index.vue'), meta: { label: '日志', icon: 'icon-file' } },
      { path: 'settings', name: 'Settings', component: () => import('@/pages/settings/index.vue'), meta: { label: '设置', icon: 'icon-trophy' } },
    ],
  },
  { path: '/:pathMatch(.*)*', redirect: '/dashboard/overview' },
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
