import { TokenType } from "../../lexer/types.ts";
import { parseChildrens } from "../childrens.ts";
import { IASTNode } from "../interfaces/IAstNode.ts";
import { Parser } from "../parser.ts";
import { parseAttributes } from "../tags/attributes.ts";
import { parseStyle } from "./style.ts";

  /**

Parses a column node (e.g. "Column { ... }").
Throws a ParserError if the current token is not "Column".
@returns An ASTNode representing the column.
*/
export function parseColumn(): IASTNode {
    Parser.expect(TokenType.Column);

    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();

    let ElementClass: string;
    if (Parser.currentToken?.type === TokenType.Coma) {
      Parser.advance();
      ElementClass = parseStyle();      
    }


    Parser.expect(TokenType.CloseParen);

    Parser.expect(TokenType.OpenBrace);

    const children: IASTNode[] = parseChildrens();
    Parser.expect(TokenType.CloseBrace);

    return {
      type: TokenType.Column,
      children,
      attributes,
      //@ts-ignore
      class: ElementClass
    };
  }
