import * as vscode from 'vscode';
import { WorknestApiClient } from '../api/client';
import { ProjectDto, TicketDto, TicketStatus, TicketType, Priority } from '../api/types';

export class TicketTreeProvider implements vscode.TreeDataProvider<TreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<TreeItem | undefined | null | void> = new vscode.EventEmitter<TreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<TreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private projects: ProjectDto[] = [];
    private tickets: TicketDto[] = [];

    constructor(private client: WorknestApiClient) {}

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    async loadData(): Promise<void> {
        try {
            this.projects = await this.client.getProjects();
            this.tickets = await this.client.getTickets();
            this.refresh();
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to load tickets: ${WorknestApiClient.getErrorMessage(error)}`);
        }
    }

    getTreeItem(element: TreeItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: TreeItem): Promise<TreeItem[]> {
        if (!this.client.isAuthenticated()) {
            return [];
        }

        if (!element) {
            // Root level - show projects
            await this.loadData();
            return this.projects.map(project => new ProjectTreeItem(project));
        } else if (element instanceof ProjectTreeItem) {
            // Project level - show tickets
            const projectTickets = this.tickets.filter(t => t.project_id === element.project.id);
            return projectTickets.map(ticket => new TicketTreeItem(ticket));
        }

        return [];
    }
}

class ProjectTreeItem extends vscode.TreeItem {
    constructor(public readonly project: ProjectDto) {
        super(project.name, vscode.TreeItemCollapsibleState.Collapsed);
        this.contextValue = 'project';
        this.description = project.archived ? '(Archived)' : '';
        this.tooltip = project.description || project.name;

        if (project.color) {
            this.iconPath = new vscode.ThemeIcon('folder', new vscode.ThemeColor(project.color));
        } else {
            this.iconPath = new vscode.ThemeIcon('folder');
        }
    }
}

class TicketTreeItem extends vscode.TreeItem {
    constructor(public readonly ticket: TicketDto) {
        super(ticket.title, vscode.TreeItemCollapsibleState.None);
        this.contextValue = 'ticket';
        this.description = `${ticket.ticket_type} - ${ticket.status}`;
        this.tooltip = this.buildTooltip();
        this.iconPath = this.getIcon();

        // Command to open ticket details
        this.command = {
            command: 'worknest.openTicket',
            title: 'Open Ticket',
            arguments: [this.ticket]
        };
    }

    private buildTooltip(): string {
        return `${this.ticket.title}\n\nType: ${this.ticket.ticket_type}\nStatus: ${this.ticket.status}\nPriority: ${this.ticket.priority}`;
    }

    private getIcon(): vscode.ThemeIcon {
        // Icon based on ticket type
        let icon = 'issue-opened';
        switch (this.ticket.ticket_type) {
            case 'Bug':
                icon = 'bug';
                break;
            case 'Feature':
                icon = 'lightbulb';
                break;
            case 'Epic':
                icon = 'milestone';
                break;
            case 'Task':
                icon = 'tasklist';
                break;
        }

        // Color based on priority
        let color: vscode.ThemeColor | undefined;
        switch (this.ticket.priority) {
            case 'Critical':
                color = new vscode.ThemeColor('errorForeground');
                break;
            case 'High':
                color = new vscode.ThemeColor('editorWarning.foreground');
                break;
            case 'Medium':
                color = new vscode.ThemeColor('editorInfo.foreground');
                break;
            case 'Low':
                color = new vscode.ThemeColor('editorHint.foreground');
                break;
        }

        return new vscode.ThemeIcon(icon, color);
    }
}

type TreeItem = ProjectTreeItem | TicketTreeItem;
