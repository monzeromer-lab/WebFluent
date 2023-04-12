import { Cli } from "./cli/cli.ts";
import { ASTNode, HTMLCompiler } from "./compiler/htmlElements.ts";
import { Lexer } from "./lexer/lexer.ts";
import { Parser } from "./parser/parser.ts";

export function main() {
  const code = `Page Home () {
        Component Navbar () {
            Row () {
                // Style.Display(flex).Place(center).Padding(3,3,3,3)
                Column () {
                    Input(text)
                    Text(value: "Hii, this is me monzer", Style(Header).Border().Display(flex).Place(center).Padding(3,3,3,3).FontColor(#1e1e1e).Font(Lexend))
                    Image(src: "")
                    
                }
            }
        }
    }`;

  const tokens = new Lexer(code).tokenize();  
  console.log(tokens);
  const parser: ASTNode[] = Parser.parse(tokens);
  const output = new HTMLCompiler().compile(parser);

  console.log (output);
}


await Cli();