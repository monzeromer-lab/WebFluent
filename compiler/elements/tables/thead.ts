import { HTMLCompiler } from "../../htmlElements.ts";

export function CompileTableHead(titles: string[] | undefined) {
  // log("thead");
  HTMLCompiler.output += `<thead><tr class="trow">`;
  titles?.map((element) => {
    HTMLCompiler.output += `<th>${element}</th>`;
  });
  HTMLCompiler.output += `</tr></thead>`;
}
