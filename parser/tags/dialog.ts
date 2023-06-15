import { TokenType } from "../../lexer/types.ts";
import { parseChildrens } from "../childrens.ts";
import { IASTNode } from "../interfaces/IAstNode.ts";
import { Parser } from "../parser.ts";
import { parseAttributes } from "../tags/attributes.ts";

/**
   * Parses a dialog node (e.g. "dialog MyComponent { ... }").
   * Throws a ParserError if the current token is not "dialog".
   * @returns An IASTNode representing the dialog.
   */
export function parseDialog(): IASTNode {
    Parser.expect(TokenType.Dialog);

    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    Parser.expect(TokenType.CloseParen);
    Parser.expect(TokenType.OpenBrace);

    const children: IASTNode[] = parseChildrens();

    Parser.expect(TokenType.CloseBrace);

    return {
      type: TokenType.Dialog,
      children,
      attributes,
    };
  }