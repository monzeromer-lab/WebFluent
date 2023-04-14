// deno-lint-ignore-file ban-ts-comment no-explicit-any
import { Token } from "../lexer/types.ts";
import { IASTNode } from "../parser/interfaces/IAstNode.ts";
import { IStyle, StyleAST } from "../parser/interfaces/IStyle.ts";
import { Parser, ParserError } from "../parser/parser.ts";

export class Enviroment {
    private static Vals = new Map<string, IASTNode>()
    private static styles = new Map<string, IStyle>()

    public static setIdentifier(identifier: string, data: any){
        this.Vals.set(identifier, data);
        return data
    }

    public static getIdentifier(identifier: string, currentToken: Token): IASTNode {        
        if (this.Vals.get(identifier) === undefined) {
            console.log(
                `Error: "${identifier}" at ${
                  currentToken.line
                }:${currentToken.column}, is not defined \n   ${ParserError.genLog(
                  Parser.tokens,
                  //@ts-ignore
                  currentToken.line,
                  currentToken.column
                )}`
              );
              Deno.exit(1);
        }
        return this.Vals.get(identifier) as IASTNode;
    }

    public static setStyle(identifier: string, data: any){
        this.styles.set(identifier, data);
        return data
    }


    public static getStyle(identifier: string, currentToken: Token): StyleAST {        
        if (this.styles.get(identifier) === undefined) {
            console.log(
                `Error: Style "${identifier}" at ${
                  currentToken.line
                }:${currentToken.column}, is not defined \n   ${ParserError.genLog(
                  Parser.tokens,
                  //@ts-ignore
                  currentToken.line,
                  currentToken.column
                )}`
              );
              Deno.exit(1);
        }
        return this.styles.get(identifier) as StyleAST;
    }
}