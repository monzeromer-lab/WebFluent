import { TokenType } from "../../../lexer/types.ts";
import { IASTNode } from "../../interfaces/IAstNode.ts";
import { Parser } from "../../parser.ts";
import { parseAttribute, parseAttributes } from "../attributes.ts";
import { parseTdata } from "./tdata.ts";
import { parseThead } from "./thead.ts";

export function parseTrow(): IASTNode {

    const TrowContent: IASTNode[] = [];
    Parser.expect(TokenType.Trow);

    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    Parser.expect(TokenType.CloseParen);

    Parser.expect(TokenType.OpenBrace);
    while (Parser.currentToken && Parser.currentToken.type !== TokenType.CloseBrace) {
        switch (Parser.currentToken.type) {
            case TokenType.Thead:
                TrowContent.push(parseThead());
                break;

            case TokenType.Tdata:
                TrowContent.push(parseTdata());
                break;

            default:
                console.log(Parser.currentToken.type);
                
                console.log(`Parser Error: \nTable row must have tdata or thead only for this beta`);
                
        }
    }
    Parser.expect(TokenType.CloseBrace);
    
    return {
        type: TokenType.Trow,
        children: TrowContent,
        attributes
    }
}