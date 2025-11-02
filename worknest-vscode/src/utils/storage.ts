import * as vscode from 'vscode';

export class SecureStorage {
    private static readonly TOKEN_KEY = 'worknest.token';
    private static readonly USER_KEY = 'worknest.user';

    constructor(private secretStorage: vscode.SecretStorage) {}

    async storeToken(token: string): Promise<void> {
        await this.secretStorage.store(SecureStorage.TOKEN_KEY, token);
    }

    async getToken(): Promise<string | undefined> {
        return await this.secretStorage.get(SecureStorage.TOKEN_KEY);
    }

    async deleteToken(): Promise<void> {
        await this.secretStorage.delete(SecureStorage.TOKEN_KEY);
    }

    async storeUser(user: { id: string; username: string; email: string }): Promise<void> {
        await this.secretStorage.store(SecureStorage.USER_KEY, JSON.stringify(user));
    }

    async getUser(): Promise<{ id: string; username: string; email: string } | undefined> {
        const userJson = await this.secretStorage.get(SecureStorage.USER_KEY);
        if (userJson) {
            try {
                return JSON.parse(userJson);
            } catch {
                return undefined;
            }
        }
        return undefined;
    }

    async deleteUser(): Promise<void> {
        await this.secretStorage.delete(SecureStorage.USER_KEY);
    }

    async clear(): Promise<void> {
        await this.deleteToken();
        await this.deleteUser();
    }
}
