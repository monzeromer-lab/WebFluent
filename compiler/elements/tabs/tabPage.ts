import { IASTNode } from "../../../parser/interfaces/IAstNode.ts";
import { HTMLCompiler } from "../../htmlElements.ts"

export function CompileTabPage(Class: string | undefined, attributes: Record<string, string> | undefined, children: IASTNode[] | undefined) {
  // log("tab page");
  HTMLCompiler.output += `<div class='tab ${
    Class ? Class : ""
  }' ${HTMLCompiler.renderAttributes(attributes)}>`;
  HTMLCompiler.visitNodes(children || []);
  HTMLCompiler.output += "</div>";
}
