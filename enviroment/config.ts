import { OutputConfig, StylePlace, ThemeConfig } from "./config.types.ts";
import { StyleMode } from "./config.types.ts";
import { StyleConfig } from "./config.types.ts";

export class ProjectConfig {
  public static projectName: string;
  public static version: string;
  public static auther: string;
  public static Style: StyleConfig;
  public static output: OutputConfig;
  public static theme: ThemeConfig;

  public static debug: boolean;

  name = "demo";
  version = "0.1.0";
  auther = "WebFluent";
  Style = { mode: StyleMode.classless, place: StylePlace.file };
  output = OutputConfig.pages;
  theme = ThemeConfig.light;
  debug = false;

 

  public validateStyleMode(mode: StyleMode) {
    switch (mode) {
      case StyleMode.classless:
      case StyleMode.tailwind:
      case StyleMode.material:
        return mode;

      default:
        console.log(
          `%cConfig Error:`,
          "color: red;",
          "please provide a correct style mode classless | tailwind | material"
        );
        Deno.exit(1);
    }
  }

  public validateStylePlace(place: StylePlace) {
    switch (place) {
      case StylePlace.styleTag:
      case StylePlace.file:
        return place;

      default:
        console.log(
          `%cConfig Error:`,
          "color: red;",
          "please provide a correct style output place styleTag | file"
        );
        Deno.exit(1);
    }
  }
  
}
