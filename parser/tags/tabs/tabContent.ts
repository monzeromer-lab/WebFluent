import { TokenType } from "../../../lexer/types.ts";
import { IASTNode } from "../../interfaces/IAstNode.ts";
import { Parser, ParserError } from "../../parser.ts";
import { parseTabPage } from "./tabPage.ts";

export function praseTabContent(): IASTNode[] {
    const TabContent: IASTNode[] = [];

    while (Parser.currentToken && Parser.currentToken.type !== TokenType.CloseBrace) {
        switch (Parser.currentToken.type) {
            case TokenType.TabPage:
                TabContent.push(parseTabPage());
                break;

            default:
                console.log(`%cParser Error:`, 'color: red;', `Unexpected Token: ${Parser.currentToken.type} at ${Parser.currentToken.line}:${Parser.currentToken.column}\n${ParserError.genLog(Parser.tokens, Parser.currentToken.line as number, Parser.currentToken.column as number)}`);
                Deno.exit(1);
        }
    }

    return TabContent
}