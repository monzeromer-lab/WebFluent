// deno-lint-ignore-file ban-ts-comment
import { TokenType, Token } from "../lexer/types.ts";
import { parseComponent } from "./component.ts";
import { parseEOF } from "./eof.ts";
import { parsePage } from "./page.ts";
import { parseColumn } from "./style/column.ts";
import { parseRow } from "./style/row.ts";
import { parseImage } from "./tags/image.ts";

import { parseInput } from './tags/input.ts'
import { parseText } from "./tags/text.ts";
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
   * Creates a new parser with the given tokens.
   * @param tokens The tokens produced by the lexer.
   */
  constructor(tokens: Token[]) {

  }

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
   * Parses a single attribute (e.g. "class: 'my-class'").
   * Throws a ParserError if the current token is not an identifier followed by a colon and a string literal or a hex color code.
   * @returns The name and value of the attribute.
   */
  public static parseAttribute(): { name: string; value: string } {
    this.expect(TokenType.Identifier);
    const name = this.tokens[this.index - 1].value;

    this.expect(TokenType.Colon);

    let value: string;

    //@ts-ignore
    if (this.currentToken.type === TokenType.StringLiteral) {
      this.expect(TokenType.StringLiteral);
      value = this.tokens[this.index - 1].value;
      this.expect(TokenType.StringLiteral);
      //@ts-ignore
    } else if (
      //@ts-ignore
      this.currentToken.type === TokenType.HexColor &&
      //@ts-ignore
      this.currentToken.value.startsWith("#")
    ) {
      //@ts-ignore
      value = this.currentToken.value;
      this.advance();
    } else if (
      //@ts-ignore
      this.currentToken.type === TokenType.String
    ) {
      //@ts-ignore
      value = this.currentToken.value;
      this.advance();
    } else {
      throw new ParserError(
        //@ts-ignore
        this.currentToken,
        TokenType.StringLiteral
      );
    }

    return { name, value };
  }

  /**
   * Parses a set of attributes (e.g. "class: 'my-class', style: 'color: red;'").
   * @returns A record of attribute names and values.
   */
  public static parseAttributes(): Record<string, string> {
    const attributes: Record<string, string> = {};

    while (
      this.currentToken &&
      this.currentToken.type === TokenType.Identifier
    ) {
      const { name, value } = this.parseAttribute();

      attributes[name] = value;
    }
    return attributes;
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
