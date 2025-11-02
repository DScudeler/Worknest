import * as vscode from 'vscode';
import { WorknestApiClient } from '../api/client';
import { TicketDto, ProjectDto, UserDto } from '../api/types';
import { TicketTreeProvider } from '../views/ticketTree';
import { StatusBarManager } from '../views/statusBar';

export class TicketCommands {
    constructor(
        private client: WorknestApiClient,
        private treeProvider: TicketTreeProvider,
        private statusBar: StatusBarManager
    ) {}

    async createTicket(): Promise<void> {
        if (!this.client.isAuthenticated()) {
            vscode.window.showErrorMessage('Please login first');
            return;
        }

        try {
            // Get projects
            const projects = await this.client.getProjects();
            if (projects.length === 0) {
                vscode.window.showErrorMessage('No projects found. Create a project first.');
                return;
            }

            // Select project
            const projectItems = projects.map(p => ({
                label: p.name,
                description: p.description,
                project: p
            }));

            const selectedProject = await vscode.window.showQuickPick(projectItems, {
                placeHolder: 'Select a project'
            });

            if (!selectedProject) return;

            // Get title
            const title = await vscode.window.showInputBox({
                prompt: 'Enter ticket title',
                validateInput: (value) => value.trim().length > 0 ? null : 'Title cannot be empty'
            });

            if (!title) return;

            // Select type
            const typeItems = [
                { label: 'Task', value: 'task' },
                { label: 'Bug', value: 'bug' },
                { label: 'Feature', value: 'feature' },
                { label: 'Epic', value: 'epic' }
            ];

            const selectedType = await vscode.window.showQuickPick(typeItems, {
                placeHolder: 'Select ticket type'
            });

            if (!selectedType) return;

            // Select priority
            const priorityItems = [
                { label: 'Low', value: 'low' },
                { label: 'Medium', value: 'medium' },
                { label: 'High', value: 'high' },
                { label: 'Critical', value: 'critical' }
            ];

            const selectedPriority = await vscode.window.showQuickPick(priorityItems, {
                placeHolder: 'Select priority'
            });

            if (!selectedPriority) return;

            // Get description
            const description = await vscode.window.showInputBox({
                prompt: 'Enter description (optional)',
                placeHolder: 'Describe the ticket...'
            });

            // Create ticket
            const ticket = await this.client.createTicket({
                project_id: selectedProject.project.id,
                title: title,
                description: description || undefined,
                ticket_type: selectedType.value,
                priority: selectedPriority.value
            });

            vscode.window.showInformationMessage(`Ticket created: ${ticket.title}`);
            this.treeProvider.refresh();
            this.statusBar.update();

        } catch (error) {
            vscode.window.showErrorMessage(`Failed to create ticket: ${WorknestApiClient.getErrorMessage(error)}`);
        }
    }

    async searchTickets(): Promise<void> {
        if (!this.client.isAuthenticated()) {
            vscode.window.showErrorMessage('Please login first');
            return;
        }

        const query = await vscode.window.showInputBox({
            prompt: 'Enter search query',
            placeHolder: 'Search tickets...'
        });

        if (!query) return;

        try {
            const tickets = await this.client.searchTickets(query);

            if (tickets.length === 0) {
                vscode.window.showInformationMessage('No tickets found');
                return;
            }

            const ticketItems = tickets.map(t => ({
                label: t.title,
                description: `${t.ticket_type} - ${t.status}`,
                detail: t.description,
                ticket: t
            }));

            const selected = await vscode.window.showQuickPick(ticketItems, {
                placeHolder: 'Select a ticket to open'
            });

            if (selected) {
                await this.openTicket(selected.ticket);
            }

        } catch (error) {
            vscode.window.showErrorMessage(`Search failed: ${WorknestApiClient.getErrorMessage(error)}`);
        }
    }

    async openTicket(ticket: TicketDto): Promise<void> {
        const panel = vscode.window.createWebviewPanel(
            'worknestTicket',
            `Ticket: ${ticket.title}`,
            vscode.ViewColumn.One,
            {
                enableScripts: true
            }
        );

        panel.webview.html = this.getTicketHtml(ticket);

        // Handle messages from webview
        panel.webview.onDidReceiveMessage(
            async message => {
                switch (message.command) {
                    case 'updateStatus':
                        await this.updateTicketStatus(ticket.id, message.status);
                        panel.dispose();
                        break;
                    case 'addComment':
                        await this.addComment(ticket.id, message.content);
                        break;
                }
            }
        );
    }

    private getTicketHtml(ticket: TicketDto): string {
        return `<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>${ticket.title}</title>
            <style>
                body {
                    font-family: var(--vscode-font-family);
                    padding: 20px;
                    color: var(--vscode-foreground);
                    background-color: var(--vscode-editor-background);
                }
                h1 { margin-top: 0; }
                .metadata {
                    display: grid;
                    grid-template-columns: 150px 1fr;
                    gap: 10px;
                    margin: 20px 0;
                    font-size: 14px;
                }
                .label { font-weight: bold; }
                .description {
                    margin-top: 20px;
                    padding: 15px;
                    background-color: var(--vscode-textBlockQuote-background);
                    border-left: 4px solid var(--vscode-textBlockQuote-border);
                }
                .actions {
                    margin-top: 20px;
                }
                button {
                    padding: 8px 16px;
                    margin-right: 8px;
                    background-color: var(--vscode-button-background);
                    color: var(--vscode-button-foreground);
                    border: none;
                    cursor: pointer;
                }
                button:hover {
                    background-color: var(--vscode-button-hoverBackground);
                }
            </style>
        </head>
        <body>
            <h1>${ticket.title}</h1>

            <div class="metadata">
                <div class="label">ID:</div>
                <div>${ticket.id}</div>

                <div class="label">Type:</div>
                <div>${ticket.ticket_type}</div>

                <div class="label">Status:</div>
                <div>${ticket.status}</div>

                <div class="label">Priority:</div>
                <div>${ticket.priority}</div>

                <div class="label">Created:</div>
                <div>${new Date(ticket.created_at).toLocaleString()}</div>

                <div class="label">Updated:</div>
                <div>${new Date(ticket.updated_at).toLocaleString()}</div>
            </div>

            ${ticket.description ? `<div class="description"><h3>Description</h3><p>${ticket.description}</p></div>` : ''}

            <div class="actions">
                <button onclick="updateStatus('InProgress')">Start Work</button>
                <button onclick="updateStatus('Review')">Move to Review</button>
                <button onclick="updateStatus('Done')">Mark Done</button>
                <button onclick="updateStatus('Closed')">Close</button>
            </div>

            <script>
                const vscode = acquireVsCodeApi();

                function updateStatus(status) {
                    vscode.postMessage({
                        command: 'updateStatus',
                        status: status
                    });
                }
            </script>
        </body>
        </html>`;
    }

    async assignToMe(ticket: TicketDto): Promise<void> {
        try {
            const user = await this.client.getCurrentUser();
            await this.client.updateTicket(ticket.id, { assignee_id: user.id });
            vscode.window.showInformationMessage(`Ticket assigned to you`);
            this.treeProvider.refresh();
            this.statusBar.update();
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to assign ticket: ${WorknestApiClient.getErrorMessage(error)}`);
        }
    }

    async updateTicketStatus(ticketId: string, status: string): Promise<void> {
        try {
            await this.client.updateTicket(ticketId, { status });
            vscode.window.showInformationMessage(`Ticket status updated to ${status}`);
            this.treeProvider.refresh();
            this.statusBar.update();
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to update status: ${WorknestApiClient.getErrorMessage(error)}`);
        }
    }

    private async addComment(ticketId: string, content: string): Promise<void> {
        try {
            await this.client.createComment(ticketId, content);
            vscode.window.showInformationMessage('Comment added');
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to add comment: ${WorknestApiClient.getErrorMessage(error)}`);
        }
    }

    async changeStatus(ticket: TicketDto): Promise<void> {
        const statuses = ['Open', 'InProgress', 'Review', 'Done', 'Closed'];
        const selected = await vscode.window.showQuickPick(statuses, {
            placeHolder: `Current status: ${ticket.status}`
        });

        if (selected) {
            await this.updateTicketStatus(ticket.id, selected);
        }
    }
}
