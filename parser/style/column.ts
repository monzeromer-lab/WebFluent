import { TokenType } from "../../lexer/types.ts";
import { parseChildrens } from "../childrens.ts";
import { ASTNode, Parser } from "../parser.ts";
import { parseAttributes } from "../tags/attributes.ts";

  /**

Parses a column node (e.g. "Column { ... }").
Throws a ParserError if the current token is not "Column".
@returns An ASTNode representing the column.
*/
export function parseColumn(): ASTNode {
    Parser.expect(TokenType.Column);

    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    Parser.expect(TokenType.CloseParen);

    Parser.expect(TokenType.OpenBrace);

    const children: ASTNode[] = parseChildrens();
    Parser.expect(TokenType.CloseBrace);

    return {
      type: TokenType.Column,
      children,
      attributes,
    };
  }
