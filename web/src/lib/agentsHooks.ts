// TanStack Query hooks for the Agents feature.
//
// Polling cadences mirror the plan: deployment lists refresh every 10s while
// a user is on the agents screen so activation transitions and tick stats
// surface without manual refresh.

import { useMutation, useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import toast from "react-hot-toast";

import { agentDeploymentsApi, personasApi, projectsApi } from "./api";
import type {
  AgentDeploymentId,
  AgentDeploymentResponse,
  AgentStatus,
  CreateAgentDeploymentRequest,
  CreatePersonaRequest,
  PersonaId,
  ProjectId,
  UpdatePersonaRequest,
} from "./types";

const personaListKey = ["personas"] as const;
const projectListKey = ["projects"] as const;
const deploymentListKey = (projectId: ProjectId) =>
  ["agent-deployments", "project", projectId] as const;
const allDeploymentsKey = ["agent-deployments", "all"] as const;
const deploymentDetailKey = (id: AgentDeploymentId) =>
  ["agent-deployment", id] as const;

export function usePersonas() {
  return useQuery({
    queryKey: personaListKey,
    queryFn: () => personasApi.list(),
  });
}

export function usePersona(id: PersonaId | null) {
  return useQuery({
    queryKey: ["persona", id],
    queryFn: () => personasApi.get(id as PersonaId),
    enabled: !!id,
  });
}

/// Fetches deployments for every project the user can see and flattens the
/// result. Polled every 10 seconds so activation status and tick stats
/// surface without manual refresh.
export function useAllAgentDeployments() {
  const projectsQ = useQuery({
    queryKey: projectListKey,
    queryFn: () => projectsApi.list(),
  });
  const projectIds = projectsQ.data?.map((p) => p.id) ?? [];
  const childQueries = useQueries({
    queries: projectIds.map((pid) => ({
      queryKey: deploymentListKey(pid),
      queryFn: () => agentDeploymentsApi.listByProject(pid),
      staleTime: 10_000,
      refetchInterval: 10_000,
    })),
  });
  const deployments: AgentDeploymentResponse[] = childQueries.flatMap(
    (q) => q.data ?? [],
  );
  const isLoading =
    projectsQ.isLoading || childQueries.some((q) => q.isLoading);
  const error =
    projectsQ.error ?? childQueries.find((q) => q.error)?.error ?? null;
  return { deployments, projects: projectsQ.data ?? [], isLoading, error };
}

export function useAgentDeployment(id: AgentDeploymentId | null) {
  return useQuery({
    queryKey: deploymentDetailKey(id ?? "" as AgentDeploymentId),
    queryFn: () => agentDeploymentsApi.get(id as AgentDeploymentId),
    enabled: !!id,
    staleTime: 5_000,
    refetchInterval: id ? 5_000 : false,
  });
}

export function useAgentDeploymentTicks(id: AgentDeploymentId | null) {
  return useQuery({
    queryKey: ["agent-deployment-ticks", id],
    queryFn: () => agentDeploymentsApi.ticks(id as AgentDeploymentId, 50),
    enabled: !!id,
    staleTime: 10_000,
  });
}

export function useAgentDeploymentEvents(id: AgentDeploymentId | null) {
  return useQuery({
    queryKey: ["agent-deployment-events", id],
    queryFn: () => agentDeploymentsApi.events(id as AgentDeploymentId, 50),
    enabled: !!id,
    staleTime: 10_000,
  });
}

// Persona mutations --------------------------------------------------------

export function useSavePersona() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (
      args:
        | { kind: "create"; data: CreatePersonaRequest }
        | { kind: "update"; id: PersonaId; data: UpdatePersonaRequest },
    ) => {
      if (args.kind === "create") return personasApi.create(args.data);
      return personasApi.update(args.id, args.data);
    },
    onSuccess: (saved) => {
      qc.invalidateQueries({ queryKey: personaListKey });
      qc.setQueryData(["persona", saved.id], saved);
      toast.success(`Agent "${saved.name}" saved`);
    },
    onError: (err: Error) => toast.error(err.message || "Could not save agent"),
  });
}

export function useDeletePersona() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: PersonaId) => personasApi.remove(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: personaListKey });
      toast.success("Agent removed");
    },
    onError: (err: Error) =>
      toast.error(err.message || "Could not remove agent"),
  });
}

// Deployment mutations -----------------------------------------------------

export function useDeployPersona() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (args: {
      projectId: ProjectId;
      data: CreateAgentDeploymentRequest;
    }) => agentDeploymentsApi.deploy(args.projectId, args.data),
    onSuccess: (_resp, vars) => {
      qc.invalidateQueries({ queryKey: deploymentListKey(vars.projectId) });
      qc.invalidateQueries({ queryKey: allDeploymentsKey });
      toast.success("Agent deployment requested");
    },
    onError: (err: Error) => toast.error(err.message || "Could not deploy agent"),
  });
}

export type DeploymentControl = "suspend" | "resume" | "retry" | "stop";

export function useUpdateDeploymentStatus() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (args: {
      deployment: AgentDeploymentResponse;
      action: DeploymentControl;
    }) => {
      const ifMatch = args.deployment.updated_at;
      switch (args.action) {
        case "suspend":
          return agentDeploymentsApi.suspend(args.deployment.id, ifMatch);
        case "resume":
          return agentDeploymentsApi.resume(args.deployment.id, ifMatch);
        case "retry":
          return agentDeploymentsApi.retry(args.deployment.id, ifMatch);
        case "stop":
          return agentDeploymentsApi.stop(args.deployment.id, ifMatch);
      }
    },
    onMutate: async (vars) => {
      // Optimistic patch: flip status immediately so the controls feel snappy.
      const optimistic = optimisticPatch(vars.deployment, vars.action);
      const projectKey = deploymentListKey(vars.deployment.project_id);
      const detailKey = deploymentDetailKey(vars.deployment.id);
      await qc.cancelQueries({ queryKey: projectKey });
      await qc.cancelQueries({ queryKey: detailKey });
      const prevList =
        qc.getQueryData<AgentDeploymentResponse[]>(projectKey) ?? null;
      const prevDetail = qc.getQueryData<AgentDeploymentResponse>(detailKey) ?? null;
      if (prevList) {
        qc.setQueryData<AgentDeploymentResponse[]>(
          projectKey,
          prevList.map((d) => (d.id === vars.deployment.id ? optimistic : d)),
        );
      }
      if (prevDetail) qc.setQueryData(detailKey, optimistic);
      return { prevList, prevDetail, projectKey, detailKey };
    },
    onError: (err: Error, _vars, ctx) => {
      if (ctx?.projectKey && ctx.prevList)
        qc.setQueryData(ctx.projectKey, ctx.prevList);
      if (ctx?.detailKey && ctx.prevDetail)
        qc.setQueryData(ctx.detailKey, ctx.prevDetail);
      toast.error(err.message || "Could not update deployment");
    },
    onSettled: (_data, _err, vars) => {
      qc.invalidateQueries({
        queryKey: deploymentListKey(vars.deployment.project_id),
      });
      qc.invalidateQueries({
        queryKey: deploymentDetailKey(vars.deployment.id),
      });
    },
  });
}

export function useDeleteDeployment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (deployment: AgentDeploymentResponse) =>
      agentDeploymentsApi.remove(deployment.id),
    onSuccess: (_data, deployment) => {
      qc.invalidateQueries({
        queryKey: deploymentListKey(deployment.project_id),
      });
      qc.invalidateQueries({ queryKey: allDeploymentsKey });
      toast.success("Deployment deleted");
    },
    onError: (err: Error) =>
      toast.error(err.message || "Could not delete deployment"),
  });
}

function optimisticPatch(
  d: AgentDeploymentResponse,
  action: DeploymentControl,
): AgentDeploymentResponse {
  const now = new Date().toISOString();
  let nextStatus: AgentStatus = d.status;
  let errorMessage = d.error_message;
  switch (action) {
    case "suspend":
      nextStatus = "Paused";
      break;
    case "resume":
      nextStatus = "Running";
      errorMessage = null;
      break;
    case "retry":
      nextStatus = "Pending";
      errorMessage = null;
      break;
    case "stop":
      nextStatus = "Stopped";
      break;
  }
  return {
    ...d,
    status: nextStatus,
    error_message: errorMessage,
    last_activity_at: now,
    updated_at: now,
  };
}
