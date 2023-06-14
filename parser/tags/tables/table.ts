import { TokenType } from "../../../lexer/types.ts";
import { IASTNode } from "../../interfaces/IAstNode.ts";
import { Parser } from "../../parser.ts";
import { parseAttributes } from "../attributes.ts";
import { praseTableContent } from "./tableContent.ts";

export function parseTable(): IASTNode {
    Parser.expect(TokenType.Table);
    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    Parser.expect(TokenType.CloseParen);

    Parser.expect(TokenType.OpenBrace);
    const content = praseTableContent();
    Parser.expect(TokenType.CloseBrace);

    return {
        type: TokenType.Table,
        attributes, 
        children: content
    }
}