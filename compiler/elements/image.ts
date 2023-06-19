import { HTMLCompiler } from "../htmlElements.ts";

export function CompileImage(
  src: string,
  Class: string,
  attributes: Record<string, string> | undefined
) {
  // log("image");
  HTMLCompiler.output += `<img class='${
    Class ? Class : ""
  }' src="${src}" ${HTMLCompiler.renderAttributes(attributes)}>`;
}
