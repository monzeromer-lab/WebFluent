import { TokenType } from "../../lexer/types.ts";
import { ASTNode, Parser } from "../parser.ts";
import { parseStyle } from "../style/style.ts";
import { parseAttributes } from "./attributes.ts";

/**
   * Parses a Input node (e.g. "Input(Text)").
   * Throws a ParserError if the current token is not "Component".
   * @returns An ASTNode representing the component.
   */
export function parseText(): ASTNode {
    Parser.expect(TokenType.Text);

    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    if (Parser.currentToken?.type === TokenType.Coma) {
      Parser.advance();
      parseStyle();
    }
    Parser.expect(TokenType.CloseParen);

    return {
      type: TokenType.Text,
      attributes,
    };
  }