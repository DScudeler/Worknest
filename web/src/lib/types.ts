// TypeScript mirrors of the Worknest backend DTOs.
// Source of truth: crates/worknest-core/src/models/ and crates/worknest-api/src/main.rs.
//
// Enum casing note: the backend serializes enum variants in PascalCase (no
// `#[serde(rename_all)]`), e.g. "Task", "Open", "InProgress", "High". The
// API also accepts lowercase on input (handlers `to_lowercase()` first), but
// outgoing JSON is PascalCase, so we use those values here.

export type UserId = string;
export type ProjectId = string;
export type TicketId = string;
export type CommentId = string;
export type AttachmentId = string;

export interface User {
  id: UserId;
  username: string;
  email: string;
  full_name: string | null;
  avatar_url: string | null;
  created_at: string;
  updated_at: string;
}

export interface PublicUser {
  id: UserId;
  username: string;
}

export interface DashboardStats {
  open_tickets: number;
  assigned_to_me: number;
  due_this_week: number;
  active_projects: number;
}

/// Display name for a user — prefers the optional full_name, falls back to
/// the unique username.
export function displayName(user: { full_name?: string | null; username: string }): string {
  return user.full_name?.trim() ? user.full_name : user.username;
}

export interface Project {
  id: ProjectId;
  name: string;
  description: string | null;
  color: string | null;
  archived: boolean;
  created_by: UserId;
  /// Optional source-repo path (local absolute or clone URL). Used by the
  /// agents subsystem's BootstrapWorktree step. Null means no worktree.
  repo_path: string | null;
  created_at: string;
  updated_at: string;
}

export type TicketType = "Task" | "Bug" | "Feature" | "Epic";
export type TicketStatus = "Open" | "InProgress" | "Review" | "Done" | "Closed";
export type Priority = "Low" | "Medium" | "High" | "Critical";

export const TICKET_TYPES: TicketType[] = ["Task", "Bug", "Feature", "Epic"];
export const TICKET_STATUSES: TicketStatus[] = ["Open", "InProgress", "Review", "Done", "Closed"];
export const PRIORITIES: Priority[] = ["Low", "Medium", "High", "Critical"];

export type TagId = string;

export interface Tag {
  id: TagId;
  name: string;
  color_bg: string;
  color_fg: string;
  created_at: string;
}

export interface Ticket {
  id: TicketId;
  project_id: ProjectId;
  title: string;
  description: string | null;
  ticket_type: TicketType;
  status: TicketStatus;
  priority: Priority;
  assignee_id: UserId | null;
  created_by: UserId;
  due_date: string | null;
  estimate_hours: number | null;
  created_at: string;
  updated_at: string;
  /// Empty array when the ticket has no tags. The backend always populates
  /// this field on every Ticket response (since V5).
  tags: Tag[];
}

export interface Comment {
  id: CommentId;
  ticket_id: TicketId;
  user_id: UserId;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface Attachment {
  id: AttachmentId;
  ticket_id: TicketId;
  filename: string;
  file_size: number;
  mime_type: string;
  file_path: string;
  uploaded_by: UserId;
  created_at: string;
}

export interface ProjectMember {
  user_id: UserId;
  role: string;
}

export interface AuthResponse {
  user: User;
  token: string;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
}

export interface CreateProjectRequest {
  name: string;
  description?: string;
  color?: string;
  repo_path?: string;
}

export interface UpdateProjectRequest {
  name?: string;
  description?: string | null;
  archived?: boolean;
  color?: string | null;
  repo_path?: string | null;
}

export interface CreateTicketRequest {
  project_id: ProjectId;
  title: string;
  description?: string;
  ticket_type: TicketType;
  priority?: Priority;
  tag_ids?: TagId[];
}

export interface UpdateTicketRequest {
  title?: string;
  description?: string | null;
  status?: TicketStatus;
  priority?: Priority;
  ticket_type?: TicketType;
  assignee_id?: UserId | "";
  tag_ids?: TagId[];
}

export interface AddMemberRequest {
  user_id: UserId;
  role?: string;
}

// CSS class hint for tag chips. Returns the design-language slug only when the
// backend tag matches a known palette; otherwise the chip falls back to the
// neutral surface-3 styling.
export type TagSlug = "bug" | "feature" | "design" | "research" | "docs" | "chore";
const KNOWN_TAG_SLUGS: TagSlug[] = ["bug", "feature", "design", "research", "docs", "chore"];

export function tagSlug(tag: { name: string }): TagSlug | null {
  const candidate = tag.name.toLowerCase();
  return KNOWN_TAG_SLUGS.includes(candidate as TagSlug) ? (candidate as TagSlug) : null;
}

// Display helpers — Review is first-class (own color); Closed reuses the
// "blocked" red treatment for color/spacing.
export type DisplayStatus = "open" | "progress" | "review" | "done" | "blocked";
export type DisplayPriority = "low" | "med" | "high" | "urgent";

export function toDisplayStatus(s: TicketStatus): DisplayStatus {
  switch (s) {
    case "Open":
      return "open";
    case "InProgress":
      return "progress";
    case "Review":
      return "review";
    case "Done":
      return "done";
    case "Closed":
      return "blocked";
  }
}

export function toDisplayPriority(p: Priority): DisplayPriority {
  switch (p) {
    case "Low":
      return "low";
    case "Medium":
      return "med";
    case "High":
      return "high";
    case "Critical":
      return "urgent";
  }
}

export function statusLabel(s: TicketStatus): string {
  return ({ Open: "Open", InProgress: "In Progress", Review: "In Review", Done: "Done", Closed: "Closed" } as const)[s];
}

export function priorityLabel(p: Priority): string {
  return ({ Low: "Low", Medium: "Medium", High: "High", Critical: "Urgent" } as const)[p];
}

export function ticketTypeLabel(t: TicketType): string {
  return t;
}

// =============================================================================
// Agents (V7)
// =============================================================================

export type PersonaId = string;
export type AgentDeploymentId = string;
export type AgentTickId = string;
export type AgentEventId = string;

export type Capability =
  | "Comment"
  | "Label"
  | "Assign"
  | "SetPriority"
  | "SetStatus"
  | "Attach"
  | "CreateTicket"
  | "Close";

export const CAPABILITIES: { id: Capability; label: string }[] = [
  { id: "Comment", label: "Comment" },
  { id: "Label", label: "Label" },
  { id: "Assign", label: "Assign" },
  { id: "SetPriority", label: "Set priority" },
  { id: "SetStatus", label: "Set status" },
  { id: "Attach", label: "Attach files" },
  { id: "CreateTicket", label: "Create ticket" },
  { id: "Close", label: "Close ticket" },
];

export type AgentModel = "Haiku" | "Sonnet" | "Opus";
export const AGENT_MODELS: { id: AgentModel; label: string; tag: "fast" | "balanced" | "reasoning" }[] = [
  // Tier names persisted in the DB are version-agnostic ("Haiku" / "Sonnet" /
  // "Opus"); the labels and the wire model id resolve to the latest minor
  // release per family at the time of this build.
  { id: "Haiku", label: "Claude Haiku 4.5", tag: "fast" },
  { id: "Sonnet", label: "Claude Sonnet 4.6", tag: "balanced" },
  { id: "Opus", label: "Claude Opus 4.7", tag: "reasoning" },
];

export type AgentStatus =
  | "Pending"
  | "Registering"
  | "Granting"
  | "Snapshotting"
  | "Provisioning"
  | "Scheduling"
  | "Running"
  | "Paused"
  | "Idle"
  | "Stopped"
  | "Error";

export const AGENT_STATUSES: AgentStatus[] = [
  "Running",
  "Paused",
  "Idle",
  "Stopped",
  "Error",
];

export type DisplayAgentStatus = "running" | "paused" | "idle" | "stopped" | "error" | "activating";

/// Map the wire status to the lowercase tag the design CSS expects.
/// Activation states (`Pending`/`Registering`/.../`Scheduling`) all collapse
/// to "activating" — the UI shows a single neutral pill while the backend
/// drives the deployment forward.
export function toDisplayAgentStatus(s: AgentStatus): DisplayAgentStatus {
  switch (s) {
    case "Running":
      return "running";
    case "Paused":
      return "paused";
    case "Idle":
      return "idle";
    case "Stopped":
      return "stopped";
    case "Error":
      return "error";
    default:
      return "activating";
  }
}

export function agentStatusLabel(s: AgentStatus): string {
  switch (s) {
    case "Pending":
      return "Pending";
    case "Registering":
      return "Registering";
    case "Granting":
      return "Granting access";
    case "Snapshotting":
      return "Snapshotting";
    case "Provisioning":
      return "Provisioning";
    case "Scheduling":
      return "Scheduling";
    case "Running":
      return "Running";
    case "Paused":
      return "Suspended";
    case "Idle":
      return "Idle";
    case "Stopped":
      return "Stopped";
    case "Error":
      return "Error";
  }
}

export interface Persona {
  id: PersonaId;
  slug: string;
  name: string;
  emoji: string;
  color: string;
  description: string;
  role: string;
  tone: string;
  expertise: string[];
  instructions: string;
  capabilities: Capability[];
  model: AgentModel;
  default_cron: string;
  created_at: string;
  updated_at: string;
}

export interface AgentDeployment {
  id: AgentDeploymentId;
  project_id: ProjectId;
  persona_id: PersonaId;
  agent_user_id: UserId | null;
  snapshot_name: string | null;
  snapshot_role: string | null;
  snapshot_tone: string | null;
  snapshot_expertise: string[];
  snapshot_instructions: string | null;
  snapshot_capabilities: Capability[];
  snapshot_model: AgentModel | null;
  snapshot_taken_at: string | null;
  workspace_path: string | null;
  cron_expression: string;
  next_tick_at: string | null;
  tick_locked_at: string | null;
  tick_lock_token: string | null;
  status: AgentStatus;
  last_error_step: string | null;
  error_message: string | null;
  error_count: number;
  current_ticket_id: TicketId | null;
  runs_today: number;
  touched_this_week: number;
  success_rate: number;
  last_activity_at: string | null;
  created_at: string;
  updated_at: string;
}

/// Wire response shape for deployment endpoints — flattens the deployment
/// row and embeds the persona alongside it.
export interface AgentDeploymentResponse extends AgentDeployment {
  persona: Persona;
}

export type TickOutcome = "Success" | "Failure" | "Skipped";

export interface AgentTick {
  id: AgentTickId;
  deployment_id: AgentDeploymentId;
  started_at: string;
  finished_at: string | null;
  outcome: TickOutcome | null;
  touched_ticket_id: TicketId | null;
  action_summary: string | null;
  error_message: string | null;
}

export type AgentEventKind =
  | "DeploymentCreated"
  | "IdentityRegistered"
  | "MembershipGranted"
  | "PersonaSnapshotted"
  | "WorkspaceProvisioned"
  | "TickScheduled"
  | "MarkedRunning"
  | "ActivationFailed"
  | "Suspended"
  | "Resumed"
  | "Retried"
  | "Stopped"
  | "TickFailedThreshold";

export interface AgentEvent {
  id: AgentEventId;
  deployment_id: AgentDeploymentId;
  kind: AgentEventKind;
  payload: unknown;
  message: string;
  at: string;
}

export interface CreatePersonaRequest {
  slug: string;
  name: string;
  emoji: string;
  color: string;
  description: string;
  role: string;
  tone: string;
  expertise: string[];
  instructions: string;
  capabilities: Capability[];
  model: AgentModel;
  default_cron: string;
}

export interface UpdatePersonaRequest {
  name?: string;
  emoji?: string;
  color?: string;
  description?: string;
  role?: string;
  tone?: string;
  expertise?: string[];
  instructions?: string;
  capabilities?: Capability[];
  model?: AgentModel;
  default_cron?: string;
}

export interface CreateAgentDeploymentRequest {
  persona_id: PersonaId;
  cron_expression?: string;
}
