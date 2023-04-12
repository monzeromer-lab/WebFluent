import { TokenType } from "../../lexer/types.ts";
import { ASTNode, Parser } from "../parser.ts";

  /**
   * Parses a Input node (e.g. "Input(Text)").
   * Throws a ParserError if the current token is not "Component".
   * @returns An ASTNode representing the component.
   */
  export function parseImage(): ASTNode {
    Parser.expect(TokenType.Image);

    Parser.expect(TokenType.OpenParen);
    Parser.expect(TokenType.src);
    Parser.expect(TokenType.Colon);
    //@ts-ignore
    const source = Parser.currentToken?.value;
    Parser.expect(TokenType.String);
    if (Parser.currentToken?.type === TokenType.Coma) {
      Parser.expect(TokenType.Coma);
    }
    const attributes = Parser.parseAttributes();
    Parser.expect(TokenType.CloseParen);

    return {
      type: TokenType.Image,
      value: source,
      attributes,
    };
  }
