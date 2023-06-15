// deno-lint-ignore-file ban-ts-comment no-explicit-any
import { Token } from "../lexer/types.ts";
import { IASTNode } from "../parser/interfaces/IAstNode.ts";
import { IStyle, StyleAST } from "../parser/interfaces/IStyle.ts";
import { Parser, ParserError } from "../parser/parser.ts";

export class Enviroment {
  private static Vals = new Map<string, IASTNode>();
  private static Styles = new Map<string, IStyle>();

  private static Pages = new Map<string, IASTNode>();
  public static pages: string[] = [];
  public static components: string[] = [];

  public static setIdentifier(identifier: string, data: any) {
    this.Vals.set(identifier, data);
    return data;
  }

  public static getIdentifier(
    identifier: string,
    currentToken: Token
  ): IASTNode {
    if (this.Vals.get(identifier) === undefined) {
      console.log(
        `%cEnviroment Error:`, 'color: red;', `"${identifier}" at ${currentToken.line}:${
          currentToken.column
        }, is not defined \n   ${ParserError.genLog(
          Parser.tokens,
          //@ts-ignore
          currentToken.line,
          currentToken.column
        )}`
      );
      Deno.exit(1);
    }
    return this.Vals.get(identifier) as IASTNode;
  }

  public static getComponent(name: string): IASTNode[] {
    return this.Vals.get(name) as unknown as IASTNode[];
  }

  public static setPage(name: string, data: any) {
    this.Pages.set(name, data);
    return data;
  }

  public static getPage(name: string): IASTNode[] {
    return this.Pages.get(name) as unknown as IASTNode[];
  }

  public static setStyle(identifier: string, data: any) {
    this.Styles.set(identifier, data);
    return data;
  }

  public static getStyle(identifier: string, currentToken: Token): StyleAST {
    if (this.Styles.get(identifier) === undefined) {
      console.log(
        `%cEnviroment Error:`, 'color: red;', `Style "${identifier}" at ${currentToken.line}:${
          currentToken.column
        }, is not defined \n   ${ParserError.genLog(
          Parser.tokens,
          //@ts-ignore
          currentToken.line,
          currentToken.column
        )}`
      );
      Deno.exit(1);
    }
    return this.Styles.get(identifier) as StyleAST;
  }
}
