import { IASTNode } from "../../parser/interfaces/IAstNode.ts";
import { InputTypes } from "../../parser/interfaces/attributes.types.ts";
import { HTMLCompiler } from "../htmlElements.ts";

export function CompileInput(
  value: string | string[] | undefined,
  Class: string,
  attributes: Record<string, string> | undefined,
  children: IASTNode[]
) {
  // log("input");
  switch (value as InputTypes) {
    case InputTypes.Email:
    case InputTypes.Color:
    case InputTypes.IDate:
    case InputTypes.Checkbox:
    case InputTypes.Datetime:
    case InputTypes.DatetimeLocal:
    case InputTypes.IFile:
    case InputTypes.Image:
    case InputTypes.Hidden:
    case InputTypes.Month:
    case InputTypes.INumber:
    case InputTypes.Password:
    case InputTypes.Radio:
    case InputTypes.Range:
    case InputTypes.Reset:
    case InputTypes.Submit:
    case InputTypes.Search:
    case InputTypes.Tel:
    case InputTypes.Time:
    case InputTypes.Url:
    case InputTypes.Week:
    case InputTypes.Text:
      HTMLCompiler.output += `<input class='input-${value} ${
        Class ? Class : ""
      }' type="${value}" ${HTMLCompiler.renderAttributes(attributes)}>`;
      HTMLCompiler.visitNodes(children || []);
      break;

    default:
      console.log(
        "%cCompiler Error:",
        "color: red;",
        `Input type "${value}" is not valid use the same one available in html`
      );
      Deno.exit(1);
  }
}
