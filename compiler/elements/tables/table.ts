import { IASTNode } from "../../../parser/interfaces/IAstNode.ts";
import { HTMLCompiler } from "../../htmlElements.ts";

export function CompileTable(
  Class: string,
  attributes: Record<string, string> | undefined,
  children: IASTNode[] | undefined
) {
  // log("table");
  HTMLCompiler.output += `<table class='table ${
    Class ? Class : ""
  }' ${HTMLCompiler.renderAttributes(attributes)}>`;
  HTMLCompiler.visitNodes(children || []);
  //@ts-expect-error title could be undefiend
  if (attributes["title"]) {
    //@ts-expect-error title could be undefiend
    HTMLCompiler.output += `<caption>${attributes["title"]}</caption>`;
  }
  HTMLCompiler.output += "</table>";
}
