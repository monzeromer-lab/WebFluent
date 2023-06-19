import { IASTNode } from "../../../parser/interfaces/IAstNode.ts";
import { HTMLCompiler } from "../../htmlElements.ts"

export function CompileTab(Class: string | undefined, attributes: Record<string, string> | undefined, children: IASTNode[] | undefined) {
  // log("tab");
  HTMLCompiler.output += `<div class='tabs ${
    Class ? Class : ""
  }' ${HTMLCompiler.renderAttributes(attributes)}>`;
  HTMLCompiler.visitNodes(children || []);
  HTMLCompiler.output += "</div>";
}
