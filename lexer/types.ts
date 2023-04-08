export enum TokenType {
Component="Component",
Identifier="Identifier",
OpenBrace="OpenBrace",
CloseBrace="CloseBrace",
Column="Column",
Export="Export",
EOF="EOF",
Number="Number",
OpenParen="OpenParen",
CloseParen="CloseParen",
StringLiteral="StringLiteral",
Colon="Colon",
Page="Page",
Row="Row",
Input="Input",
HexColor="HexColor",
Dot=".",
Text="Text",
String="String",
Coma="Coma",
src="src",
Image="Image",
}

export type Token = {
    type: TokenType;
    // deno-lint-ignore no-explicit-any
    value?: any;
    line?: number;
    column?: number;
}