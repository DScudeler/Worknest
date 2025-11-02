import axios, { AxiosInstance, AxiosError } from 'axios';
import {
    AuthResponse,
    UserDto,
    ProjectDto,
    TicketDto,
    CommentDto,
    AttachmentDto,
    CreateTicketRequest,
    UpdateTicketRequest,
    TicketFilters
} from './types';

export class WorknestApiClient {
    private client: AxiosInstance;
    private token: string | null = null;

    constructor(private baseUrl: string) {
        this.client = axios.create({
            baseURL: baseUrl,
            headers: {
                'Content-Type': 'application/json'
            }
        });

        // Add request interceptor to include auth token
        this.client.interceptors.request.use(
            (config) => {
                if (this.token) {
                    config.headers.Authorization = `Bearer ${this.token}`;
                }
                return config;
            },
            (error) => Promise.reject(error)
        );
    }

    // Update base URL
    setBaseUrl(url: string): void {
        this.baseUrl = url;
        this.client.defaults.baseURL = url;
    }

    // Set auth token
    setToken(token: string): void {
        this.token = token;
    }

    // Clear auth token
    clearToken(): void {
        this.token = null;
    }

    // Get current token
    getToken(): string | null {
        return this.token;
    }

    // Check if authenticated
    isAuthenticated(): boolean {
        return this.token !== null;
    }

    // Health check
    async healthCheck(): Promise<boolean> {
        try {
            const response = await this.client.get('/health');
            return response.status === 200;
        } catch {
            return false;
        }
    }

    // Authentication
    async login(username: string, password: string): Promise<AuthResponse> {
        const response = await this.client.post<AuthResponse>('/api/auth/login', {
            username,
            password
        });
        this.token = response.data.token;
        return response.data;
    }

    async register(username: string, email: string, password: string): Promise<AuthResponse> {
        const response = await this.client.post<AuthResponse>('/api/auth/register', {
            username,
            email,
            password
        });
        this.token = response.data.token;
        return response.data;
    }

    // Users
    async getUsers(): Promise<UserDto[]> {
        const response = await this.client.get<UserDto[]>('/api/users');
        return response.data;
    }

    async getCurrentUser(): Promise<UserDto> {
        const response = await this.client.get<UserDto>('/api/users/me');
        return response.data;
    }

    // Projects
    async getProjects(): Promise<ProjectDto[]> {
        const response = await this.client.get<ProjectDto[]>('/api/projects');
        return response.data;
    }

    async getProject(id: string): Promise<ProjectDto> {
        const response = await this.client.get<ProjectDto>(`/api/projects/${id}`);
        return response.data;
    }

    async createProject(name: string, description?: string): Promise<ProjectDto> {
        const response = await this.client.post<ProjectDto>('/api/projects', {
            name,
            description
        });
        return response.data;
    }

    async updateProject(id: string, name?: string, description?: string): Promise<ProjectDto> {
        const response = await this.client.put<ProjectDto>(`/api/projects/${id}`, {
            name,
            description
        });
        return response.data;
    }

    async deleteProject(id: string): Promise<void> {
        await this.client.delete(`/api/projects/${id}`);
    }

    async archiveProject(id: string): Promise<ProjectDto> {
        const response = await this.client.post<ProjectDto>(`/api/projects/${id}/archive`);
        return response.data;
    }

    // Tickets
    async getTickets(filters?: TicketFilters): Promise<TicketDto[]> {
        const params: Record<string, string> = {};

        if (filters) {
            if (filters.project_id) params.project_id = filters.project_id;
            if (filters.status) params.status = filters.status;
            if (filters.priority) params.priority = filters.priority;
            if (filters.assignee_id) params.assignee_id = filters.assignee_id;
            if (filters.sort) params.sort = filters.sort;
            if (filters.limit) params.limit = filters.limit.toString();
            if (filters.offset) params.offset = filters.offset.toString();
        }

        const response = await this.client.get<TicketDto[]>('/api/tickets', { params });
        return response.data;
    }

    async getTicket(id: string): Promise<TicketDto> {
        const response = await this.client.get<TicketDto>(`/api/tickets/${id}`);
        return response.data;
    }

    async searchTickets(query: string, projectId?: string): Promise<TicketDto[]> {
        const params: Record<string, string> = { q: query };
        if (projectId) {
            params.project_id = projectId;
        }
        const response = await this.client.get<TicketDto[]>('/api/tickets/search', { params });
        return response.data;
    }

    async createTicket(data: CreateTicketRequest): Promise<TicketDto> {
        const response = await this.client.post<TicketDto>('/api/tickets', data);
        return response.data;
    }

    async updateTicket(id: string, data: UpdateTicketRequest): Promise<TicketDto> {
        const response = await this.client.put<TicketDto>(`/api/tickets/${id}`, data);
        return response.data;
    }

    async deleteTicket(id: string): Promise<void> {
        await this.client.delete(`/api/tickets/${id}`);
    }

    // Comments
    async getComments(ticketId: string): Promise<CommentDto[]> {
        const response = await this.client.get<CommentDto[]>(`/api/tickets/${ticketId}/comments`);
        return response.data;
    }

    async createComment(ticketId: string, content: string): Promise<CommentDto> {
        const response = await this.client.post<CommentDto>(`/api/tickets/${ticketId}/comments`, {
            content
        });
        return response.data;
    }

    async updateComment(id: string, content: string): Promise<CommentDto> {
        const response = await this.client.put<CommentDto>(`/api/comments/${id}`, {
            content
        });
        return response.data;
    }

    async deleteComment(id: string): Promise<void> {
        await this.client.delete(`/api/comments/${id}`);
    }

    // Attachments
    async getAttachments(ticketId: string): Promise<AttachmentDto[]> {
        const response = await this.client.get<AttachmentDto[]>(`/api/tickets/${ticketId}/attachments`);
        return response.data;
    }

    async deleteAttachment(id: string): Promise<void> {
        await this.client.delete(`/api/attachments/${id}`);
    }

    // Error handling helper
    static isApiError(error: unknown): error is AxiosError {
        return axios.isAxiosError(error);
    }

    static getErrorMessage(error: unknown): string {
        if (axios.isAxiosError(error)) {
            if (error.response?.data?.error) {
                return error.response.data.error;
            }
            return error.message;
        }
        if (error instanceof Error) {
            return error.message;
        }
        return 'An unknown error occurred';
    }
}
