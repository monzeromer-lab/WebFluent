import { TokenType } from "../lexer/types.ts";
import { parseComponent } from "./component.ts";
import { ASTNode, Parser } from "./parser.ts";
import { parseColumn } from "./style/column.ts";
import { parseRow } from "./style/row.ts";
import { parseImage } from "./tags/image.ts";
import { parseInput } from "./tags/input.ts";
import { parseText } from "./tags/text.ts";

export function parseChildrens(): ASTNode[] {
    const children: ASTNode[] = [];
    while (Parser.currentToken && Parser.currentToken.value !== "}") {
      //@ts-ignore
      switch (Parser.currentToken.type) {
        case TokenType.Component:
          children.push(parseComponent());
          break;

        case TokenType.Column:
          children.push(parseColumn());
          break;

        case TokenType.Row:
          children.push(parseRow());
          break;

        case TokenType.Input:
          children.push(parseInput());
          break;

        case TokenType.Text:
          children.push(parseText());
          break;

        case TokenType.Image:
          children.push(parseImage());
          break;

        case TokenType.Identifier:
          console.log(`Error: "${Parser.currentToken.value}" at ${Parser.currentToken.line}:${Parser.currentToken.column} Using Identifier Not Supported Yet`);
          Deno.exit(1);
          break;

        default:
          console.log(`Error: Unexpected type: "${Parser.currentToken.type}" => ${Parser.currentToken.value} at ${Parser.currentToken.line}:${Parser.currentToken.column}, expected: "Page" or "}"`);
          Deno.exit(1);
      }
    }
    return children;
  }
