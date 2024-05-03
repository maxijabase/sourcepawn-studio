import { WorkspaceConfiguration, WorkspaceFolder, workspace } from 'vscode'

/**
 * Get a value from the user's extension settings.
 * @param section The section setting to access
 * @param key The setting key
 * @param workspaceFolder The workplace folder to seek the setting value
 * @param def A default value in case the value returns as undefined
 * @returns A value from the user's extension settings
 */
export function getConfig(section: Section, key?: string, workspaceFolder?: WorkspaceFolder, def?: any): any {
    let config: WorkspaceConfiguration;
    if (!key) {
        return workspace.getConfiguration(section.toString());
    }
    if (workspaceFolder) {
        config = workspace.getConfiguration(section.toString(), workspaceFolder);
    }
    else {
        config = workspace.getConfiguration(section.toString());
    }
    return config.get(key, def);
}

export enum Section {
    SourcePawn = "sourcepawn",
    LSP = "SourcePawnLanguageServer",
    Editor = "editor",
}