import { TokenType, Token } from "./lexer/types.ts";


interface ASTNode {
  type: string;
  value?: string;
  children?: ASTNode[];
}

export class Parser {
  private tokens: Token[];
  private currentToken: Token | null;
  private index: number;

  constructor(tokens: Token[]) {
    this.tokens = tokens;
    this.index = 0;
    this.currentToken = this.tokens[this.index];
  }

  private advance(): void {
    this.index++;
    if (this.index < this.tokens.length) {
      this.currentToken = this.tokens[this.index];
    } else {
      this.currentToken = null;
    }
  }

  private expect(expectedType: TokenType): void {
    if (!this.currentToken) {
      throw new Error(`Unexpected end of input at line ${this.tokens[this.tokens.length - 1].line}, column ${this.tokens[this.tokens.length - 1].column}`);
    }
    if (this.currentToken.type === "Identifier" && expectedType === TokenType.Identifier) {
      this.advance();
    } else if (this.currentToken.type === "OpenBrace" && expectedType === TokenType.OpenBrace) {
      this.advance();
    } else if (this.currentToken.type === expectedType) {
      this.advance();
    } else {
      throw new Error(`Token: ${JSON.stringify(this.currentToken)} Expected "${expectedType}" at line ${this.currentToken.line}, column ${this.currentToken.column}`);
    }
  }

  public parseComponent(): ASTNode {
    this.expect(TokenType.Component);
    const identifierToken = this.currentToken;
    this.expect(TokenType.Identifier);
    this.expect(TokenType.OpenBrace);
    const children: ASTNode[] = [];
    while (this.currentToken && this.currentToken.value !== "}") {
      switch (this.currentToken.type) {
        case "Column":
          children.push(this.parseColumn());
          break;
        default:
          throw new Error(`Unknown [token type](poe://www.poe.com/_api/key_phrase?phrase=token%20type&prompt=Tell%20me%20more%20about%20token%20type.): ${this.currentToken.type} at line ${this.currentToken.line}, column ${this.currentToken.column}`);
      }
    }
    this.expect(TokenType.CloseBrace);
    return {
      type: "Component",
      //@ts-ignore
      value: identifierToken.value,
      children,
    };
  }

  public parseColumn(): ASTNode {
    this.expect(TokenType.Column);
    this.expect(TokenType.OpenBrace);
    this.expect(TokenType.CloseBrace);
    return {
      type: "Column",
    };
  }

  public parse(): ASTNode {
    const componentNode = this.parseComponent();
    this.expect(TokenType.EOF);
    return {
      type: "Program",
      children: [componentNode],
    };
  }
}