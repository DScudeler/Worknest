import * as vscode from 'vscode';
import { WorknestApiClient } from './api/client';
import { SecureStorage } from './utils/storage';
import { Config } from './utils/config';
import { TicketTreeProvider } from './views/ticketTree';
import { StatusBarManager } from './views/statusBar';
import { TicketCommands } from './commands/tickets';
import { GitCommands } from './commands/git';

let client: WorknestApiClient;
let storage: SecureStorage;
let treeProvider: TicketTreeProvider;
let statusBar: StatusBarManager;
let ticketCommands: TicketCommands;
let gitCommands: GitCommands;

export async function activate(context: vscode.ExtensionContext) {
    console.log('Worknest extension is now active');

    // Initialize storage and API client
    storage = new SecureStorage(context.secrets);
    const apiUrl = Config.getApiUrl();
    client = new WorknestApiClient(apiUrl);

    // Restore token if exists
    const token = await storage.getToken();
    if (token) {
        client.setToken(token);
        await vscode.commands.executeCommand('setContext', 'worknest.authenticated', true);
    }

    // Initialize views
    treeProvider = new TicketTreeProvider(client);
    statusBar = new StatusBarManager(client);

    // Initialize commands
    ticketCommands = new TicketCommands(client, treeProvider, statusBar);
    gitCommands = new GitCommands(client);

    // Register tree view
    const treeView = vscode.window.createTreeView('worknestTickets', {
        treeDataProvider: treeProvider
    });
    context.subscriptions.push(treeView);

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('worknest.login', async () => {
            await login();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('worknest.logout', async () => {
            await logout();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('worknest.createTicket', async () => {
            await ticketCommands.createTicket();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('worknest.searchTickets', async () => {
            await ticketCommands.searchTickets();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('worknest.refresh', async () => {
            treeProvider.refresh();
            await statusBar.update();
            vscode.window.showInformationMessage('Worknest data refreshed');
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('worknest.openTicket', async (ticket) => {
            if (ticket) {
                await ticketCommands.openTicket(ticket);
            } else {
                const currentTicket = statusBar.getCurrentTicket();
                if (currentTicket) {
                    await ticketCommands.openTicket(currentTicket);
                } else {
                    vscode.window.showInformationMessage('No ticket selected');
                }
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('worknest.assignToMe', async (ticket) => {
            if (ticket) {
                await ticketCommands.assignToMe(ticket);
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('worknest.changeStatus', async (ticket) => {
            if (ticket) {
                await ticketCommands.changeStatus(ticket);
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('worknest.createBranchFromTicket', async (ticket) => {
            if (ticket) {
                await gitCommands.createBranchFromTicket(ticket);
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('worknest.insertTicketReference', async (ticket) => {
            if (ticket) {
                await gitCommands.insertTicketReference(ticket);
            }
        })
    );

    // Setup git hooks
    await gitCommands.setupGitHooks();

    // Auto-refresh if enabled
    if (Config.getAutoRefresh()) {
        const interval = Config.getRefreshInterval() * 1000;
        const refreshTimer = setInterval(async () => {
            if (client.isAuthenticated()) {
                await treeProvider.loadData();
                await statusBar.update();
            }
        }, interval);

        context.subscriptions.push({
            dispose: () => clearInterval(refreshTimer)
        });
    }

    // Initial update
    if (client.isAuthenticated()) {
        await treeProvider.loadData();
        await statusBar.update();
    }

    // Listen for configuration changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(async (e) => {
            if (e.affectsConfiguration('worknest.apiUrl')) {
                const newUrl = Config.getApiUrl();
                client.setBaseUrl(newUrl);
                vscode.window.showInformationMessage('Worknest API URL updated');
            }
        })
    );
}

async function login(): Promise<void> {
    try {
        // Check connection first
        const isHealthy = await client.healthCheck();
        if (!isHealthy) {
            const changeUrl = await vscode.window.showErrorMessage(
                'Cannot connect to Worknest server. Check your API URL in settings.',
                'Change URL'
            );

            if (changeUrl) {
                const newUrl = await vscode.window.showInputBox({
                    prompt: 'Enter Worknest API URL',
                    value: Config.getApiUrl(),
                    validateInput: (value) => {
                        try {
                            new URL(value);
                            return null;
                        } catch {
                            return 'Invalid URL';
                        }
                    }
                });

                if (newUrl) {
                    await Config.setApiUrl(newUrl);
                    client.setBaseUrl(newUrl);
                }
            }
            return;
        }

        // Get credentials
        const username = await vscode.window.showInputBox({
            prompt: 'Enter your Worknest username',
            validateInput: (value) => value.trim().length > 0 ? null : 'Username cannot be empty'
        });

        if (!username) return;

        const password = await vscode.window.showInputBox({
            prompt: 'Enter your Worknest password',
            password: true,
            validateInput: (value) => value.length > 0 ? null : 'Password cannot be empty'
        });

        if (!password) return;

        // Login
        const response = await client.login(username, password);

        // Store credentials
        await storage.storeToken(response.token);
        await storage.storeUser({
            id: response.user.id,
            username: response.user.username,
            email: response.user.email
        });

        await vscode.commands.executeCommand('setContext', 'worknest.authenticated', true);

        vscode.window.showInformationMessage(`Welcome, ${response.user.username}!`);

        // Refresh data
        await treeProvider.loadData();
        await statusBar.update();

    } catch (error) {
        vscode.window.showErrorMessage(`Login failed: ${WorknestApiClient.getErrorMessage(error)}`);
    }
}

async function logout(): Promise<void> {
    client.clearToken();
    await storage.clear();
    await vscode.commands.executeCommand('setContext', 'worknest.authenticated', false);

    treeProvider.refresh();
    statusBar.dispose();

    vscode.window.showInformationMessage('Logged out from Worknest');
}

export function deactivate() {
    if (statusBar) {
        statusBar.dispose();
    }
}
