import { OutputConfig, StylePlace, ThemeConfig } from "./config.types.ts";
import { StyleMode } from "./config.types.ts";
import { StyleConfig } from "./config.types.ts";

export class ProjectConfig {
  private name: string;
  private version: string;
  private auther: string;
  private Style: StyleConfig;
  private output: OutputConfig;
  private theme: ThemeConfig;

  constructor({
    name = "demo",
    version = "0.1.0",
    auther = "WebFluent",
    Style = { mode: StyleMode.classless, place: StylePlace.file },
    output = OutputConfig.pages,
    theme = ThemeConfig.light,
  }) {
    
    this.name = name;
    this.version = version;
    this.auther = auther;
    this.Style = Style;
    this.output = output;
    this.theme = theme;
  }

  public getProjectName() {
    return this.name;
  }

  public getProjectVersion() {
    return this.version;
  }

  public getProjectAuther() {
    return this.auther;
  }

  private validateStyleMode(mode: StyleMode) {
    switch(mode) {
        case StyleMode.classless:
        case StyleMode.tailwind:
        case StyleMode.material:
        return mode;
        
        default:
            console.log(`%cConfig Error:`, 'color: red;', "please provide a correct style mode classless | tailwind | material");
            Deno.exit(1);
            
    }
  }

  private validateStylePlace(place: StylePlace) {
    switch(place) {
        case StylePlace.styleTag:
        case StylePlace.file:
        return place;
        
        default:
            console.log(`%cConfig Error:`, 'color: red;', "please provide a correct style output place styleTag | file");
            Deno.exit(1);
            
    }
  }
  public getStyleConfigs() {
    return this.Style;
  }

  public getOutputConfig() {
    return this.output;
  }

  public getThemeConfig() {
    return this.theme;
  }
}
