import { TokenType } from "../lexer/types.ts";
import { parseChildrens } from "./childrens.ts";
import { ASTNode, Parser } from "./parser.ts";
import { parseAttributes } from "./tags/attributes.ts";

/**
   * Parses a component node (e.g. "Component MyComponent { ... }").
   * Throws a ParserError if the current token is not "Component".
   * @returns An ASTNode representing the component.
   */
export function parseComponent(): ASTNode {
    Parser.expect(TokenType.Component);

    const identifierToken = Parser.currentToken;

    Parser.expect(TokenType.Identifier);

    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    Parser.expect(TokenType.CloseParen);

    Parser.expect(TokenType.OpenBrace);

    const children: ASTNode[] = parseChildrens();

    Parser.expect(TokenType.CloseBrace);

    return {
      type: TokenType.Component,
      //@ts-ignore
      value: identifierToken.value,
      children,
      attributes,
    };
  }