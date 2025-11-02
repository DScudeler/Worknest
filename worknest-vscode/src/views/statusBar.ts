import * as vscode from 'vscode';
import { WorknestApiClient } from '../api/client';
import { TicketDto } from '../api/types';

export class StatusBarManager {
    private statusBarItem: vscode.StatusBarItem;
    private currentTicket: TicketDto | null = null;

    constructor(private client: WorknestApiClient) {
        this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
        this.statusBarItem.command = 'worknest.openTicket';
    }

    async update(): Promise<void> {
        if (!this.client.isAuthenticated()) {
            this.statusBarItem.hide();
            return;
        }

        try {
            // Try to detect ticket from current branch
            const ticket = await this.detectTicketFromBranch();

            if (ticket) {
                this.currentTicket = ticket;
                this.statusBarItem.text = `$(issue-opened) ${ticket.id.substring(0, 8)}: ${ticket.title}`;
                this.statusBarItem.tooltip = `${ticket.title}\nStatus: ${ticket.status}\nPriority: ${ticket.priority}`;
                this.statusBarItem.show();
            } else {
                // Show ticket count
                const tickets = await this.client.getTickets({ assignee_id: 'me' });
                this.statusBarItem.text = `$(issue-opened) ${tickets.length} assigned`;
                this.statusBarItem.tooltip = `${tickets.length} tickets assigned to you`;
                this.statusBarItem.show();
            }
        } catch (error) {
            console.error('Failed to update status bar:', error);
            this.statusBarItem.hide();
        }
    }

    private async detectTicketFromBranch(): Promise<TicketDto | null> {
        try {
            const gitExtension = vscode.extensions.getExtension('vscode.git')?.exports;
            if (!gitExtension) {
                return null;
            }

            const api = gitExtension.getAPI(1);
            if (api.repositories.length === 0) {
                return null;
            }

            const repo = api.repositories[0];
            const branchName = repo.state.HEAD?.name;

            if (!branchName) {
                return null;
            }

            // Try to extract ticket ID from branch name
            // Patterns: feature/TICKET-123, TICKET-123-description, etc.
            const patterns = [
                /([0-9a-f]{8})/i,  // UUID prefix
                /TICKET-(\d+)/i     // TICKET-123 format
            ];

            for (const pattern of patterns) {
                const match = branchName.match(pattern);
                if (match) {
                    const ticketId = match[1];
                    try {
                        // Try to find ticket by searching
                        const tickets = await this.client.searchTickets(ticketId);
                        if (tickets.length > 0) {
                            return tickets[0];
                        }
                    } catch {
                        // Continue to next pattern
                    }
                }
            }

            return null;
        } catch {
            return null;
        }
    }

    getCurrentTicket(): TicketDto | null {
        return this.currentTicket;
    }

    dispose(): void {
        this.statusBarItem.dispose();
    }
}
