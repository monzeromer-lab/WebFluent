import { TokenType } from "../../../lexer/types.ts";
import { parseChildrens } from "../../childrens.ts";
import { IASTNode } from "../../interfaces/IAstNode.ts";
import { Parser } from "../../parser.ts";
import { parseAttributes } from "../attributes.ts";

export function parseTabPage(): IASTNode {
    Parser.expect(TokenType.TabPage);
    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    Parser.expect(TokenType.CloseParen);

    Parser.expect(TokenType.OpenBrace);
    const content = parseChildrens();
    Parser.expect(TokenType.CloseBrace);

    return {
        type: TokenType.TabPage,
        attributes, 
        children: content
    }
}