import { Token, TokenType } from "./lexer.ts";

class ASTNode {
  constructor(public type: string, public children: ASTNode[]) {}
}

class Parser {
  private tokens: Token[];
  private pos: number;

  constructor(tokens: Token[]) {
    this.tokens = tokens;
    this.pos = 0;
  }

  private error(): never {
    throw new Error("Invalid syntax");
  }

  private eat(type: TokenType): Token {
    if (this.tokens[this.pos].type === type) {
      return this.tokens[this.pos++];
    } else {
      this.error();
    }
  }

  private parseComponent(): ASTNode {
    const name = this.eat(TokenType.Identifier).value;
    this.eat(TokenType.OpenBrace);
    const children = this.parseChildren();
    this.eat(TokenType.CloseBrace);
    return new ASTNode("Component", [new ASTNode(name, children)]);
  }

  private parseChildren(): ASTNode[] {
    const children: ASTNode[] = [];
    while (this.tokens[this.pos].type !== TokenType.CloseBrace) {
      const child = this.parseColumn();
      children.push(child);
    }
    return children;
  }

  private parseColumn(): ASTNode {
    const name = this.eat(TokenType.Column).value;
    this.eat(TokenType.OpenBrace);
    this.eat(TokenType.CloseBrace);
    return new ASTNode(name, []);
  }

  public parse(): ASTNode {
    const component = this.parseComponent();
    if (this.tokens[this.pos].type !== TokenType.Export) {
      this.error();
    }
    this.eat(TokenType.Export);
    this.eat(TokenType.Identifier);
    this.eat(TokenType.EOF);
    return new ASTNode("Program", [component]);
  }
}
