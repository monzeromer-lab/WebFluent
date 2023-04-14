// deno-lint-ignore-file no-inferrable-types ban-ts-comment ban-types
import { TokenType } from "../../lexer/types.ts";
import { IStyle } from "../interfaces/IStyle.ts";
import { Parser } from "../parser.ts";
let style: IStyle = {};

export function parseStyle(): string | null {
  let currentStyleInProgress: boolean = true;
  let identifier: string | null = null;

  Parser.expect(TokenType.Style);
  Parser.expect(TokenType.OpenParen);
  if (Parser.currentToken?.type === TokenType.Identifier) {
    identifier =  Parser.currentToken?.value;
    Parser.expect(TokenType.Identifier);
  }
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
        Parser.expect(TokenType.String);
        Parser.expect(TokenType.CloseParen);

        checkIfMore();
        break;

      case TokenType.Border:
        Parser.expect(TokenType.Border);
        Parser.expect(TokenType.OpenParen);
        Parser.expect(TokenType.String);
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
        style.padding = Parser.currentToken.value;
        Parser.expect(TokenType.String);
        Parser.expect(TokenType.CloseParen);

        checkIfMore();
        break;

      case TokenType.FontColor:
        Parser.expect(TokenType.FontColor);
        Parser.expect(TokenType.OpenParen);
        style.FontColor = Parser.currentToken.value;
        Parser.expect(TokenType.HexColor);
        Parser.expect(TokenType.CloseParen);

        checkIfMore();
        break;

      case TokenType.Font:
        Parser.expect(TokenType.Font);
        Parser.expect(TokenType.OpenParen);
        style.Font = Parser.currentToken.value;
        Parser.expect(TokenType.String);
        Parser.expect(TokenType.CloseParen);

        checkIfMore();
        break;

      default:
        currentStyleInProgress = false;
        break;
    }
  }

  Parser.styleAST.push({ identifier, styleAST: style });

  return identifier;
}
