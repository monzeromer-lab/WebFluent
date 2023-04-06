import { Lexer } from "./lexer/lexer.ts";
import { Parser } from "./parser.ts";

const code = `Component Mon8zer{
    Column{

    }
}`;


const tokens = new Lexer(code).tokenize();
console.log(tokens);

let parser = new Parser(tokens).parse()
console.log(JSON.stringify(parser));

