import { TokenType, Token } from "./lexer/types.ts";

/**
 * The nodes in the abstract syntax tree (AST) produced by the parser.
 */
interface ASTNode {
  /** The type of the node (e.g. "Component", "Column"). */
  type: string;
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
class ParserError extends Error {
  /** The token that caused the error. */
  public token: Token;
  /** The expected token type. */
  public expectedType: TokenType;

  constructor(token: Token, expectedType: TokenType) {
    super(
      `Unexpected token: ${JSON.stringify(token)}, expected: ${expectedType}`
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
  private tokens: Token[];
  /** The current token being parsed. */
  private currentToken: Token | null;
  /** The index of the current token in the tokens array. */
  private index: number;

  /**
   * Creates a new parser with the given tokens.
   * @param tokens The tokens produced by the lexer.
   */
  constructor(tokens: Token[]) {
    this.tokens = tokens;
    this.index = 0;
    this.currentToken = this.tokens[this.index];
  }

  /**
   * Advances to the next token in the tokens array.
   */
  private advance(): void {
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
  private expect(expectedType: TokenType): void {
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
   * Throws a ParserError if the current token is not an identifier followed by a colon and a string literal.
   * @returns The name and value of the attribute.
   */
  private parseAttribute(): { name: string; value: string } {
    this.expect(TokenType.Identifier);
    //@ts-ignore
    const name = this.currentToken.value;
    this.expect(TokenType.Colon);
    this.expect(TokenType.StringLiteral);
    //@ts-ignore
    const value = this.currentToken.value;
    return { name, value };
  }

  /**
   * Parses a set of attributes (e.g. "class: 'my-class', style: 'color: red;'").
   * @returns A record of attribute names and values.
   */
  private parseAttributes(): Record<string, string> {
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
   * Parses a component node (e.g. "Component MyComponent { ... }").
   * Throws a ParserError if the current token is not "Component".
   * @returns An ASTNode representing the component.
   */
  private parseComponent(): ASTNode {
    this.expect(TokenType.Component);
    const identifierToken = this.currentToken;
    this.expect(TokenType.Identifier);
    this.expect(TokenType.OpenBrace);
    const children: ASTNode[] = [];
    const attributes = this.parseAttributes();
    while (this.currentToken && this.currentToken.value !== "}") {
      switch (this.currentToken.type) {
        case TokenType.Component:
          children.push(this.parseComponent());
          break;
        case TokenType.Column:
          children.push(this.parseColumn());
          break;
        default:
          throw new ParserError(this.currentToken, TokenType.Component);
      }
    }
    this.expect(TokenType.CloseBrace);
    return {
      type: "Component",
      //@ts-ignore
      value: identifierToken.value,
      children,
      attributes,
    };
  }
  /**

Parses a column node (e.g. "Column { ... }").
Throws a ParserError if the current token is not "Column".
@returns An ASTNode representing the column.
*/
  private parseColumn(): ASTNode {
    this.expect(TokenType.Column);
    this.expect(TokenType.OpenBrace);
    const children: ASTNode[] = [];
    const attributes = this.parseAttributes();
    while (this.currentToken && this.currentToken.value !== "}") {
      switch (this.currentToken.type) {
        case TokenType.Component:
          children.push(this.parseComponent());
          break;
        default:
          throw new ParserError(this.currentToken, TokenType.Component);
      }
    }
    this.expect(TokenType.CloseBrace);
    return {
      type: "Column",
      children,
      attributes,
    };
  }
  /**
  
  Parses the end of file (EOF) token.
  Throws a ParserError if the current token is not the end of the file.
  */
  private parseEOF(): void {
    if (this.currentToken) {
      throw new ParserError(this.currentToken, TokenType.EOF);
    }
  }
  /**
  
  Parses the set of tokens produced by the lexer and returns an array of ASTNodes.
  @returns An array of ASTNodes representing the parsed markup.
  */
  public parse(): ASTNode[] {
    const nodes: ASTNode[] = [];
    while (this.currentToken) {
      switch (this.currentToken.type) {
        case TokenType.Component:
          nodes.push(this.parseComponent());
          break;
        case TokenType.Column:
          nodes.push(this.parseColumn());
          break;
        default:
          throw new ParserError(this.currentToken, TokenType.Component);
      }
    }
    this.parseEOF();
    return nodes;
  }
}
