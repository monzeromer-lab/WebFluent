import { IASTNode } from "../../parser/interfaces/IAstNode.ts";
import { HTMLCompiler } from "../htmlElements.ts";

export function CompileComponent(
  value: string | string[] | undefined,
  Class: string,
  attributes: Record<string, string> | undefined,
  children: IASTNode[]
) {
  // log("component");
  HTMLCompiler.output += `<div class="${value} ${
    Class ? Class : ""
  }" ${HTMLCompiler.renderAttributes(attributes)}>`;
  HTMLCompiler.visitNodes(children || []);
  HTMLCompiler.output += "</div>";
}