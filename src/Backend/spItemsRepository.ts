import {
  Memento,
  Disposable,
  TextDocument,
  Position,
  CompletionList,
  FileCreateEvent,
  TextDocumentChangeEvent,
} from "vscode";
import { URI } from "vscode-uri";

import { SPItem } from "./Items/spItems";
import { events } from "../Misc/sourceEvents";
import { FileItem } from "./spFilesRepository";
import {
  handleAddedDocument,
  documentChangeCallback,
  isSPFile,
  newDocumentCallback,
} from "./spFileHandlers";
import { getAllItems, getItemFromPosition } from "./spItemsGetters";
import { refreshDiagnostics } from "../Providers/spLinter";
import { refreshCfgDiagnostics } from "../Providers/cfgLinter";
import { updateDecorations } from "../Providers/decorationsProvider";
import { performance } from "perf_hooks";

export class ItemsRepository implements Disposable {
  public fileItems: Map<string, FileItem>;
  public documents: Map<string, boolean>;
  private globalState: Memento;

  constructor(globalState: Memento) {
    this.fileItems = new Map<string, FileItem>();
    this.documents = new Map<string, boolean>();
    this.globalState = globalState;
  }

  public dispose() {}

  public handleAddedDocument(event: FileCreateEvent) {
    handleAddedDocument(this, event);
  }

  public handleDocumentChange(event: TextDocumentChangeEvent) {
    if (
      !isSPFile(event.document.uri.fsPath) ||
      event.contentChanges.length === 0
    ) {
      return;
    }
    refreshDiagnostics(event.document);
    refreshCfgDiagnostics(event.document);
    documentChangeCallback(this, event);
    updateDecorations(this);
  }

  public handleNewDocument(document: TextDocument) {
    if (!isSPFile(document.uri.fsPath)) {
      return;
    }
    refreshDiagnostics(document);
    refreshCfgDiagnostics(document);
    newDocumentCallback(this, document.uri);
  }

  public handleDocumentOpening(filePath: string) {
    newDocumentCallback(this, URI.file(filePath));
  }

  public getEventCompletions(): CompletionList {
    return new CompletionList(events);
  }

  public getAllItems(uri: URI): SPItem[] {
    return getAllItems(this, uri);
  }

  public getItemFromPosition(
    document: TextDocument,
    position: Position
  ): SPItem[] {
    return getItemFromPosition(this, document, position);
  }
}
