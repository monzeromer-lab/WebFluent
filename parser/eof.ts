import { TokenType } from "../lexer/types.ts";
import { ASTNode, Parser } from "./parser.ts";

  /**
  
  Parses the end of file (EOF) token.
  Throws a ParserError if the current token is not the end of the file.
  */
  export function parseEOF(): ASTNode {
    Parser.advance();

    return {
      type: TokenType.EOF,
      attributes: {},
      children: [],
    };
  }