import * as vscode from "vscode";
import * as lc from "vscode-languageclient/node";
import * as ra from "./lsp_ext";

import { Config, prepareVSCodeConfig } from "./config";
import { createClient } from "./client";
import {
    executeDiscoverProject,
    isCrabLangDocument,
    isCrabLangEditor,
    LazyOutputChannel,
    log,
    CrabLangEditor,
} from "./util";
import { ServerStatusParams } from "./lsp_ext";
import { PersistentState } from "./persistent_state";
import { bootstrap } from "./bootstrap";
import { ExecOptions } from "child_process";

// We only support local folders, not eg. Live Share (`vlsl:` scheme), so don't activate if
// only those are in use. We use "Empty" to represent these scenarios
// (r-a still somewhat works with Live Share, because commands are tunneled to the host)

export type Workspace =
    | { kind: "Empty" }
    | {
          kind: "Workspace Folder";
      }
    | {
          kind: "Detached Files";
          files: vscode.TextDocument[];
      };

export function fetchWorkspace(): Workspace {
    const folders = (vscode.workspace.workspaceFolders || []).filter(
        (folder) => folder.uri.scheme === "file"
    );
    const crablangDocuments = vscode.workspace.textDocuments.filter((document) =>
        isCrabLangDocument(document)
    );

    return folders.length === 0
        ? crablangDocuments.length === 0
            ? { kind: "Empty" }
            : {
                  kind: "Detached Files",
                  files: crablangDocuments,
              }
        : { kind: "Workspace Folder" };
}

export async function discoverWorkspace(
    files: readonly vscode.TextDocument[],
    command: string[],
    options: ExecOptions
): Promise<JsonProject> {
    const paths = files.map((f) => `"${f.uri.fsPath}"`).join(" ");
    const joinedCommand = command.join(" ");
    const data = await executeDiscoverProject(`${joinedCommand} ${paths}`, options);
    return JSON.parse(data) as JsonProject;
}

export type CommandFactory = {
    enabled: (ctx: CtxInit) => Cmd;
    disabled?: (ctx: Ctx) => Cmd;
};

export type CtxInit = Ctx & {
    readonly client: lc.LanguageClient;
};

export class Ctx {
    readonly statusBar: vscode.StatusBarItem;
    config: Config;
    readonly workspace: Workspace;

    private _client: lc.LanguageClient | undefined;
    private _serverPath: string | undefined;
    private traceOutputChannel: vscode.OutputChannel | undefined;
    private outputChannel: vscode.OutputChannel | undefined;
    private clientSubscriptions: Disposable[];
    private state: PersistentState;
    private commandFactories: Record<string, CommandFactory>;
    private commandDisposables: Disposable[];

    get client() {
        return this._client;
    }

    constructor(
        readonly extCtx: vscode.ExtensionContext,
        commandFactories: Record<string, CommandFactory>,
        workspace: Workspace
    ) {
        extCtx.subscriptions.push(this);
        this.statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left);
        this.statusBar.show();
        this.workspace = workspace;
        this.clientSubscriptions = [];
        this.commandDisposables = [];
        this.commandFactories = commandFactories;

        this.state = new PersistentState(extCtx.globalState);
        this.config = new Config(extCtx);

        this.updateCommands("disable");
        this.setServerStatus({
            health: "stopped",
        });
    }

    dispose() {
        this.config.dispose();
        this.statusBar.dispose();
        void this.disposeClient();
        this.commandDisposables.forEach((disposable) => disposable.dispose());
    }

    async onWorkspaceFolderChanges() {
        const workspace = fetchWorkspace();
        if (workspace.kind === "Detached Files" && this.workspace.kind === "Detached Files") {
            if (workspace.files !== this.workspace.files) {
                if (this.client?.isRunning()) {
                    // Ideally we wouldn't need to tear down the server here, but currently detached files
                    // are only specified at server start
                    await this.stopAndDispose();
                    await this.start();
                }
                return;
            }
        }
        if (workspace.kind === "Workspace Folder" && this.workspace.kind === "Workspace Folder") {
            return;
        }
        if (workspace.kind === "Empty") {
            await this.stopAndDispose();
            return;
        }
        if (this.client?.isRunning()) {
            await this.restart();
        }
    }

    private async getOrCreateClient() {
        if (this.workspace.kind === "Empty") {
            return;
        }

        if (!this.traceOutputChannel) {
            this.traceOutputChannel = new LazyOutputChannel("CrabLang Analyzer Language Server Trace");
            this.pushExtCleanup(this.traceOutputChannel);
        }
        if (!this.outputChannel) {
            this.outputChannel = vscode.window.createOutputChannel("CrabLang Analyzer Language Server");
            this.pushExtCleanup(this.outputChannel);
        }

        if (!this._client) {
            this._serverPath = await bootstrap(this.extCtx, this.config, this.state).catch(
                (err) => {
                    let message = "bootstrap error. ";

                    message +=
                        'See the logs in "OUTPUT > CrabLang Analyzer Client" (should open automatically). ';
                    message +=
                        'To enable verbose logs use { "crablang-analyzer.trace.extension": true }';

                    log.error("Bootstrap error", err);
                    throw new Error(message);
                }
            );
            const newEnv = Object.assign({}, process.env, this.config.serverExtraEnv);
            const run: lc.Executable = {
                command: this._serverPath,
                options: { env: newEnv },
            };
            const serverOptions = {
                run,
                debug: run,
            };

            let rawInitializationOptions = vscode.workspace.getConfiguration("crablang-analyzer");

            if (this.workspace.kind === "Detached Files") {
                rawInitializationOptions = {
                    detachedFiles: this.workspace.files.map((file) => file.uri.fsPath),
                    ...rawInitializationOptions,
                };
            }

            const discoverProjectCommand = this.config.discoverProjectCommand;
            if (discoverProjectCommand) {
                const workspaces: JsonProject[] = await Promise.all(
                    vscode.workspace.workspaceFolders!.map(async (folder): Promise<JsonProject> => {
                        const crablangDocuments = vscode.workspace.textDocuments.filter(isCrabLangDocument);
                        return discoverWorkspace(crablangDocuments, discoverProjectCommand, {
                            cwd: folder.uri.fsPath,
                        });
                    })
                );

                this.addToDiscoveredWorkspaces(workspaces);
            }

            const initializationOptions = prepareVSCodeConfig(
                rawInitializationOptions,
                (key, obj) => {
                    // we only want to set discovered workspaces on the right key
                    // and if a workspace has been discovered.
                    if (key === "linkedProjects" && this.config.discoveredWorkspaces.length > 0) {
                        obj["linkedProjects"] = this.config.discoveredWorkspaces;
                    }
                }
            );

            this._client = await createClient(
                this.traceOutputChannel,
                this.outputChannel,
                initializationOptions,
                serverOptions,
                this.config
            );
            this.pushClientCleanup(
                this._client.onNotification(ra.serverStatus, (params) =>
                    this.setServerStatus(params)
                )
            );
            this.pushClientCleanup(
                this._client.onNotification(ra.openServerLogs, () => {
                    this.outputChannel!.show();
                })
            );
        }
        return this._client;
    }

    async start() {
        log.info("Starting language client");
        const client = await this.getOrCreateClient();
        if (!client) {
            return;
        }
        await client.start();
        this.updateCommands();
    }

    async restart() {
        // FIXME: We should re-use the client, that is ctx.deactivate() if none of the configs have changed
        await this.stopAndDispose();
        await this.start();
    }

    async stop() {
        if (!this._client) {
            return;
        }
        log.info("Stopping language client");
        this.updateCommands("disable");
        await this._client.stop();
    }

    async stopAndDispose() {
        if (!this._client) {
            return;
        }
        log.info("Disposing language client");
        this.updateCommands("disable");
        await this.disposeClient();
    }

    private async disposeClient() {
        this.clientSubscriptions?.forEach((disposable) => disposable.dispose());
        this.clientSubscriptions = [];
        await this._client?.dispose();
        this._serverPath = undefined;
        this._client = undefined;
    }

    get activeCrabLangEditor(): CrabLangEditor | undefined {
        const editor = vscode.window.activeTextEditor;
        return editor && isCrabLangEditor(editor) ? editor : undefined;
    }

    get extensionPath(): string {
        return this.extCtx.extensionPath;
    }

    get subscriptions(): Disposable[] {
        return this.extCtx.subscriptions;
    }

    get serverPath(): string | undefined {
        return this._serverPath;
    }

    addToDiscoveredWorkspaces(workspaces: JsonProject[]) {
        for (const workspace of workspaces) {
            const index = this.config.discoveredWorkspaces.indexOf(workspace);
            if (~index) {
                this.config.discoveredWorkspaces[index] = workspace;
            } else {
                this.config.discoveredWorkspaces.push(workspace);
            }
        }
    }

    private updateCommands(forceDisable?: "disable") {
        this.commandDisposables.forEach((disposable) => disposable.dispose());
        this.commandDisposables = [];

        const clientRunning = (!forceDisable && this._client?.isRunning()) ?? false;
        const isClientRunning = function (_ctx: Ctx): _ctx is CtxInit {
            return clientRunning;
        };

        for (const [name, factory] of Object.entries(this.commandFactories)) {
            const fullName = `crablang-analyzer.${name}`;
            let callback;
            if (isClientRunning(this)) {
                // we asserted that `client` is defined
                callback = factory.enabled(this);
            } else if (factory.disabled) {
                callback = factory.disabled(this);
            } else {
                callback = () =>
                    vscode.window.showErrorMessage(
                        `command ${fullName} failed: crablang-analyzer server is not running`
                    );
            }

            this.commandDisposables.push(vscode.commands.registerCommand(fullName, callback));
        }
    }

    setServerStatus(status: ServerStatusParams | { health: "stopped" }) {
        let icon = "";
        const statusBar = this.statusBar;
        statusBar.tooltip = new vscode.MarkdownString("", true);
        statusBar.tooltip.isTcrablanged = true;
        switch (status.health) {
            case "ok":
                statusBar.tooltip.appendText(status.message ?? "Ready");
                statusBar.color = undefined;
                statusBar.backgroundColor = undefined;
                statusBar.command = "crablang-analyzer.stopServer";
                break;
            case "warning":
                if (status.message) {
                    statusBar.tooltip.appendText(status.message);
                }
                statusBar.color = new vscode.ThemeColor("statusBarItem.warningForeground");
                statusBar.backgroundColor = new vscode.ThemeColor(
                    "statusBarItem.warningBackground"
                );
                statusBar.command = "crablang-analyzer.openLogs";
                icon = "$(warning) ";
                break;
            case "error":
                if (status.message) {
                    statusBar.tooltip.appendText(status.message);
                }
                statusBar.color = new vscode.ThemeColor("statusBarItem.errorForeground");
                statusBar.backgroundColor = new vscode.ThemeColor("statusBarItem.errorBackground");
                statusBar.command = "crablang-analyzer.openLogs";
                icon = "$(error) ";
                break;
            case "stopped":
                statusBar.tooltip.appendText("Server is stopped");
                statusBar.tooltip.appendMarkdown(
                    "\n\n[Start server](command:crablang-analyzer.startServer)"
                );
                statusBar.color = undefined;
                statusBar.backgroundColor = undefined;
                statusBar.command = "crablang-analyzer.startServer";
                statusBar.text = `$(stop-circle) crablang-analyzer`;
                return;
        }
        if (statusBar.tooltip.value) {
            statusBar.tooltip.appendText("\n\n");
        }
        statusBar.tooltip.appendMarkdown(
            "\n\n[Reload Workspace](command:crablang-analyzer.reloadWorkspace)"
        );
        statusBar.tooltip.appendMarkdown("\n\n[Open logs](command:crablang-analyzer.openLogs)");
        statusBar.tooltip.appendMarkdown("\n\n[Restart server](command:crablang-analyzer.startServer)");
        statusBar.tooltip.appendMarkdown("[Stop server](command:crablang-analyzer.stopServer)");
        if (!status.quiescent) icon = "$(sync~spin) ";
        statusBar.text = `${icon}crablang-analyzer`;
    }

    pushExtCleanup(d: Disposable) {
        this.extCtx.subscriptions.push(d);
    }

    private pushClientCleanup(d: Disposable) {
        this.clientSubscriptions.push(d);
    }
}

export interface Disposable {
    dispose(): void;
}
export type Cmd = (...args: any[]) => unknown;
