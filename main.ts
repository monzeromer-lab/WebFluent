import { ASTNode, HTMLCompiler } from "./compiler/htmlElements.ts";
import { Lexer } from "./lexer/lexer.ts";
import { Parser } from "./parser/parser.ts";

const code = `Page Home () {
    Component Navbar () {
        Row () {
            Column () {
                
            }
        }
    }
}`;


const tokens = new Lexer(code).tokenize();
console.log(tokens);

const parser: ASTNode[] = new Parser(tokens).parse()
console.log(JSON.stringify(parser));
const output = new HTMLCompiler().compile(parser)

console.log(output);
