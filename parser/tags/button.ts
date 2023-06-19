import { TokenType } from "../../lexer/types.ts";
import { IASTNode } from "../interfaces/IAstNode.ts";
import { Parser } from "../parser.ts";
import { parseAttributes } from "./attributes.ts";
import { ButtonAttribute } from "./button.attributes.ts";

/**
   * Parses a Button node (e.g. "Button("<type>")").
   * Throws a ParserError if the current token is not "Component".
   * @returns An ASTNode representing the component.
   */
export function parseButton(): IASTNode {
    Parser.expect(TokenType.Button);
    const buttonType: string[] = [];
    Parser.expect(TokenType.OpenParen);

    buttonType.push(Parser.currentToken?.value);
    Parser.expect(TokenType.String);
    Parser.expect(TokenType.Coma);
  
    buttonType.push(Parser.currentToken?.value);
    Parser.expect(TokenType.String);

    const attributes = parseAttributes();
    ButtonAttribute(attributes);
    
    Parser.expect(TokenType.CloseParen);

    return {
      type: TokenType.Button,
      value: buttonType,
    };
  }