// deno-lint-ignore-file ban-ts-comment
import { ProjectConfig } from "../enviroment/config.ts";
import { Enviroment } from "../enviroment/eval.ts";
import { TokenType } from "../lexer/types.ts";
import { log } from "../utils/logging.ts";
import { parseChildrens } from "./childrens.ts";
import { IASTNode } from "./interfaces/IAstNode.ts";
import { Parser } from "./parser.ts";
import { parseAttributes } from "./tags/attributes.ts";

/**
   * Parses a page node (e.g. "Page MyComponent { ... }").
   * Throws a ParserError if the current token is not "Page".
   * @returns An IASTNode representing the page.
   */
export function parsePage(): IASTNode {
    Parser.expect(TokenType.Page);

    const identifierToken = Parser.currentToken;

    Parser.expect(TokenType.Identifier);
    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    Parser.expect(TokenType.CloseParen);
    Parser.expect(TokenType.OpenBrace);

    const children: IASTNode[] = parseChildrens();

    Parser.expect(TokenType.CloseBrace);

    if(attributes["path"]) {
      log(`http://localhost:${ProjectConfig.port}/${attributes["path"]}`);
      
      Enviroment.setPage(`http://localhost:${ProjectConfig.port}/${attributes["path"]}`, {
        type: TokenType.Page,
        //@ts-ignore
        value: identifierToken.value,
        children,
        attributes,
      });
    }

    return {
      type: TokenType.Page,
      //@ts-ignore
      value: identifierToken.value,
      children,
      attributes,
    };
  }