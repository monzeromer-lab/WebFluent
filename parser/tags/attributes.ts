// deno-lint-ignore-file ban-ts-comment
import { TokenType } from "../../lexer/types.ts";
import { Parser, ParserError } from "../parser.ts";

  /**
   * Parses a set of attributes (e.g. "class: 'my-class', style: 'color: red;'").
   * @returns A record of attribute names and values.
   */
  export function parseAttributes(): Record<string, string> {
    const attributes: Record<string, string> = {};

    while (
      Parser.currentToken &&
      Parser.currentToken.type === TokenType.Identifier
    ) {
      const { name, value } = parseAttribute();

      attributes[name] = value;
      
      //@ts-ignore idk but this statement made it work
      if (Parser.currentToken?.type === TokenType.Coma) {
        Parser.expect(TokenType.Coma);
      }
      
    }
    
    
    return attributes;
  }

    /**
   * Parses a single attribute (e.g. "class: 'my-class'").
   * Throws a ParserError if the current token is not an identifier followed by a colon and a string literal or a hex color code.
   * @returns The name and value of the attribute.
   */
    export function parseAttribute(): { name: string; value: string } {
        Parser.expect(TokenType.Identifier);
        const name = Parser.tokens[Parser.index - 1].value;
    
        Parser.expect(TokenType.Colon);
    
        let value: string;
    
        //@ts-ignore
        if (Parser.currentToken.type === TokenType.StringLiteral) {
          Parser.expect(TokenType.StringLiteral);
          value = Parser.tokens[Parser.index - 1].value;
          Parser.expect(TokenType.StringLiteral);
          //@ts-ignore
        } else if (
          //@ts-ignore
          Parser.currentToken.type === TokenType.HexColor &&
          //@ts-ignore
          Parser.currentToken.value.startsWith("#")
        ) {
          //@ts-ignore
          value = Parser.currentToken.value;
          Parser.advance();
        } else if (
          //@ts-ignore
          Parser.currentToken.type === TokenType.String
        ) {
          //@ts-ignore
          value = Parser.currentToken.value;
          Parser.advance();
        } else {
          throw new ParserError(
            //@ts-ignore
            Parser.currentToken,
            TokenType.StringLiteral
          );
        }
        
        if (Parser.tokens[Parser.index+1]?.type === TokenType.Coma) {
          Parser.expect(TokenType.Coma);
        }

        return { name, value };
      }