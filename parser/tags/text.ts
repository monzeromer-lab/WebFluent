// deno-lint-ignore-file ban-ts-comment
import { TokenType } from "../../lexer/types.ts";
import { IASTNode } from "../interfaces/IAstNode.ts";
import { Parser } from "../parser.ts";
// import { parseStyle } from "../style/style.ts";
import { parseAttributes } from "./attributes.ts";
import { TextAttribute } from "./text.attributes.ts";

/**
   * Parses a Input node (e.g. "Input(Text)").
   * Throws a ParserError if the current token is not "Component".
   * @returns An ASTNode representing the component.
   */
export function parseText(): IASTNode {
    Parser.expect(TokenType.Text);

    Parser.expect(TokenType.OpenParen);
    const value = Parser.currentToken?.value;
    Parser.expect(TokenType.String);
    Parser.expect(TokenType.Coma);
    const identifier = Parser.currentToken?.value;
    Parser.expect(TokenType.Identifier);
    if (Parser.currentToken?.type === TokenType.Coma) {
      Parser.expect(TokenType.Coma);
    }

    const attributes = parseAttributes();
    TextAttribute(attributes);
    // let ElementClass: string;
    // if (Parser.currentToken?.type === TokenType.Coma) {
    //   Parser.advance();
    //   ElementClass = parseStyle();      
    // }
    Parser.expect(TokenType.CloseParen);

    return {
      type: TokenType.Text,
      attributes,
      value: [value, identifier],
      //@ts-ignore
      // class: ElementClass
    };
  }