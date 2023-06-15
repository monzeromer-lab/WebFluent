import { TokenType } from "../../lexer/types.ts";
import { IStyle } from "./IStyle.ts";

/**
 * The nodes in the abstract syntax tree (AST) produced by the parser.
 */
export interface IASTNode {
  /** The type of the node (e.g. "Component", "Column"). */
  type: TokenType;
  /** The value of the node (e.g. the name of the component). */
  value?: string | string[];
  /** The children of the node (e.g. nested components or columns). */
  children?: IASTNode[];
  /** The custom attributes of the node. */
  attributes?: Record<string, string>;

  class?: string | IStyle;

  /* used for the web server only on pages */
  path?: string;
}