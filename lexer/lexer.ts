// part of this code from: https://github.com/tylerlaceby/guide-to-interpreters-series
// -----------------------------------------------------------
// ---------------          LEXER          -------------------
// ---  Responsible for producing tokens from the source   ---
// -----------------------------------------------------------

import { Token, TokenType } from "./types.ts";

const KEYWORDS: Record<string, TokenType> = {
  Component: TokenType.Component,
  Column: TokenType.Column,
  export: TokenType.Export,
};

export class Lexer {
  private source: string;
  // current lexer position
  private pos: number;

  //this is used to idintify the position of errors excatly
  private line: number;
  private column: number;

  // identifiy the current charctar
  private currentChar: string | null;

  // store the tokens here before return them
  private tokens: Array<Token>;

  constructor(sourceCode: string) {
    this.source = sourceCode;
    this.pos = 0;
    this.line = 1;
    this.column = 1;
    this.currentChar = this.source.charAt(this.pos);
    this.tokens = [];
  }

  private advance(): void {
    this.pos++;
    if (this.pos >= this.source.length) {
      this.currentChar = null;
    } else {
      this.currentChar = this.source.charAt(this.pos);
      this.column++;
    }
  }

  private skipWhitespace(): void {
    while (this.currentChar !== null && /\s/.test(this.currentChar)) {
      if (this.currentChar === "\n") {
        this.line++;
        this.column = 1;
      }
      this.advance();
    }
  }

  private readIdentifier(): string {
    let result = "";
    while (this.currentChar !== null && /[a-zA-Z]/.test(this.currentChar) || this.isint(this.currentChar as string)) {
      result += this.currentChar;
      if(this.isint(this.currentChar as string)){
        throw new Error(`Hold on a sec take a look at ${this.tokens[this.tokens.length - 1].line}:${this.tokens[this.tokens.length - 1].column} We dont have that here`);
      }
      this.advance();
    }
    return result;
  }

  /**
   * Returns whether the character passed in alphabetic -> [a-zA-Z]
   */
  private isalpha(src: string) : boolean {
    return src.toUpperCase() != src.toLowerCase();
  }

  /**
   * Returns true if the character is whitespace like -> [\s, \t, \n]
   */
  private isskippable(str: string) {
    return str == " " || str == "\n" || str == "\t" || str == "\r" ? true : false;
  }

  /**
 Return whether the character is a valid integer -> [0-9]
 */
  private isint(str: string) {
    const c = str.charCodeAt(0);
    const bounds = ["0".charCodeAt(0), "9".charCodeAt(0)];
    return c >= bounds[0] && c <= bounds[1];
  }

  // Returns a token of a given type and value
  private token(value = "", type: TokenType): void {
    this.tokens.push({ value, type, line: this.line, column: this.column });
  }

  /**
   * Given a string representing source code: Produce tokens and handles
   * possible unidentified characters.
   *
   * - Returns a array of tokens.
   * - Does not modify the incoming string.
   */
  tokenize() {
    while (this.currentChar !== null) {
      if (this.isskippable(this.currentChar)) {
        this.skipWhitespace();
        continue;

      } else if (this.currentChar === "{") {
        this.token("{", TokenType.OpenBrace);
        this.advance();

      } else if (this.currentChar === "}") {
        this.token("}", TokenType.CloseBrace);
        this.advance();

      } else if (this.isalpha(this.currentChar)) {
        const identifier = this.readIdentifier();
        const reserved = KEYWORDS[identifier];

         if (reserved === TokenType.Component) {
          this.token(identifier, TokenType.Component);

        } else if (reserved === TokenType.Column) {
          this.token(identifier, TokenType.Column);

        } else if (reserved === TokenType.Export) {
          this.token(identifier, TokenType.Export);

        } else {
          this.token(identifier, TokenType.Identifier);
          
        }
      }
    }
    this.token("EndOfFile", TokenType.EOF);
    return this.tokens;
  }
}
