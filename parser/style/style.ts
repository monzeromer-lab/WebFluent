import { TokenType } from "../../lexer/types.ts";
import { Parser } from "../parser.ts";

export function parseStyle() {
    let currentStyleInProgress: boolean = true;
    Parser.expect(TokenType.Style);
    Parser.expect(TokenType.OpenParen);
    const identifier = Parser.currentToken?.value;
    console.log(identifier);

    Parser.expect(TokenType.Identifier);
    Parser.expect(TokenType.CloseParen);
    Parser.expect(TokenType.Dot);

    const checkIfMore = () => {
      // @ts-ignore
      if (Parser.currentToken.type === TokenType.Dot) {
        Parser.expect(TokenType.Dot);
      } else {
        currentStyleInProgress = false;
      }
    };
    
    while (currentStyleInProgress) {
      switch (Parser.currentToken?.type) {
        case TokenType.Background:
          Parser.expect(TokenType.Background);
          Parser.expect(TokenType.OpenParen);
          // Parser.expect(<Something about the background>);
          Parser.expect(TokenType.CloseParen);

          checkIfMore();
          break;

        case TokenType.Border:
          Parser.expect(TokenType.Border);
          Parser.expect(TokenType.OpenParen);
          // Parser.expect(<Something about the background>);
          Parser.expect(TokenType.CloseParen);

          checkIfMore();
          break;

        case TokenType.Display:
          Parser.expect(TokenType.Display);
          Parser.expect(TokenType.OpenParen);
          Parser.expect(TokenType.Identifier);
          Parser.expect(TokenType.CloseParen);

          checkIfMore();
          break;

        case TokenType.Place:
          Parser.expect(TokenType.Place);
          Parser.expect(TokenType.OpenParen);
          Parser.expect(TokenType.Identifier);
          Parser.expect(TokenType.CloseParen);

          checkIfMore();
          break;

        case TokenType.Padding:
          Parser.expect(TokenType.Padding);
          Parser.expect(TokenType.OpenParen);
          // Parser.expect(TokenType.Identifier);

          Parser.expect(TokenType.CloseParen);

          checkIfMore();
          break;

        default:
          currentStyleInProgress = false;
          break;
      }
    }
  }