// Data Transfer Objects matching Worknest backend

export interface UserDto {
    id: string;
    username: string;
    email: string;
    created_at: string;
    updated_at: string;
}

export interface ProjectDto {
    id: string;
    name: string;
    description?: string;
    color?: string;
    archived: boolean;
    created_by: string;
    created_at: string;
    updated_at: string;
}

export type TicketType = 'Task' | 'Bug' | 'Feature' | 'Epic';
export type TicketStatus = 'Open' | 'InProgress' | 'Review' | 'Done' | 'Closed';
export type Priority = 'Low' | 'Medium' | 'High' | 'Critical';

export interface TicketDto {
    id: string;
    project_id: string;
    title: string;
    description?: string;
    ticket_type: TicketType;
    status: TicketStatus;
    priority: Priority;
    assignee_id?: string;
    created_by: string;
    created_at: string;
    updated_at: string;
}

export interface CommentDto {
    id: string;
    ticket_id: string;
    user_id: string;
    content: string;
    created_at: string;
    updated_at: string;
}

export interface AttachmentDto {
    id: string;
    ticket_id: string;
    filename: string;
    file_size: number;
    mime_type: string;
    uploaded_by: string;
    created_at: string;
}

export interface AuthResponse {
    user: UserDto;
    token: string;
}

export interface CreateTicketRequest {
    project_id: string;
    title: string;
    description?: string;
    ticket_type: string;
    priority?: string;
}

export interface UpdateTicketRequest {
    title?: string;
    description?: string;
    status?: string;
    priority?: string;
    assignee_id?: string;
}

export interface TicketFilters {
    project_id?: string;
    status?: string;
    priority?: string;
    assignee_id?: string;
    sort?: 'created_at' | 'updated_at' | 'priority';
    limit?: number;
    offset?: number;
}
