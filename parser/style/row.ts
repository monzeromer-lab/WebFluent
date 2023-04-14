// deno-lint-ignore-file ban-ts-comment
import { TokenType } from "../../lexer/types.ts";
import { parseChildrens } from "../childrens.ts";
import { IASTNode } from "../interfaces/IAstNode.ts";
import { Parser } from "../parser.ts";
import { parseAttributes } from "../tags/attributes.ts";
import { parseStyle } from "./style.ts";

  /**

Parses a row node (e.g. "Column { ... }").
Throws a ParserError if the current token is not "Row".
@returns An ASTNode representing the row.
*/
export function parseRow(): IASTNode {
    Parser.expect(TokenType.Row);

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
      type: TokenType.Row,
      children,
      attributes,
      //@ts-ignore
      class: ElementClass
    };
  }
