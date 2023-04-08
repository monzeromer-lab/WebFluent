import { ASTNode, HTMLCompiler } from "./compiler/htmlElements.ts";
import { Lexer } from "./lexer/lexer.ts";
import { Parser } from "./parser/parser.ts";

const code = `Page Home (color: #1e1e1e) {
    Component Navbar () {
        Row () {
            Column () {
                TextInput(text,)
                Text(value: "Hii, this is me monzer")
                Image(src: "",)
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
