// deno-lint-ignore-file ban-ts-comment
import { TokenType, Token } from "../lexer/types.ts";
import { parseComponent } from "./component.ts";
import { parseEOF } from "./eof.ts";
import { parsePage } from "./page.ts";

/**
 * The nodes in the abstract syntax tree (AST) produced by the parser.
 */
export interface ASTNode {
  /** The type of the node (e.g. "Component", "Column"). */
  type: TokenType;
  /** The value of the node (e.g. the name of the component). */
  value?: string;
  /** The children of the node (e.g. nested components or columns). */
  children?: ASTNode[];
  /** The custom attributes of the node. */
  attributes?: Record<string, string>;
}

/**
 * An error thrown by the parser when it encounters an unexpected token.
 */
export class ParserError extends Error {
  /** The token that caused the error. */
  public token: Token;
  /** The expected token type. */
  public expectedType: TokenType;

  constructor(token: Token, expectedType: TokenType) {
    super(
      `Unexpected type: ${token.type} => ${token.value} at ${token.line}:${token.column}, expected: ${expectedType}`
    );

    this.token = token;
    this.expectedType = expectedType;
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
  public static parse(tokens: Token[]): ASTNode[] {
    Parser.tokens = tokens;
    Parser.index = 0;
    Parser.currentToken = Parser.tokens[Parser.index];
    const nodes: ASTNode[] = [];

    while (this.currentToken) {
      switch (this.currentToken.type) {
        case TokenType.Page:
          nodes.push(parsePage());
          break;

        case TokenType.Component:
          nodes.push(parseComponent());
          break;

        case TokenType.EOF:
          nodes.push(parseEOF());
          break;

        default:
          throw new ParserError(this.currentToken, TokenType.Component);
      }
    }
    parseEOF();

    return nodes;
  }
}
