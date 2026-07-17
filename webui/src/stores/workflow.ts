import { defineStore } from 'pinia';
import { ref } from 'vue';
import { rpc } from '@/api/rpc';

export interface WorkflowNode {
  id: string;
  name: string;
  nodeType: string;
  agentRole?: string;
  promptTemplate?: string;
  toolName?: string;
  toolArgsTemplate?: any;
  branches?: { expr: string; target: string }[];
  defaultEdge?: string;
  fanOutField?: string;
  fanOutRole?: string;
  maxConcurrent?: number;
  reduceMode?: string;
  prompt?: string;
  outputField?: string;
}

export interface WorkflowEdge {
  id: string;
  from: string;
  to: string;
  condition?: string;
  label?: string;
}

export interface WorkflowGraph {
  id: string;
  name: string;
  description: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  entryNode: string;
  stateSchema: string[];
  enabled: boolean;
  createdAt: number;
}

export interface WorkflowRun {
  id: string;
  graphId: string;
  graphName: string;
  state: any;
  currentNode: string;
  status: string;
  checkpointsCount: number;
  startedAt: number;
  finishedAt?: number;
  error?: string;
  breakpoints: string[];
}

export interface AgentRole {
  id: string;
  name: string;
  role: string;
  goal: string;
  backstory: string;
  tools?: string[];
  model?: string;
  maxIterations: number;
  allowDelegation: boolean;
  createdAt: number;
}

export interface Pipeline {
  id: string;
  name: string;
  description: string;
  stages: { order: number; agentRoleId: string; taskTemplate: string }[];
  passThrough: boolean;
  enabled: boolean;
}

export const useWorkflowStore = defineStore('workflow', () => {
  const graphs = ref<WorkflowGraph[]>([]);
  const runs = ref<WorkflowRun[]>([]);
  const roles = ref<AgentRole[]>([]);
  const pipelines = ref<Pipeline[]>([]);
  const loading = ref(false);

  async function loadGraphs() {
    loading.value = true;
    try {
      const res = await rpc.call<{ graphs: WorkflowGraph[] }>('workflow.graphs.list');
      graphs.value = res.graphs || [];
    } finally { loading.value = false; }
  }
  async function loadRuns() {
    const res = await rpc.call<{ runs: WorkflowRun[] }>('workflow.runs.list');
    runs.value = res.runs || [];
  }
  async function loadRoles() {
    const res = await rpc.call<{ roles: AgentRole[] }>('agent_roles.list');
    roles.value = res.roles || [];
  }
  async function loadPipelines() {
    const res = await rpc.call<{ pipelines: Pipeline[] }>('pipelines.list');
    pipelines.value = res.pipelines || [];
  }

  return { graphs, runs, roles, pipelines, loading, loadGraphs, loadRuns, loadRoles, loadPipelines };
});
