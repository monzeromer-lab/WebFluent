// deno-lint-ignore-file ban-ts-comment no-inferrable-types
import { TokenType, Token } from "../lexer/types.ts";
import { parseComponent } from "./component.ts";
import { parseEOF } from "./eof.ts";
import { IASTNode } from "./interfaces/IAstNode.ts";
import { IASTs } from "./interfaces/IAST.ts";
import { parsePage } from "./page.ts";
import { Enviroment } from "../enviroment/eval.ts"
// import { parseStyle } from "./style/style.ts";
// import { StyleAST } from "./interfaces/IStyle.ts";

/**
 * An error thrown by the parser when it encounters an unexpected token.
 */
export class ParserError {
  /** The token that caused the error. */
  public token: Token;

  /** The expected token type. */
  public expectedType: TokenType;

  public tokens: Token[];

  constructor(
    token: Token,
    expectedType: TokenType,
    tokens: Token[] | null = null
  ) {
    this.token = token;
    this.expectedType = expectedType;
    this.tokens = tokens as Token[];

    console.log(
      `Error: Unexpected type: "${token.type}" => "${token.value}" at ${
        token.line
      }:${token.column}, expected: "${expectedType}" \n   ${ParserError.genLog(
        Parser.tokens,
        //@ts-ignore
        token.line,
        token.column
      )}`
    );
    Deno.exit(1);
  }

  public static genLog(tokens: Token[], line: number, column: number) {
    let code: string = "";
    for (let index = 0; index < tokens.length; index++) {
      if (tokens[index].line === line) {
        if (tokens[index].line === line && tokens[index].column === column) {
          code += "=>" + tokens[index].value;
          return `"${code}"`;
        }
        code += tokens[index].value + "";
      }
    }
    return `"${code}"`;
  }
}

/**
 * A parser for the custom markup language used by the component library.
 */
export class Parser {
  /** The tokens produced by the lexer. */
  public static tokens: Token[];
  /** The current token being parsed. */
  public static currentToken: Token | null;
  /** The index of the current token in the tokens array. */
  public static index: number;

  public static ASTs: IASTs = {};

  // public static styleAST: StyleAST[] = [];

  /**
   * Advances to the next token in the tokens array.
   */
  public static advance(): void {
    this.index++;

    if (this.index < this.tokens.length) {
      this.currentToken = this.tokens[this.index];
    } else {
      this.currentToken = null;
    }
  }

  /**
   * Expects the current token to be of the given type, and advances to the next token if it is.
   * Throws a ParserError if the current token is not of the expected type.
   * @param expectedType The expected token type.
   */
  public static expect(expectedType: TokenType): void {
    if (!this.currentToken) {
      throw new ParserError(this.tokens[this.tokens.length - 1], expectedType);
    }

    if (this.currentToken.type === expectedType) {
      this.advance();
    } else {
      throw new ParserError(this.currentToken, expectedType);
    }
  }

  /**
  Parses the set of tokens produced by the lexer and returns an array of ASTNodes.
  @returns An array of ASTNodes representing the parsed markup.
  */
  public static parse(tokens: Token[]): IASTs {
    Parser.tokens = tokens;
    Parser.index = 0;
    Parser.currentToken = Parser.tokens[Parser.index];
    const nodes: IASTNode[] = [];

    while (this.currentToken) {
      switch (this.currentToken.type) {
        case TokenType.Page:
          nodes.push(parsePage());
          break;

        // deno-lint-ignore no-case-declarations
        case TokenType.Component:
          let component = parseComponent();
          Enviroment.setIdentifier(component.value as string, component);
          nodes.push(component);
          break;

        // case TokenType.Style:
        //   parseStyle();
        //   break;

        case TokenType.EOF:
          nodes.push(parseEOF());
          break;

        default:
          throw new ParserError(this.currentToken, TokenType.Component);
      }
    }
    parseEOF();

    Parser.ASTs.MarkupASTL = nodes;
    // Parser.ASTs.StyleAST = this.styleAST;
    return Parser.ASTs;
  }
}
