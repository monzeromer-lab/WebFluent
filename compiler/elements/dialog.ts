import { IASTNode } from "../../parser/interfaces/IAstNode.ts";
import { HTMLCompiler } from "../htmlElements.ts";

export function CompileDialog(children: IASTNode[] | undefined) {
  // log("dialog");
  HTMLCompiler.output += `<dialog>`;
  HTMLCompiler.visitNodes(children || []);
  HTMLCompiler.output += "</dialog>";
}
