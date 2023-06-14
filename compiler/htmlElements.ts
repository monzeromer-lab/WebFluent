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
        this.output += `<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta http-equiv="X-UA-Compatible" content="IE=edge"><meta name="viewport" content="width=device-width, initial-scale=1.0"><title>${node.value}</title></head><body>`;
        this.visitNodes(node.children || []);
        this.output += "</body></html>";
        break;

      case TokenType.Component:
        
          this.output += `<div class="${node.value} ${node.class ? node.class : ""}" ${this.renderAttributes(node.attributes)}>`;
          this.visitNodes(node.children || []);
          this.output += "</div>";
        
        break;

      case TokenType.Row:
        this.output += `<div class='row ${node.class ? node.class : ""}' ${this.renderAttributes(
          node.attributes
        )}>`;
        this.visitNodes(node.children || []);
        this.output += "</div>";
        break;

      case TokenType.Column:
        this.output += `<div class='column ${node.class ? node.class : ""}' ${this.renderAttributes(
          node.attributes
        )}>`;
        this.visitNodes(node.children || []);
        this.output += "</div>";
        break;

      case TokenType.Input:
        if (node.value == "text") {
          this.output += `<input class='input-${node.value} ${node.class ? node.class : ""}' type="${
            node.value
          }" ${this.renderAttributes(node.attributes)}>`;
          this.visitNodes(node.children || []);
        }
        break;

      case TokenType.Text:
          this.output += `<p class='text ${node.class ? node.class : ""}'>`;
          // ${this.renderAttributes(node.attributes)}
          // deno-lint-ignore ban-ts-comment
          //@ts-ignore
          this.output += node.attributes.value;
          this.output += "</p>";
        break;

        case TokenType.Image:
          this.output += `<img class='text  ${node.class ? node.class : ""}' src="${node.value}" ${this.renderAttributes(node.attributes)}>`;
        break;

        case TokenType.Table:
          console.log("just found a table to compile but I'm not ready for it yet help me with it\n", node);
          break;
          

      case TokenType.EOF:
        // nothing to do for EOF node
        break;

      default:
        // console.log("Error at node:\n", JSON.stringify(node));
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
