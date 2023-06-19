import { HTMLCompiler } from "../../htmlElements.ts";

export function CompileTableData(values: string[] | undefined) {
  // log("tdata");
  values?.map((element) => {
    HTMLCompiler.output += `<td>${element}</td>`;
  });
}
