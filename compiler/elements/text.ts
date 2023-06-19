import { TextSizes } from "../../parser/interfaces/attributes.types.ts";
import { HTMLCompiler } from "../htmlElements.ts";
function getSizeTag(type: string): string {
    switch (type) {
        case "large":
            return "h2";
        case "larger":
            return "h1"
        case "medium":
            return "h3"
        case "normal":
            return "h4"
        case "small":
            return "h5"
        case "smaller":
            return "h6"
        
            default:
                console.log("%cCompiler Error:", "color: red;", `${type} is not a valid text type`);
                Deno.exit(1);
                
    }
return "";
}
export function CompileText(
  value: string,
  size: TextSizes = TextSizes.medium,
  Class: string
): void {
  // log("text");
  HTMLCompiler.output += `<${getSizeTag(size)} class='text ${Class ? Class : ""}'>`;
  // ${HTMLCompiler.renderAttributes(node.attributes)}
  // deno-lint-ignore ban-ts-comment
  //@ts-ignore
  HTMLCompiler.output += value;
  HTMLCompiler.output += `</${getSizeTag(size)}>`;
}
