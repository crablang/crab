import * as vscode from "vscode";

import { Ctx, Disposable } from "./ctx";
import { CrabLangEditor, isCrabLangEditor } from "./util";

// FIXME: consider implementing this via the Tree View API?
// https://code.visualstudio.com/api/extension-guides/tree-view
export class AstInspector implements vscode.HoverProvider, vscode.DefinitionProvider, Disposable {
    private readonly astDecorationType = vscode.window.createTextEditorDecorationType({
        borderColor: new vscode.ThemeColor("crablang_analyzer.syntaxTreeBorder"),
        borderStyle: "solid",
        borderWidth: "2px",
    });
    private crablangEditor: undefined | CrabLangEditor;

    // Lazy crablang token range -> syntax tree file range.
    private readonly crablang2Ast = new Lazy(() => {
        const astEditor = this.findAstTextEditor();
        if (!this.crablangEditor || !astEditor) return undefined;

        const buf: [vscode.Range, vscode.Range][] = [];
        for (let i = 0; i < astEditor.document.lineCount; ++i) {
            const astLine = astEditor.document.lineAt(i);

            // Heuristically look for nodes with quoted text (which are token nodes)
            const isTokenNode = astLine.text.lastIndexOf('"') >= 0;
            if (!isTokenNode) continue;

            const crablangRange = this.parseCrabLangTextRange(this.crablangEditor.document, astLine.text);
            if (!crablangRange) continue;

            buf.push([crablangRange, this.findAstNodeRange(astLine)]);
        }
        return buf;
    });

    constructor(ctx: Ctx) {
        ctx.pushExtCleanup(
            vscode.languages.registerHoverProvider({ scheme: "crablang-analyzer" }, this)
        );
        ctx.pushExtCleanup(vscode.languages.registerDefinitionProvider({ language: "crablang" }, this));
        vscode.workspace.onDidCloseTextDocument(
            this.onDidCloseTextDocument,
            this,
            ctx.subscriptions
        );
        vscode.workspace.onDidChangeTextDocument(
            this.onDidChangeTextDocument,
            this,
            ctx.subscriptions
        );
        vscode.window.onDidChangeVisibleTextEditors(
            this.onDidChangeVisibleTextEditors,
            this,
            ctx.subscriptions
        );
    }
    dispose() {
        this.setCrabLangEditor(undefined);
    }

    private onDidChangeTextDocument(event: vscode.TextDocumentChangeEvent) {
        if (
            this.crablangEditor &&
            event.document.uri.toString() === this.crablangEditor.document.uri.toString()
        ) {
            this.crablang2Ast.reset();
        }
    }

    private onDidCloseTextDocument(doc: vscode.TextDocument) {
        if (this.crablangEditor && doc.uri.toString() === this.crablangEditor.document.uri.toString()) {
            this.setCrabLangEditor(undefined);
        }
    }

    private onDidChangeVisibleTextEditors(editors: readonly vscode.TextEditor[]) {
        if (!this.findAstTextEditor()) {
            this.setCrabLangEditor(undefined);
            return;
        }
        this.setCrabLangEditor(editors.find(isCrabLangEditor));
    }

    private findAstTextEditor(): undefined | vscode.TextEditor {
        return vscode.window.visibleTextEditors.find(
            (it) => it.document.uri.scheme === "crablang-analyzer"
        );
    }

    private setCrabLangEditor(newCrabLangEditor: undefined | CrabLangEditor) {
        if (this.crablangEditor && this.crablangEditor !== newCrabLangEditor) {
            this.crablangEditor.setDecorations(this.astDecorationType, []);
            this.crablang2Ast.reset();
        }
        this.crablangEditor = newCrabLangEditor;
    }

    // additional positional params are omitted
    provideDefinition(
        doc: vscode.TextDocument,
        pos: vscode.Position
    ): vscode.ProviderResult<vscode.DefinitionLink[]> {
        if (!this.crablangEditor || doc.uri.toString() !== this.crablangEditor.document.uri.toString()) {
            return;
        }

        const astEditor = this.findAstTextEditor();
        if (!astEditor) return;

        const crablang2AstRanges = this.crablang2Ast
            .get()
            ?.find(([crablangRange, _]) => crablangRange.contains(pos));
        if (!crablang2AstRanges) return;

        const [crablangFileRange, astFileRange] = crablang2AstRanges;

        astEditor.revealRange(astFileRange);
        astEditor.selection = new vscode.Selection(astFileRange.start, astFileRange.end);

        return [
            {
                targetRange: astFileRange,
                targetUri: astEditor.document.uri,
                originSelectionRange: crablangFileRange,
                targetSelectionRange: astFileRange,
            },
        ];
    }

    // additional positional params are omitted
    provideHover(
        doc: vscode.TextDocument,
        hoverPosition: vscode.Position
    ): vscode.ProviderResult<vscode.Hover> {
        if (!this.crablangEditor) return;

        const astFileLine = doc.lineAt(hoverPosition.line);

        const crablangFileRange = this.parseCrabLangTextRange(this.crablangEditor.document, astFileLine.text);
        if (!crablangFileRange) return;

        this.crablangEditor.setDecorations(this.astDecorationType, [crablangFileRange]);
        this.crablangEditor.revealRange(crablangFileRange);

        const crablangSourceCode = this.crablangEditor.document.getText(crablangFileRange);
        const astFileRange = this.findAstNodeRange(astFileLine);

        return new vscode.Hover(["```crablang\n" + crablangSourceCode + "\n```"], astFileRange);
    }

    private findAstNodeRange(astLine: vscode.TextLine): vscode.Range {
        const lineOffset = astLine.range.start;
        const begin = lineOffset.translate(undefined, astLine.firstNonWhitespaceCharacterIndex);
        const end = lineOffset.translate(undefined, astLine.text.trimEnd().length);
        return new vscode.Range(begin, end);
    }

    private parseCrabLangTextRange(
        doc: vscode.TextDocument,
        astLine: string
    ): undefined | vscode.Range {
        const parsedRange = /(\d+)\.\.(\d+)/.exec(astLine);
        if (!parsedRange) return;

        const [begin, end] = parsedRange.slice(1).map((off) => this.positionAt(doc, +off));

        return new vscode.Range(begin, end);
    }

    // Memoize the last value, otherwise the CPU is at 100% single core
    // with quadratic lookups when we build crablang2Ast cache
    cache?: { doc: vscode.TextDocument; offset: number; line: number };

    positionAt(doc: vscode.TextDocument, targetOffset: number): vscode.Position {
        if (doc.eol === vscode.EndOfLine.LF) {
            return doc.positionAt(targetOffset);
        }

        // Dirty workaround for crlf line endings
        // We are still in this prehistoric era of carriage returns here...

        let line = 0;
        let offset = 0;

        const cache = this.cache;
        if (cache?.doc === doc && cache.offset <= targetOffset) {
            ({ line, offset } = cache);
        }

        while (true) {
            const lineLenWithLf = doc.lineAt(line).text.length + 1;
            if (offset + lineLenWithLf > targetOffset) {
                this.cache = { doc, offset, line };
                return doc.positionAt(targetOffset + line);
            }
            offset += lineLenWithLf;
            line += 1;
        }
    }
}

class Lazy<T> {
    val: undefined | T;

    constructor(private readonly compute: () => undefined | T) {}

    get() {
        return this.val ?? (this.val = this.compute());
    }

    reset() {
        this.val = undefined;
    }
}
