import { TokenType } from "../lexer/types.ts";
import { parseChildrens } from "./childrens.ts";
import { ASTNode, Parser } from "./parser.ts";
import { parseAttributes } from "./tags/attributes.ts";

/**
   * Parses a page node (e.g. "Page MyComponent { ... }").
   * Throws a ParserError if the current token is not "Page".
   * @returns An ASTNode representing the page.
   */
export function parsePage(): ASTNode {
    Parser.expect(TokenType.Page);

    const identifierToken = Parser.currentToken;

    Parser.expect(TokenType.Identifier);
    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    Parser.expect(TokenType.CloseParen);
    Parser.expect(TokenType.OpenBrace);

    const children: ASTNode[] = parseChildrens();

    Parser.expect(TokenType.CloseBrace);

    return {
      type: TokenType.Page,
      //@ts-ignore
      value: identifierToken.value,
      children,
      attributes,
    };
  }