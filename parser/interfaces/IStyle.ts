import { TokenType } from "../../lexer/types.ts";

interface IBorder {
  radius?: string;
  style?: string;
  color?: string;
  collapse?: string;
  spacing?: string;
  image?: string;
  left?: string;
  right?: string;
  top?: string;
  bottom?: string;
}

enum EDisplay {
  Flex,
  Grid,
}

enum EPlace {
  Center,
  Right,
  Left,
}


export interface IStyle {
  border?: IBorder;
  padding?: string;
  display?: EDisplay;
  place?: EPlace;
  // FontColor?: TokenType.HexColor;
  Font?: string;
}

export interface StyleAST {
    identifier: string | null;
    styleAST: IStyle;
  }