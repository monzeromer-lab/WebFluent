// deno-lint-ignore-file ban-ts-comment
import { TokenType } from "../../lexer/types.ts";
import { IASTNode } from "../interfaces/IAstNode.ts";
import { Parser } from "../parser.ts";
// import { parseStyle } from "../style/style.ts";
import { parseAttributes } from "./attributes.ts";

/**
   * Parses a Input node (e.g. "Input(Text)").
   * Throws a ParserError if the current token is not "Component".
   * @returns An ASTNode representing the component.
   */
export function parseText(): IASTNode {
    Parser.expect(TokenType.Text);

    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    let ElementClass: string;
    // if (Parser.currentToken?.type === TokenType.Coma) {
    //   Parser.advance();
    //   ElementClass = parseStyle();      
    // }
    Parser.expect(TokenType.CloseParen);

    return {
      type: TokenType.Text,
      attributes,
      //@ts-ignore
      class: ElementClass
    };
  }