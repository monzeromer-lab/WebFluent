import { Cli } from "./cli/cli.ts";
import { ASTNode, HTMLCompiler } from "./compiler/htmlElements.ts";
import { Lexer } from "./lexer/lexer.ts";
import { Parser } from "./parser/parser.ts";

export function main() {
  const code = `Page Home (color: #1e1e1e) {
        Component Navbar () {
            Row () {
                Column () {
                    Input(text,)
                    Text(value: "Hii, this is me monzer")
                    Image(src: "",)
                }
            }
        }
    }`;

  const tokens = new Lexer(code).tokenize();
  const parser: ASTNode[] = new Parser(tokens).parse();
  const output = new HTMLCompiler().compile(parser);

  console.log(output);
}


await Cli();