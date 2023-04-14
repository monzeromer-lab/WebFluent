// deno-lint-ignore-file no-inferrable-types ban-ts-comment ban-types
import { Enviroment } from "../../enviroment/eval.ts";
import { Token, TokenType } from "../../lexer/types.ts";
import { IStyle } from "../interfaces/IStyle.ts";
import { Parser } from "../parser.ts";

export function parseStyle(): string {
  let style: IStyle = {};
  let currentStyleInProgress: boolean = true;
  let identifier: string;

  Parser.expect(TokenType.Style);
  Parser.expect(TokenType.OpenParen);
  identifier = Parser.currentToken?.value;
  Parser.expect(TokenType.Identifier);

  Parser.expect(TokenType.CloseParen);
  
  if (Parser.currentToken?.type !== TokenType.Dot) {
    return identifier;
  }
  
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
      //@ts-ignore
      case TokenType.Background:
        Parser.expect(TokenType.Background);
        Parser.expect(TokenType.OpenParen);
        Parser.expect(TokenType.String);
        Parser.expect(TokenType.CloseParen);

        checkIfMore();
        break;

        //@ts-ignore
      case TokenType.Border:
        Parser.expect(TokenType.Border);
        Parser.expect(TokenType.OpenParen);
        Parser.expect(TokenType.String);
        Parser.expect(TokenType.CloseParen);

        checkIfMore();
        break;

        //@ts-ignore
      case TokenType.Display:
        Parser.expect(TokenType.Display);
        Parser.expect(TokenType.OpenParen);
        Parser.expect(TokenType.Identifier);
        Parser.expect(TokenType.CloseParen);

        checkIfMore();
        break;

        //@ts-ignore
      case TokenType.Place:
        Parser.expect(TokenType.Place);
        Parser.expect(TokenType.OpenParen);
        Parser.expect(TokenType.Identifier);
        Parser.expect(TokenType.CloseParen);

        checkIfMore();
        break;

        //@ts-ignore
      case TokenType.Padding:
        Parser.expect(TokenType.Padding);
        Parser.expect(TokenType.OpenParen);
        style.padding = Parser.currentToken.value;
        Parser.expect(TokenType.String);
        Parser.expect(TokenType.CloseParen);

        checkIfMore();
        break;

        //@ts-ignore
      case TokenType.FontColor:
        Parser.expect(TokenType.FontColor);
        Parser.expect(TokenType.OpenParen);
        style.FontColor = Parser.currentToken.value;
        Parser.expect(TokenType.HexColor);
        Parser.expect(TokenType.CloseParen);

        checkIfMore();
        break;

        //@ts-ignore
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

  Enviroment.setStyle(identifier, style)
  Parser.styleAST.push({ identifier, styleAST: style });

  return identifier;
}
