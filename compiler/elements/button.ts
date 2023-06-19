import { HTMLCompiler } from "../htmlElements.ts";

export function CompileButton(type: string, text: string) {
  // log("button");
  HTMLCompiler.output += `<button type="${type}">${text}</button>`;
}
