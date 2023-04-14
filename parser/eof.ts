import { TokenType } from "../lexer/types.ts";
import { IASTNode } from "./interfaces/IAstNode.ts";
import { Parser } from "./parser.ts";

  /**
  
  Parses the end of file (EOF) token.
  Throws a ParserError if the current token is not the end of the file.
  */
  export function parseEOF(): IASTNode {
    Parser.advance();

    return {
      type: TokenType.EOF,
      attributes: {},
      children: [],
    };
  }