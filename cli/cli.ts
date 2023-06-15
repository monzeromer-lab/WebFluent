import { HandleArgs } from "./hundleArgs.ts";
import { ConfigFileType } from "./cli.types.ts";
import { WebServer } from "../server/http.ts";
import { ProjectConfig } from "../enviroment/config.ts";

export async function Cli() {

  const jsonFilePath = `${Deno.cwd()}/webfluent.app.json`;
  // deno-lint-ignore no-unused-vars
  let configFile: ConfigFileType = {};

  try {
    const decoder = new TextDecoder("utf-8");
    const data = await Deno.readFile(jsonFilePath);
    configFile = JSON.parse(decoder.decode(data));

    // deno-lint-ignore no-unused-vars
  } catch (error) {
    console.error(
      `%cNo webfluent.app.json found try webfluent --init to create a new one`,
      "color: red;"
    );
    Deno.exit(1);
  }

  new HandleArgs(Deno.args).run();
  ProjectConfig.serve ? await WebServer() : null
}
