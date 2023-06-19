import { TokenType } from "../lexer/types.ts";
import { IASTNode } from "../parser/interfaces/IAstNode.ts";
import { CompileButton } from "./elements/button.ts";
import { CompileComponent } from "./elements/component.ts";
import { CompileDialog } from "./elements/dialog.ts";
import { CompileGrid } from "./elements/grid.ts";
import { CompileImage } from "./elements/image.ts";
import { CompileInput } from "./elements/input.ts";
import { CompilePage } from "./elements/page.ts";
import { CompileTable } from "./elements/tables/table.ts";
import { CompileTableData } from "./elements/tables/tdata.ts";
import { CompileTableHead } from "./elements/tables/thead.ts";
import { CompileTableRow } from "./elements/tables/trow.ts";
import { CompileTab } from "./elements/tabs/tab.ts";
import { CompileTabPage } from "./elements/tabs/tabPage.ts";
import { CompileText } from "./elements/text.ts";

export class HTMLCompiler {
  public static output = "";

  public static compile(ast: IASTNode[], clearOld = false): string {
    // console.log(ast);
    if (clearOld) {
      this.output = "";
    }

    this.visitNodes(ast);
    return this.output;
  }

  public static visitNodes(nodes: IASTNode[]): void {
    for (const node of nodes) {
      this.visit(node);
    }
  }

  public static visit(node: IASTNode): void {
    switch (node.type) {
      case TokenType.Page:
        CompilePage(node.children || [], node.value);
        break;

      case TokenType.Component:
        CompileComponent(
          node.value,
          node.class as string,
          node.attributes,
          node.children || []
        );
        break;

      case TokenType.Row:
        CompileGrid(
          TokenType.Row,
          node.class as string,
          node.attributes,
          node.children || []
        );
        break;

      case TokenType.Container:
        CompileGrid(
          TokenType.Container,
          node.class as string,
          node.attributes,
          node.children || []
        );
        break;

      case TokenType.Column:
        CompileGrid(
          TokenType.Column,
          node.class as string,
          node.attributes,
          node.children || []
        );
        break;

      case TokenType.Input:
        CompileInput(
          node.value,
          node.class as string,
          node.attributes,
          node.children || []
        );
        break;

      case TokenType.Text:
        //@ts-expect-error type attribute must be added!
        CompileText(node.value[0], node.value[1], node.class as string);
        break;

      case TokenType.Image:
        CompileImage(
          node.value as string,
          node.class as string,
          node.attributes
        );
        break;

      case TokenType.Table:
        CompileTable(node.class as string, node.attributes, node.children);
        break;

      case TokenType.Trow:
        CompileTableRow(node.class as string, node.attributes, node.children);
        break;

      case TokenType.Thead:
        CompileTableHead(node.value as string[]);
        break;

      case TokenType.Tdata:
        CompileTableData(node.value as string[]);
        break;

      case TokenType.Tab:
        CompileTab(node.class as string, node.attributes, node.children);
        break;

      case TokenType.TabPage:
        CompileTabPage(node.class as string, node.attributes, node.children);
        break;

      case TokenType.Dialog:
        CompileDialog(node.children);
        break;

      case TokenType.Button:
        //@ts-expect-error there's no error because the type and text are forced by the parser
        CompileButton(node.value[0], node.value[1]);
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

  public static renderAttributes(attributes?: Record<string, string>): string {
    if (!attributes) {
      return "";
    }
    return Object.entries(attributes)
      .map(([key, value]) => `${key}="${value}"`)
      .join(" ");
  }
}
