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
          throw new Error("Not Supported Yet");

        default:
          //@ts-ignore
          throw new ParserError(
            Parser.currentToken,
            TokenType.Page || TokenType.CloseBrace
          );
      }
    }
    return children;
  }
