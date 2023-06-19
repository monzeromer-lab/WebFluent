import { HandleArgs } from "./hundleArgs.ts";
import { ConfigFileType } from "./cli.types.ts";
import { WebServer } from "../server/http.ts";
import { ProjectConfig } from "../enviroment/config.ts";
import { StyleMode } from "../enviroment/config.types.ts";
let able = true;
export async function Cli() {

  const jsonFilePath = `${Deno.cwd()}/webfluent.app.json`;
  // deno-lint-ignore no-unused-vars
  let configFile: ConfigFileType;

  try {
    const decoder = new TextDecoder("utf-8");
    const data = await Deno.readFile(jsonFilePath);
    configFile = JSON.parse(decoder.decode(data)) as ProjectConfig;
    ProjectConfig.mode = configFile.mode as unknown as StyleMode
    if (configFile.port) {
      ProjectConfig.port = configFile.port;
    }
    
    // deno-lint-ignore no-unused-vars
  } catch (error) {
    able = false;
    new HandleArgs(Deno.args, false).run();
  }

  if (able) {
    new HandleArgs(Deno.args).run();
  }
  ProjectConfig.serve ? await WebServer() : null
}
