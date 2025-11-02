import * as vscode from 'vscode';
import { WorknestApiClient } from '../api/client';
import { TicketDto } from '../api/types';

export class GitCommands {
    constructor(private client: WorknestApiClient) {}

    async createBranchFromTicket(ticket: TicketDto): Promise<void> {
        try {
            const gitExtension = vscode.extensions.getExtension('vscode.git')?.exports;
            if (!gitExtension) {
                vscode.window.showErrorMessage('Git extension not found');
                return;
            }

            const api = gitExtension.getAPI(1);
            if (api.repositories.length === 0) {
                vscode.window.showErrorMessage('No git repository found');
                return;
            }

            const repo = api.repositories[0];

            // Generate branch name
            const ticketIdShort = ticket.id.substring(0, 8);
            const titleSlug = ticket.title
                .toLowerCase()
                .replace(/[^a-z0-9]+/g, '-')
                .replace(/^-|-$/g, '');

            const branchPrefix = ticket.ticket_type === 'Bug' ? 'fix' : 'feature';
            const branchName = `${branchPrefix}/${ticketIdShort}-${titleSlug}`;

            // Ask for confirmation
            const confirm = await vscode.window.showInputBox({
                prompt: 'Branch name',
                value: branchName,
                validateInput: (value) => value.trim().length > 0 ? null : 'Branch name cannot be empty'
            });

            if (!confirm) return;

            // Create and checkout branch
            await repo.createBranch(confirm, true);

            vscode.window.showInformationMessage(`Created and checked out branch: ${confirm}`);

        } catch (error) {
            vscode.window.showErrorMessage(`Failed to create branch: ${error instanceof Error ? error.message : 'Unknown error'}`);
        }
    }

    async insertTicketReference(ticket: TicketDto): Promise<void> {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor');
            return;
        }

        const ticketIdShort = ticket.id.substring(0, 8);
        const reference = `// Worknest: ${ticketIdShort} - ${ticket.title}`;

        editor.edit(editBuilder => {
            editBuilder.insert(editor.selection.active, reference);
        });
    }

    async prefillCommitMessage(): Promise<void> {
        try {
            const gitExtension = vscode.extensions.getExtension('vscode.git')?.exports;
            if (!gitExtension) {
                return;
            }

            const api = gitExtension.getAPI(1);
            if (api.repositories.length === 0) {
                return;
            }

            const repo = api.repositories[0];
            const branchName = repo.state.HEAD?.name;

            if (!branchName) {
                return;
            }

            // Try to extract ticket ID from branch name
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
                            const ticket = tickets[0];
                            const ticketIdShort = ticket.id.substring(0, 8);
                            const message = `[${ticketIdShort}] ${ticket.title}: `;

                            // Set input box value
                            repo.inputBox.value = message;
                            return;
                        }
                    } catch {
                        // Continue to next pattern
                    }
                }
            }

        } catch (error) {
            // Silent failure - this is a convenience feature
            console.error('Failed to prefill commit message:', error);
        }
    }

    async setupGitHooks(): Promise<void> {
        // Listen for when SCM input box is focused
        const gitExtension = vscode.extensions.getExtension('vscode.git')?.exports;
        if (!gitExtension) {
            return;
        }

        const api = gitExtension.getAPI(1);

        // Watch for repository changes
        api.repositories.forEach((repo: any) => {
            repo.state.onDidChange(() => {
                // Prefill commit message when input box is empty and branch contains ticket reference
                if (repo.inputBox.value === '') {
                    this.prefillCommitMessage();
                }
            });
        });
    }
}
