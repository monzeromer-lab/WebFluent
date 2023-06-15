// deno-lint-ignore-file ban-ts-comment
import { TokenType } from "../../lexer/types.ts";
import { parseChildrens } from "../childrens.ts";
import { IASTNode } from "../interfaces/IAstNode.ts";
import { Parser } from "../parser.ts";
import { parseAttributes } from "../tags/attributes.ts";
// import { parseStyle } from "./style.ts";

  /**

Parses a container node (e.g. "Container { ... }").
Thcontainers a ParserError if the current token is not "container".
@returns An ASTNode representing the container.
*/
export function parseContainer(): IASTNode {
    Parser.expect(TokenType.Container);

    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();

    // let ElementClass: string;
    // if (Parser.currentToken?.type === TokenType.Coma) {
    //   Parser.advance();
    //   ElementClass = parseStyle();      
    // }
    
    Parser.expect(TokenType.CloseParen);

    Parser.expect(TokenType.OpenBrace);

    const children: IASTNode[] = parseChildrens();
    Parser.expect(TokenType.CloseBrace);

    return {
      type: TokenType.Container,
      children,
      attributes,
      //@ts-ignore
    //   class: ElementClass
    };
  }