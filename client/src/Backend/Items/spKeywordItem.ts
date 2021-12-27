import {
  CompletionItemKind,
  CompletionItem,
  SignatureInformation,
  Hover,
  DocumentSymbol,
  LocationLink,
} from "vscode";

import { SPItem } from "./spItems";

export class KeywordItem implements SPItem {
  name: string;
  kind = CompletionItemKind.Keyword;
  commitCharacters = [";", ":", "<", "("];

  constructor(name: string) {
    this.name = name;
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: "",
      commitCharacters: this.commitCharacters,
    };
  }

  toDefinitionItem(): LocationLink {
    return;
  }

  toSignature(): SignatureInformation {
    return;
  }

  toHover(): Hover {
    return;
  }

  toDocumentSymbol(): DocumentSymbol {
    return;
  }
}