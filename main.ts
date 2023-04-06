import { Lexer } from "./lexer/lexer.ts";

const code = `Component Monzer {
    Column{

    }
}
    `;


const tokens = new Lexer(code).tokenize();

console.log(tokens);
