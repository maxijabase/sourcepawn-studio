﻿import { Parser } from "./spParser";
import { TypeDefItem } from "../Providers/spItems";
import { Range } from "vscode";

export function readTypeDef(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
): void {
  let name = match[1];
  let type = match[2];
  let range = parser.makeDefinitionRange(name, line);
  let { description, params } = parser.parse_doc_comment();
  let fullRange = new Range(parser.lineNb, 0, parser.lineNb, line.length);
  parser.completions.add(
    name,
    new TypeDefItem(
      name,
      match[0],
      parser.file,
      description,
      type,
      range,
      fullRange
    )
  );
}
