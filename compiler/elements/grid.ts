import { TokenType } from "../../lexer/types.ts";
import { IASTNode } from "../../parser/interfaces/IAstNode.ts";
import { HTMLCompiler } from "../htmlElements.ts";

export function CompileGrid(
  type: TokenType,
  Class: string,
  attributes: Record<string, string> | undefined,
  childrens: IASTNode[]
): void {
  switch (type) {
    case TokenType.Row:
      // log("row");
      HTMLCompiler.output += `<div class='row ${
        Class ? Class : ""
      }' ${HTMLCompiler.renderAttributes(attributes)}>`;
      HTMLCompiler.visitNodes(childrens || []);
      HTMLCompiler.output += "</div>";
      break;

    case TokenType.Container:
      // log("container");
      HTMLCompiler.output += `<div class='container ${
        Class ? Class : ""
      }' ${HTMLCompiler.renderAttributes(attributes)}>`;
      HTMLCompiler.visitNodes(childrens || []);
      HTMLCompiler.output += "</div>";
      break;

    case TokenType.Column:
      // log("column");
      HTMLCompiler.output += `<div class='column ${
        Class ? Class : ""
      }' ${HTMLCompiler.renderAttributes(attributes)}>`;
      HTMLCompiler.visitNodes(childrens || []);
      HTMLCompiler.output += "</div>";
      break;
  }
}
