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
}

export interface UpdateProjectRequest {
  name?: string;
  description?: string | null;
  archived?: boolean;
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

// Display helpers — collapse 5 backend statuses to 4 design statuses (review→
// progress, closed→blocked treatment) for color/spacing.
export type DisplayStatus = "open" | "progress" | "done" | "blocked";
export type DisplayPriority = "low" | "med" | "high" | "urgent";

export function toDisplayStatus(s: TicketStatus): DisplayStatus {
  switch (s) {
    case "Open":
      return "open";
    case "InProgress":
    case "Review":
      return "progress";
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
