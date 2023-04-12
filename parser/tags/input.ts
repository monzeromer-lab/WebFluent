import { TokenType } from "../../lexer/types.ts";
import { ASTNode, Parser } from "../parser.ts";
import { parseAttributes } from "./attributes.ts";

  /**
   * Parses a Input node (e.g. "Input(Text)").
   * Throws a ParserError if the current token is not "Component".
   * @returns An ASTNode representing the component.
   */
  export function parseInput(): ASTNode {
    Parser.expect(TokenType.Input);

    Parser.expect(TokenType.OpenParen);

    const identifierToken = Parser.currentToken;
    Parser.expect(TokenType.Identifier);
    if (Parser.currentToken?.type === TokenType.Coma) {
        Parser.expect(TokenType.Coma);
    }
    const attributes = parseAttributes();
    Parser.expect(TokenType.CloseParen);

    return {
      type: TokenType.Input,
      //@ts-ignore
      value: identifierToken.value,
      attributes,
    };
  }