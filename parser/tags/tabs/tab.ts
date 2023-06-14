import { TokenType } from "../../../lexer/types.ts";
import { IASTNode } from "../../interfaces/IAstNode.ts";
import { Parser } from "../../parser.ts";
import { parseAttributes } from "../attributes.ts";
import { praseTabContent } from "./tabContent.ts";

export function parseTab(): IASTNode {
    Parser.expect(TokenType.Tab);
    Parser.expect(TokenType.OpenParen);
    const attributes = parseAttributes();
    Parser.expect(TokenType.CloseParen);

    Parser.expect(TokenType.OpenBrace);
    const content = praseTabContent();
    Parser.expect(TokenType.CloseBrace);

    return {
        type: TokenType.Tab,
        attributes, 
        children: content
    }
}