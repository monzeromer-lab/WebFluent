import { TokenType } from "../lexer/types.ts";
import { IASTNode } from "../parser/interfaces/IAstNode.ts";

export class HTMLCompiler {
  private output = "";

  public compile(ast: IASTNode[]): string {
    // console.log(ast);

    this.visitNodes(ast);
    return this.output;
  }

  private visitNodes(nodes: IASTNode[]): void {
    for (const node of nodes) {
      this.visit(node);
    }
  }

  private visit(node: IASTNode): void {
    switch (node.type) {
      case TokenType.Page:
        // log("page");
        this.output += `<!-- Start Page ${node.value} -->\n<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta http-equiv="X-UA-Compatible" content="IE=edge"><meta name="viewport" content="width=device-width, initial-scale=1.0"><title>${node.value}</title></head><body>`;
        this.visitNodes(node.children || []);
        this.output += `</body></html>\n<!-- End Page ${node.value} -->\n\n`;
        break;

      case TokenType.Component:
        // log("component");
        this.output += `<div class="${node.value} ${
          node.class ? node.class : ""
        }" ${this.renderAttributes(node.attributes)}>`;
        this.visitNodes(node.children || []);
        this.output += "</div>";
        break;

      case TokenType.Row:
        // log("row");
        this.output += `<div class='row ${
          node.class ? node.class : ""
        }' ${this.renderAttributes(node.attributes)}>`;
        this.visitNodes(node.children || []);
        this.output += "</div>";
        break;

      case TokenType.Container:
        // log("container");
        this.output += `<div class='container ${
          node.class ? node.class : ""
        }' ${this.renderAttributes(node.attributes)}>`;
        this.visitNodes(node.children || []);
        this.output += "</div>";
        break;

      case TokenType.Column:
        // log("column");
        this.output += `<div class='column ${
          node.class ? node.class : ""
        }' ${this.renderAttributes(node.attributes)}>`;
        this.visitNodes(node.children || []);
        this.output += "</div>";
        break;

      case TokenType.Input:
        // log("input");
        if (node.value == "text") {
          this.output += `<input class='input-${node.value} ${
            node.class ? node.class : ""
          }' type="${node.value}" ${this.renderAttributes(node.attributes)}>`;
          this.visitNodes(node.children || []);
        }
        break;

      case TokenType.Text:
        // log("text");
        this.output += `<p class='text ${node.class ? node.class : ""}'>`;
        // ${this.renderAttributes(node.attributes)}
        // deno-lint-ignore ban-ts-comment
        //@ts-ignore
        this.output += node.attributes.value;
        this.output += "</p>";
        break;

      case TokenType.Image:
        // log("image");
        this.output += `<img class='${node.class ? node.class : ""}' src="${
          node.value
        }" ${this.renderAttributes(node.attributes)}>`;
        break;

      case TokenType.Table:
        // log("table");
        this.output += `<table class='table ${
          node.class ? node.class : ""
        }' ${this.renderAttributes(node.attributes)}>`;
        //@ts-expect-error title could be undefiend
        if (node.attributes["title"]) {
          //@ts-expect-error title could be undefiend
          this.output += `<caption>${node.attributes["title"]}</caption>`;
        }
        this.visitNodes(node.children || []);
        this.output += "</table>";
        break;

      case TokenType.Trow:
        // log("trow");
        this.output += `<tr class='trow ${
          node.class ? node.class : ""
        }' ${this.renderAttributes(node.attributes)}>`;
        this.visitNodes(node.children || []);
        this.output += "</tr>";
        break;

      case TokenType.Thead:
        // log("thead");
        this.output += `<thead><tr class="trow">`;
        //@ts-expect-error value is forced to be an array
        node.value?.map((element) => {
          this.output += `<th>${element}</th>`;
        });
        this.output += `</tr></thead>`;
        break;

      case TokenType.Tdata:
        // log("tdata");
        //@ts-expect-error value is forced to be an array
        node.value?.map((element) => {
          this.output += `<td>${element}</td>`;
        });
        break;

      case TokenType.Tab:
        // log("tab");
        this.output += `<div class='tab ${
          node.class ? node.class : ""
        }' ${this.renderAttributes(node.attributes)}>`;
        this.visitNodes(node.children || []);
        this.output += "</div>";
        break;

      case TokenType.TabPage:
        // log("tab page");
        this.output += `<div class='tab-page ${
          node.class ? node.class : ""
        }' ${this.renderAttributes(node.attributes)}>`;
        this.visitNodes(node.children || []);
        this.output += "</div>";
        break;

      case TokenType.Dialog:
        // log("dialog");
        this.output += `<dialog>`;
        this.visitNodes(node.children || []);
        this.output += "</dialog>";
        break;

      case TokenType.Button:
        // log("button");
        //@ts-expect-error node value is forced to be an array here so no need to worry about the type error
        this.output += `<button type="${node.value[0]}">${node.value[1]}</button>`;
        break;

      case TokenType.EOF:
        // log("end of file");
        // nothing to do for EOF node
        break;

      default:
        console.log(
          "%cCompiler Info:",
          "color: blue;",
          "Unexpected token",
          //@ts-expect-error node[0].type is not a valid duo to the type of node `IASTNode`
          node.type || node[0].type + " From an array 'node[0].type';"
        );
        break;
    }
  }

  private renderAttributes(attributes?: Record<string, string>): string {
    if (!attributes) {
      return "";
    }
    return Object.entries(attributes)
      .map(([key, value]) => `${key}="${value}"`)
      .join(" ");
  }
}
