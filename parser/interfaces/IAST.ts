import { IASTNode } from "./IAstNode.ts";
import { StyleAST } from "./IStyle.ts";

export interface IASTs {
    StyleAST?: StyleAST[],
    MarkupASTL?: IASTNode[],
  
  }