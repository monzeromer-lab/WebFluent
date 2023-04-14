import { Cli } from "./cli/cli.ts";
import { HTMLCompiler } from "./compiler/htmlElements.ts";
import { Lexer } from "./lexer/lexer.ts";
import { IASTs } from "./parser/interfaces/IAST.ts";
import { Parser } from "./parser/parser.ts";

export function main() {
  const code = `Page Home () {
        Component Navbar () {
            Row () {
                // Style.Display(flex).Place(center).Padding(3,3,3,3)
                Column () {
                    Input(text)
                    Text(value: "Hii, this is me monzer", Style(Header).Border("").Display(flex).Place(center).Padding("3px").FontColor(#1e1e1e).Font("Lexend"))
                    Image(src: "")
                }
            }
        }
    }`;

  const tokens = new Lexer(code).tokenize();  
  const parser: IASTs = Parser.parse(tokens);
  //@ts-ignore
  const output = new HTMLCompiler().compile(parser.MarkupASTL);

  console.log (output);
  console.log(parser.StyleAST);
}


await Cli();