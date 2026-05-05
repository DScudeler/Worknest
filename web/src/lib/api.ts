// Thin typed wrapper around the Worknest REST API.
//
// The Vite dev server proxies /api -> http://127.0.0.1:3000 (see
// vite.config.ts). In production the frontend is hosted separately and the
// API origin must allow it via WORKNEST_ALLOWED_ORIGINS.

import type {
  AddMemberRequest,
  Attachment,
  AuthResponse,
  Comment,
  CreateProjectRequest,
  CreateTicketRequest,
  DashboardStats,
  LoginRequest,
  Project,
  ProjectId,
  ProjectMember,
  PublicUser,
  RegisterRequest,
  Tag,
  Ticket,
  TicketId,
  UpdateProjectRequest,
  UpdateTicketRequest,
  User,
} from "./types";

const API_BASE = "/api";
const TOKEN_KEY = "worknest.auth_token";

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

export function getStoredToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}
export function setStoredToken(token: string | null): void {
  if (token) localStorage.setItem(TOKEN_KEY, token);
  else localStorage.removeItem(TOKEN_KEY);
}

let onUnauthorized: (() => void) | null = null;
export function setUnauthorizedHandler(fn: (() => void) | null): void {
  onUnauthorized = fn;
}

interface RequestOpts {
  method?: string;
  body?: unknown;
  headers?: Record<string, string>;
  signal?: AbortSignal;
}

async function request<T>(path: string, opts: RequestOpts = {}): Promise<T> {
  const headers: Record<string, string> = { ...(opts.headers ?? {}) };
  const token = getStoredToken();
  if (token) headers["Authorization"] = `Bearer ${token}`;
  let body: BodyInit | undefined;
  if (opts.body instanceof FormData) {
    body = opts.body;
  } else if (opts.body !== undefined) {
    headers["Content-Type"] = headers["Content-Type"] ?? "application/json";
    body = JSON.stringify(opts.body);
  }

  const res = await fetch(`${API_BASE}${path}`, {
    method: opts.method ?? (opts.body !== undefined ? "POST" : "GET"),
    headers,
    body,
    signal: opts.signal,
  });

  if (res.status === 401) {
    setStoredToken(null);
    onUnauthorized?.();
    throw new ApiError(401, "Authentication required");
  }
  if (res.status === 204) return undefined as T;
  const contentType = res.headers.get("content-type") ?? "";
  if (!res.ok) {
    let message = res.statusText || `Request failed (${res.status})`;
    if (contentType.includes("application/json")) {
      try {
        const payload = (await res.json()) as { error?: string };
        if (payload?.error) message = payload.error;
      } catch {
        /* ignore */
      }
    }
    throw new ApiError(res.status, message);
  }
  if (contentType.includes("application/json")) return (await res.json()) as T;
  return (await res.text()) as unknown as T;
}

// Auth ---------------------------------------------------------------------

export const authApi = {
  login: (data: LoginRequest) =>
    request<AuthResponse>("/auth/login", { method: "POST", body: data }),
  register: (data: RegisterRequest) =>
    request<AuthResponse>("/auth/register", { method: "POST", body: data }),
  logout: () => request<void>("/auth/logout", { method: "POST", body: {} }),
};

// Users --------------------------------------------------------------------

export const usersApi = {
  me: () => request<User>("/users/me"),
  list: () => request<PublicUser[]>("/users"),
  updateMe: (data: {
    username?: string;
    email?: string;
    full_name?: string;
    avatar_url?: string;
  }) => request<User>("/users/me", { method: "PUT", body: data }),
  changePassword: (data: { old_password: string; new_password: string }) =>
    request<void>("/users/me/password", { method: "POST", body: data }),
};

export const statsApi = {
  get: () => request<DashboardStats>("/stats"),
};

// Projects -----------------------------------------------------------------

export const projectsApi = {
  list: () => request<Project[]>("/projects"),
  get: (id: ProjectId) => request<Project>(`/projects/${id}`),
  create: (data: CreateProjectRequest) =>
    request<Project>("/projects", { method: "POST", body: data }),
  update: (id: ProjectId, data: UpdateProjectRequest) =>
    request<Project>(`/projects/${id}`, { method: "PUT", body: data }),
  archive: (id: ProjectId) =>
    request<Project>(`/projects/${id}/archive`, { method: "POST", body: {} }),
  remove: (id: ProjectId) => request<void>(`/projects/${id}`, { method: "DELETE" }),
  members: (id: ProjectId) => request<ProjectMember[]>(`/projects/${id}/members`),
  addMember: (id: ProjectId, data: AddMemberRequest) =>
    request<ProjectMember>(`/projects/${id}/members`, { method: "POST", body: data }),
  removeMember: (id: ProjectId, userId: string) =>
    request<void>(`/projects/${id}/members/${userId}`, { method: "DELETE" }),
};

// Tickets ------------------------------------------------------------------

export interface ListTicketsQuery {
  project_id?: ProjectId;
  status?: string;
  priority?: string;
  assignee_id?: string;
  sort?: string;
  limit?: number;
  offset?: number;
}

export const ticketsApi = {
  list: (q: ListTicketsQuery = {}) => {
    const params = new URLSearchParams();
    Object.entries(q).forEach(([k, v]) => {
      if (v !== undefined && v !== null && v !== "") params.set(k, String(v));
    });
    const qs = params.toString();
    return request<Ticket[]>(`/tickets${qs ? `?${qs}` : ""}`);
  },
  search: (query: string, projectId?: ProjectId) => {
    const params = new URLSearchParams({ q: query });
    if (projectId) params.set("project_id", projectId);
    return request<Ticket[]>(`/tickets/search?${params}`);
  },
  get: (id: TicketId) => request<Ticket>(`/tickets/${id}`),
  create: (data: CreateTicketRequest) =>
    request<Ticket>("/tickets", { method: "POST", body: data }),
  update: (id: TicketId, data: UpdateTicketRequest, ifMatch?: string) =>
    request<Ticket>(`/tickets/${id}`, {
      method: "PUT",
      body: data,
      headers: ifMatch ? { "If-Match": ifMatch } : {},
    }),
  remove: (id: TicketId) => request<void>(`/tickets/${id}`, { method: "DELETE" }),
};

// Tags ---------------------------------------------------------------------

export const tagsApi = {
  list: () => request<Tag[]>("/tags"),
};

// Comments -----------------------------------------------------------------

export const commentsApi = {
  listForTicket: (ticketId: TicketId) =>
    request<Comment[]>(`/tickets/${ticketId}/comments`),
  create: (ticketId: TicketId, content: string) =>
    request<Comment>(`/tickets/${ticketId}/comments`, {
      method: "POST",
      body: { content },
    }),
  update: (id: string, content: string) =>
    request<Comment>(`/comments/${id}`, { method: "PUT", body: { content } }),
  remove: (id: string) => request<void>(`/comments/${id}`, { method: "DELETE" }),
};

// Attachments --------------------------------------------------------------

export const attachmentsApi = {
  listForTicket: (ticketId: TicketId) =>
    request<Attachment[]>(`/tickets/${ticketId}/attachments`),
  upload: (ticketId: TicketId, file: File) => {
    const fd = new FormData();
    fd.append("file", file);
    return request<Attachment>(`/tickets/${ticketId}/attachments`, {
      method: "POST",
      body: fd,
    });
  },
  downloadUrl: (id: string) => {
    const token = getStoredToken();
    return `${API_BASE}/attachments/${id}${token ? `?token=${encodeURIComponent(token)}` : ""}`;
  },
  remove: (id: string) => request<void>(`/attachments/${id}`, { method: "DELETE" }),
};
