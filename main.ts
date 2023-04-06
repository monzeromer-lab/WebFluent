import { Lexer } from "./lexer/lexer.ts";
import { Parser } from "./parser/parser.ts";

const code = `Page Home {
    Component Navbar {
        Column {

        }
    }
}`;


const tokens = new Lexer(code).tokenize();
console.log(tokens);

const parser = new Parser(tokens).parse()
console.log(JSON.stringify(parser));

