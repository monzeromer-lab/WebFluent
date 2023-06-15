import { TokenType } from "../../../lexer/types.ts";
import { IASTNode } from "../../interfaces/IAstNode.ts";
import { Parser, ParserError } from "../../parser.ts";

export function parseTdata(): IASTNode {

    const TableData: string[] = [];
    Parser.expect(TokenType.Tdata);
    Parser.expect(TokenType.OpenParen);
    while (Parser.currentToken && Parser.currentToken.type !== TokenType.CloseParen) {
        
        
        switch (Parser.currentToken.type) {
            case TokenType.String:
                TableData.push(Parser.currentToken.value);
                Parser.advance();
                break;
            
            case TokenType.Coma:
                Parser.advance();
                break;

            default:
                console.log(`%cParser Error:`, 'color: red;', `Unexpected Token -> ${Parser.currentToken.type} as "${Parser.currentToken.value}" at ${Parser.currentToken.line}:${Parser.currentToken.column}\n ${ParserError.genLog(Parser.tokens, Parser.currentToken.line as number, Parser.currentToken.column as number)}`);
                
        }
    }
    Parser.expect(TokenType.CloseParen);
    
    return {
        type: TokenType.Tdata,
        value: TableData
    }
}