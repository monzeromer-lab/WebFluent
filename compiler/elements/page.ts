import { ProjectConfig } from "../../enviroment/config.ts";
import { IASTNode } from "../../parser/interfaces/IAstNode.ts";
import { HTMLCompiler } from "../htmlElements.ts";
import { getStyle } from "./page.assets.ts";

export function CompilePage(
  children: IASTNode[],
  value: string | string[] | undefined
) {
  // log("page");
  HTMLCompiler.output += `<!-- Start Page ${value} -->\n<!DOCTYPE html><html lang="en"><head>${getStyle(ProjectConfig.mode)}<meta charset="UTF-8"><meta http-equiv="X-UA-Compatible" content="IE=edge"><meta name="viewport" content="width=device-width, initial-scale=1.0"><title>${value}</title></head><body>`;
  HTMLCompiler.visitNodes(children || []);
  HTMLCompiler.output += `</body></html>\n<!-- End Page ${value} -->\n\n`;
}
