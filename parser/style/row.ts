import { TokenType } from "../../lexer/types.ts";
import { parseChildrens } from "../childrens.ts";
import { ASTNode, Parser } from "../parser.ts";
import { parseAttributes } from "../tags/attributes.ts";

  /**

Parses a row node (e.g. "Column { ... }").
Throws a ParserError if the current token is not "Row".
@returns An ASTNode representing the row.
*/
export function parseRow(): ASTNode {
    Parser.expect(TokenType.Row);

    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    Parser.expect(TokenType.CloseParen);

    Parser.expect(TokenType.OpenBrace);

    const children: ASTNode[] = parseChildrens();
    Parser.expect(TokenType.CloseBrace);

    return {
      type: TokenType.Row,
      children,
      attributes,
    };
  }
