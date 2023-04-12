// deno-lint-ignore-file ban-ts-comment
import { TokenType, Token } from "../lexer/types.ts";

/**
 * The nodes in the abstract syntax tree (AST) produced by the parser.
 */
interface ASTNode {
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
class ParserError extends Error {
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

  private parseChildrens(): ASTNode[] {
    const children: ASTNode[] = [];
    while (this.currentToken && this.currentToken.value !== "}") {
      //@ts-ignore
      switch (this.currentToken.type) {
        case TokenType.Component:
          children.push(this.parseComponent());
          break;

        case TokenType.Column:
          children.push(this.parseColumn());
          break;

        case TokenType.Row:
          children.push(this.parseRow());
          break;

        case TokenType.Input:
          children.push(this.parseInput());
          break;

        case TokenType.Text:
          children.push(this.parseText());
          break;

        case TokenType.Image:
          children.push(this.parseImage());
          break;

        case TokenType.Identifier:
          throw new Error("Not Supported Yet");

        default:
          //@ts-ignore
          throw new ParserError(
            this.currentToken,
            TokenType.Page || TokenType.CloseBrace
          );
      }
    }
    return children;
  }

  /**
   * Parses a single attribute (e.g. "class: 'my-class'").
   * Throws a ParserError if the current token is not an identifier followed by a colon and a string literal or a hex color code.
   * @returns The name and value of the attribute.
   */
  private parseAttribute(): { name: string; value: string } {
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

    this.expect(TokenType.OpenParen);
    const attributes = this.parseAttributes();
    this.expect(TokenType.CloseParen);

    this.expect(TokenType.OpenBrace);

    const children: ASTNode[] = this.parseChildrens();

    this.expect(TokenType.CloseBrace);

    return {
      type: TokenType.Component,
      //@ts-ignore
      value: identifierToken.value,
      children,
      attributes,
    };
  }

  /**
   * Parses a Input node (e.g. "Input(Text)").
   * Throws a ParserError if the current token is not "Component".
   * @returns An ASTNode representing the component.
   */
  private parseInput(): ASTNode {
    this.expect(TokenType.Input);

    this.expect(TokenType.OpenParen);

    const identifierToken = this.currentToken;
    this.expect(TokenType.Identifier);
    if (this.currentToken?.type === TokenType.Coma) {
      this.expect(TokenType.Coma);
    }
    const attributes = this.parseAttributes();
    this.expect(TokenType.CloseParen);

    return {
      type: TokenType.Input,
      //@ts-ignore
      value: identifierToken.value,
      attributes,
    };
  }

  private parseStyle() {
    let currentStyleInProgress: boolean = true;
    this.expect(TokenType.Style);
    this.expect(TokenType.OpenParen);
    const identifier = this.currentToken?.value;
    console.log(identifier);

    this.expect(TokenType.Identifier);
    this.expect(TokenType.CloseParen);
    this.expect(TokenType.Dot);

    const checkIfMore = () =>{
      // @ts-ignore
      if (this.currentToken.type === TokenType.Dot) {
        this.expect(TokenType.Dot);
      } else {
        currentStyleInProgress = false;
      }
    }
    while (currentStyleInProgress) {
      switch (this.currentToken?.type) {
        case TokenType.Background:
          this.expect(TokenType.Background);
          this.expect(TokenType.OpenParen);
          // this.expect(<Something about the background>);
          this.expect(TokenType.CloseParen);
          
          checkIfMore();
          break;

        case TokenType.Border:
          this.expect(TokenType.Border);
          this.expect(TokenType.OpenParen);
          // this.expect(<Something about the background>);
          this.expect(TokenType.CloseParen);

          checkIfMore();
          break;

        case TokenType.Display:
          this.expect(TokenType.Display);
          this.expect(TokenType.OpenParen)
          this.expect(TokenType.Identifier);
          this.expect(TokenType.CloseParen);

          checkIfMore();
          break;

        default:
          currentStyleInProgress = false;
          break;
      }
    }
  }

  /**
   * Parses a Input node (e.g. "Input(Text)").
   * Throws a ParserError if the current token is not "Component".
   * @returns An ASTNode representing the component.
   */
  private parseText(): ASTNode {
    this.expect(TokenType.Text);

    this.expect(TokenType.OpenParen);
    const attributes = this.parseAttributes();
    if (this.currentToken?.type === TokenType.Coma) {
      this.advance();
      this.parseStyle();
    }
    this.expect(TokenType.CloseParen);

    return {
      type: TokenType.Text,
      attributes,
    };
  }

  /**
   * Parses a Input node (e.g. "Input(Text)").
   * Throws a ParserError if the current token is not "Component".
   * @returns An ASTNode representing the component.
   */
  private parseImage(): ASTNode {
    this.expect(TokenType.Image);

    this.expect(TokenType.OpenParen);
    this.expect(TokenType.src);
    this.expect(TokenType.Colon);
    //@ts-ignore
    const source = this.currentToken?.value;
    this.expect(TokenType.String);
    if (this.currentToken?.type === TokenType.Coma) {
      this.expect(TokenType.Coma);
    }
    const attributes = this.parseAttributes();
    this.expect(TokenType.CloseParen);

    return {
      type: TokenType.Image,
      value: source,
      attributes,
    };
  }

  /**
   * Parses a page node (e.g. "Page MyComponent { ... }").
   * Throws a ParserError if the current token is not "Page".
   * @returns An ASTNode representing the page.
   */
  private parsePage(): ASTNode {
    this.expect(TokenType.Page);

    const identifierToken = this.currentToken;

    this.expect(TokenType.Identifier);
    this.expect(TokenType.OpenParen);
    const attributes = this.parseAttributes();
    this.expect(TokenType.CloseParen);
    this.expect(TokenType.OpenBrace);

    const children: ASTNode[] = this.parseChildrens();

    this.expect(TokenType.CloseBrace);

    return {
      type: TokenType.Page,
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

    this.expect(TokenType.OpenParen);
    const attributes = this.parseAttributes();
    this.expect(TokenType.CloseParen);

    this.expect(TokenType.OpenBrace);

    const children: ASTNode[] = this.parseChildrens();
    this.expect(TokenType.CloseBrace);

    return {
      type: TokenType.Column,
      children,
      attributes,
    };
  }

  /**

Parses a row node (e.g. "Column { ... }").
Throws a ParserError if the current token is not "Row".
@returns An ASTNode representing the row.
*/
  private parseRow(): ASTNode {
    this.expect(TokenType.Row);

    this.expect(TokenType.OpenParen);
    const attributes = this.parseAttributes();
    this.expect(TokenType.CloseParen);

    this.expect(TokenType.OpenBrace);

    const children: ASTNode[] = this.parseChildrens();
    this.expect(TokenType.CloseBrace);

    return {
      type: TokenType.Row,
      children,
      attributes,
    };
  }

  /**
  
  Parses the end of file (EOF) token.
  Throws a ParserError if the current token is not the end of the file.
  */
  private parseEOF(): ASTNode {
    this.advance();

    return {
      type: TokenType.EOF,
      attributes: {},
      children: [],
    };
  }
  /**
  
  Parses the set of tokens produced by the lexer and returns an array of ASTNodes.
  @returns An array of ASTNodes representing the parsed markup.
  */
  public parse(): ASTNode[] {
    const nodes: ASTNode[] = [];

    while (this.currentToken) {
      switch (this.currentToken.type) {
        case TokenType.Page:
          nodes.push(this.parsePage());
          break;

        case TokenType.Component:
          nodes.push(this.parseComponent());
          break;

        case TokenType.EOF:
          nodes.push(this.parseEOF());
          break;

        default:
          throw new ParserError(this.currentToken, TokenType.Component);
      }
    }
    this.parseEOF();

    return nodes;
  }
}
