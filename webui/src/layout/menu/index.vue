<template>
  <a-menu
    :selected-keys="selectedKeys"
    :default-open-keys="openKeys"
    @click-menu-item="onSelect"
    :collapsed="collapsed"
    auto-open
    style="flex: 1; border-right: none"
  >
    <template v-for="item in visibleRoutes" :key="item.fullPath">
      <!-- 有子菜单（非折叠状态） -->
      <a-sub-menu v-if="item.children.length > 0 && !collapsed" :key="item.fullPath">
        <template #title>
          <menu-icon :name="item.iconName" />
          {{ item.label }}
        </template>
        <a-menu-item
          v-for="child in item.children"
          :key="child.fullPath"
          @click="onSelect(child.fullPath)"
        >
          <menu-icon :name="child.iconName" />
          {{ child.label }}
        </a-menu-item>
      </a-sub-menu>
      <!-- 普通菜单项 -->
      <a-menu-item v-else :key="item.fullPath" @click="onSelect(item.fullPath)">
        <menu-icon :name="item.iconName" />
        {{ item.label }}
      </a-menu-item>
    </template>
  </a-menu>
</template>

<script setup lang="ts">
import { computed, h, defineComponent } from 'vue';
import { useRouter } from 'vue-router';
import type { RouteRecordRaw } from 'vue-router';
import {
  IconDashboard,
  IconCommon,
  IconMessage,
  IconStorage,
  IconBookmark,
  IconShareInternal,
  IconHistory,
  IconTool,
  IconRobot,
  IconUserGroup,
  IconCheckCircle,
  IconList,
  IconMindMapping,
  IconUser,
  IconSettings,
  IconFile,
  IconTrophy,
  IconLink,
  IconSafe,
  IconFolder,
  IconLayers,
  IconEmail,
  IconThunderbolt,
} from '@arco-design/web-vue/es/icon';

const props = defineProps<{
  collapsed?: boolean;
  routes?: RouteRecordRaw[];
  isAdmin?: boolean;
}>();

const router = useRouter();

// 图标组件封装：通过 name 渲染对应图标
const MenuIcon = defineComponent({
  name: 'MenuIcon',
  props: { name: { type: String, default: '' } },
  setup(props) {
    const map: Record<string, any> = {
      'icon-dashboard': IconDashboard,
      'icon-common': IconCommon,
      'icon-message': IconMessage,
      'icon-storage': IconStorage,
      'icon-bookmark': IconBookmark,
      'icon-share-internal': IconShareInternal,
      'icon-history': IconHistory,
      'icon-magic': IconThunderbolt,
      'icon-tool': IconTool,
      'icon-robot': IconRobot,
      'icon-user-group': IconUserGroup,
      'icon-check-circle': IconCheckCircle,
      'icon-list': IconList,
      'icon-mind-mapping': IconMindMapping,
      'icon-user': IconUser,
      'icon-settings': IconSettings,
      'icon-file': IconFile,
      'icon-trophy': IconTrophy,
      'icon-link': IconLink,
      'icon-safe': IconSafe,
      'icon-folder': IconFolder,
      'icon-layers': IconLayers,
      'icon-email': IconEmail,
      'icon-thunderbolt': IconThunderbolt,
    };
    return () => {
      const Comp = map[props.name];
      return Comp ? h(Comp) : null;
    };
  },
});

interface MenuItem {
  fullPath: string;
  label: string;
  iconName: string;
  children: MenuItem[];
}

function isAuth(meta: any, isAdmin: boolean) {
  if (!meta) return true;
  if (meta.adminOnly && !isAdmin) return false;
  return true;
}

function buildMenu(routes: RouteRecordRaw[], parentPath = ''): MenuItem[] {
  return routes
    .filter((r) => r.meta?.label && isAuth(r.meta, props.isAdmin))
    .map((r) => {
      // 子路由用绝对路径（/memory）时直接使用，否则拼接 parentPath
      const fullPath = r.path.startsWith('/') ? r.path : (parentPath + '/' + r.path).replace(/\/+/g, '/');
      return {
        fullPath,
        label: r.meta?.label as string,
        iconName: (r.meta?.icon as string) || '',
        children: buildMenu(r.children || [], fullPath),
      };
    });
}

const menuRoutes = computed(() => {
  if (props.routes) return props.routes;
  const root = router.options.routes.find((r) => r.path === '/');
  return root?.children || [];
});

const visibleRoutes = computed(() => buildMenu(menuRoutes.value));

const selectedKeys = computed(() => [router.currentRoute.value.path]);

const openKeys = computed(() =>
  menuRoutes.value
    .filter((r) => r.children && r.children.some((c) => router.currentRoute.value.path.startsWith('/' + r.path)))
    .map((r) => '/' + r.path),
);

function onSelect(key: string) {
  router.push(key);
}
</script>
