import { IASTNode } from "../../../parser/interfaces/IAstNode.ts";
import { HTMLCompiler } from "../../htmlElements.ts"

export function CompileTableRow(Class: string | undefined, attributes: Record<string, string> | undefined, children: IASTNode[] | undefined) {
  // log("trow");
  HTMLCompiler.output += `<tr class='trow ${
    Class ? Class : ""
  }' ${HTMLCompiler.renderAttributes(attributes)}>`;
  HTMLCompiler.visitNodes(children || []);
  HTMLCompiler.output += "</tr>";
}
