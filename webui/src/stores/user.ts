import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { rpc, type User } from '@/api/rpc';

export const useUserStore = defineStore('user', () => {
  const user = ref<User | null>(rpc.getStoredUser());
  const loading = ref(false);

  const isLogin = computed(() => !!user.value);
  const role = computed(() => user.value?.role || 'viewer');
  const isAdmin = computed(() => role.value === 'admin');
  const canApprove = computed(() =>
    ['admin', 'manager', 'supervisor'].includes(role.value),
  );

  async function login(username: string, password: string) {
    loading.value = true;
    try {
      const { user: u } = await rpc.login(username, password);
      user.value = u;
      return u;
    } finally {
      loading.value = false;
    }
  }

  function logout() {
    rpc.logout();
    user.value = null;
  }

  function setUser(u: User | null) {
    user.value = u;
    if (u) localStorage.setItem('cradle_user', JSON.stringify(u));
  }

  async function refresh() {
    try {
      const res = await rpc.call<{ user: User }>('users.me', { token: rpc.getToken() });
      if (res?.user) setUser(res.user);
    } catch {
      /* ignore */
    }
  }

  return { user, loading, isLogin, role, isAdmin, canApprove, login, logout, setUser, refresh };
});
