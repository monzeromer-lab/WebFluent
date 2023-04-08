import { TokenType } from "../lexer/types.ts";

export interface ASTNode {
  type: TokenType;
  value?: string;
  children?: ASTNode[];
  attributes?: Record<string, string>;
}

export class HTMLCompiler {
  private output = "";

  public compile(ast: ASTNode[]): string {
    this.visitNodes(ast);
    return this.output;
  }

  private visitNodes(nodes: ASTNode[]): void {
    for (const node of nodes) {
      this.visit(node);
    }
  }

  private visit(node: ASTNode): void {
    switch (node.type) {
      case TokenType.Page:
        this.output += `<html><head><meta charset="utf-8"><title>${node.value}</title></head><body>`;
        this.visitNodes(node.children || []);
        this.output += "</body></html>";
        break;

      case TokenType.Component:
        if (node.value === "Navbar") {
          this.output += `<nav ${this.renderAttributes(node.attributes)}>`;
          this.visitNodes(node.children || []);
          this.output += "</nav>";
        }
        break;

      case TokenType.Row:
        this.output += `<div class='row' ${this.renderAttributes(
          node.attributes
        )}>`;
        this.visitNodes(node.children || []);
        this.output += "</div>";
        break;

      case TokenType.Column:
        this.output += `<div class='column' ${this.renderAttributes(
          node.attributes
        )}>`;
        this.visitNodes(node.children || []);
        this.output += "</div>";
        break;

      case TokenType.Input:
        if (node.value == "text") {
          this.output += `<input class='input-${node.value}' type="${
            node.value
          }" ${this.renderAttributes(node.attributes)}>`;
          this.visitNodes(node.children || []);
        }
        break;

      case TokenType.Text:
          //@ts-ignore
          this.output += `<p class='text' ${this.renderAttributes(node.attributes)}>`;
          //@ts-ignore
          this.output += node.attributes.value;
          this.output += "</p>";
        break;

        case TokenType.Image:
          //@ts-ignore
          this.output += `<img class='text' src="${node.value}" ${this.renderAttributes(node.attributes)}>`;
        break;

      case TokenType.EOF:
        // nothing to do for EOF node
        break;

      default:
        throw new Error(`Unknown node type: ${node.type}`);
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
