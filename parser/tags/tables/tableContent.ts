import { TokenType } from "../../../lexer/types.ts";
import { IASTNode } from "../../interfaces/IAstNode.ts";
import { Parser } from "../../parser.ts";
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
                console.log(Parser.currentToken.type);
                break;
        }
    }

    return TableContent
}