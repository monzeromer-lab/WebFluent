import { TokenType } from "../../../lexer/types.ts";
import { IASTNode } from "../../interfaces/IAstNode.ts";
import { Parser, ParserError } from "../../parser.ts";
import { parseTdata } from "./tdata.ts";
import { parseThead } from "./thead.ts";
import { parseTrow } from "./trow.ts";

export function praseTableContent(): IASTNode[] {
    const TableContent: IASTNode[] = [];

    while (Parser.currentToken && Parser.currentToken.type !== TokenType.CloseBrace) {
        switch (Parser.currentToken.type) {
            case TokenType.Thead:
                TableContent.push(parseThead());
                break;

            case TokenType.Trow:
                TableContent.push(parseTrow());
                break;

            case TokenType.Tdata:
                TableContent.push(parseTdata());
                break;

            default:
                console.log(`%cParser Error:`, 'color: red;', `Unexpected Token: ${Parser.currentToken.type} at ${Parser.currentToken.line}:${Parser.currentToken.column}\n${ParserError.genLog(Parser.tokens, Parser.currentToken.line as number, Parser.currentToken.column as number)}`);
                Deno.exit(1);
        }
    }

    return TableContent
}