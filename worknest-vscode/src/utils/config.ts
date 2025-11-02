import * as vscode from 'vscode';

export class Config {
    private static readonly SECTION = 'worknest';

    static getApiUrl(): string {
        return vscode.workspace.getConfiguration(this.SECTION).get('apiUrl', 'http://localhost:3000');
    }

    static getAutoRefresh(): boolean {
        return vscode.workspace.getConfiguration(this.SECTION).get('autoRefresh', true);
    }

    static getRefreshInterval(): number {
        return vscode.workspace.getConfiguration(this.SECTION).get('refreshInterval', 60);
    }

    static getShowStatusBar(): boolean {
        return vscode.workspace.getConfiguration(this.SECTION).get('showStatusBar', true);
    }

    static getDefaultProject(): string {
        return vscode.workspace.getConfiguration(this.SECTION).get('defaultProject', '');
    }

    static async setApiUrl(url: string): Promise<void> {
        await vscode.workspace.getConfiguration(this.SECTION).update('apiUrl', url, vscode.ConfigurationTarget.Global);
    }

    static async setDefaultProject(projectId: string): Promise<void> {
        await vscode.workspace.getConfiguration(this.SECTION).update('defaultProject', projectId, vscode.ConfigurationTarget.Global);
    }
}
