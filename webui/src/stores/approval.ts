import { defineStore } from 'pinia';
import { ref } from 'vue';
import { rpc } from '@/api/rpc';

export interface ApprovalStep {
  order: number;
  name: string;
  approverRole: string;
  approverIds: string[];
  notifyChannels: string[];
  notifyTargets: string[];
  autoApproveAfterSecs?: number;
  requireAll: boolean;
}

export interface ApprovalFlow {
  id: string;
  name: string;
  triggerPatterns: string[];
  kinds: string[];
  steps: ApprovalStep[];
  enabled: boolean;
  createdBy: string;
  createdAt: number;
}

export interface ApprovalDecision {
  stepOrder: number;
  approverId: string;
  approverUsername: string;
  decision: string;
  comment: string;
  decidedAt: number;
  viaChannel: string;
}

export interface ApprovalInstance {
  id: string;
  flowId: string;
  flowName: string;
  title: string;
  description: string;
  command: string;
  kind: string;
  requestedBy: string;
  requestedUsername: string;
  currentStep: number;
  totalSteps: number;
  status: string;
  decisions: ApprovalDecision[];
  createdAt: number;
  updatedAt: number;
  sessionKey: string;
  asyncNonBlocking: boolean;
  executionResult?: string;
  completedAt?: number;
}

export const useApprovalStore = defineStore('approval', () => {
  const flows = ref<ApprovalFlow[]>([]);
  const instances = ref<ApprovalInstance[]>([]);
  const stats = ref({ total: 0, pending: 0, approved: 0, rejected: 0, timeout: 0, completed: 0, flowsCount: 0 });
  const loading = ref(false);

  async function loadFlows() {
    loading.value = true;
    try {
      const res = await rpc.call<{ flows: ApprovalFlow[]; count: number }>('approval.flows.list');
      flows.value = res.flows || [];
    } finally {
      loading.value = false;
    }
  }

  async function loadInstances(status?: string) {
    loading.value = true;
    try {
      const params: Record<string, any> = {};
      if (status) params.status = status;
      const res = await rpc.call<{ instances: ApprovalInstance[]; count: number }>('approval.instances.list', params);
      instances.value = res.instances || [];
    } finally {
      loading.value = false;
    }
  }

  async function loadStats() {
    const res = await rpc.call<any>('approval.stats');
    stats.value = res;
  }

  async function createFlow(data: Partial<ApprovalFlow>) {
    const res = await rpc.call<{ ok: boolean; flow: ApprovalFlow }>('approval.flows.create', data as any);
    await loadFlows();
    return res;
  }

  async function updateFlow(id: string, data: Partial<ApprovalFlow>) {
    const res = await rpc.call<{ ok: boolean; flow: ApprovalFlow }>('approval.flows.update', { id, ...data });
    await loadFlows();
    return res;
  }

  async function deleteFlow(id: string) {
    await rpc.call('approval.flows.delete', { id });
    await loadFlows();
  }

  async function approveInstance(id: string, comment = '') {
    const res = await rpc.call<{ ok: boolean; instance: ApprovalInstance }>('approval.instances.approve', {
      id, comment, approverUsername: rpc.getStoredUser()?.displayName || '用户',
    });
    return res;
  }

  async function rejectInstance(id: string, comment = '') {
    const res = await rpc.call<{ ok: boolean; instance: ApprovalInstance }>('approval.instances.reject', {
      id, comment, approverUsername: rpc.getStoredUser()?.displayName || '用户',
    });
    return res;
  }

  async function cancelInstance(id: string) {
    await rpc.call('approval.instances.cancel', { id });
  }

  async function createInstance(data: Partial<ApprovalInstance>) {
    const res = await rpc.call<{ ok: boolean; instance: ApprovalInstance }>('approval.instances.create', data as any);
    return res;
  }

  return {
    flows, instances, stats, loading,
    loadFlows, loadInstances, loadStats,
    createFlow, updateFlow, deleteFlow,
    approveInstance, rejectInstance, cancelInstance, createInstance,
  };
});
