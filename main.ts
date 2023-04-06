import { Lexer } from "./lexer/lexer.ts";
import { Parser } from "./parser/parser.ts";

const code = `Page Home (name, "name,") {
    Component Navbar () {
        Row () {
            Column () {
                TextInput()
            }
        }
    }
}`;


const tokens = new Lexer(code).tokenize();
console.log(tokens);

const parser = new Parser(tokens).parse()
console.log(JSON.stringify(parser));

