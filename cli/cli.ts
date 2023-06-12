import { HandleArgs } from "./hundleArgs.ts";
import { ConfigFileType } from "./cli.types.ts";

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
      `No webfluent.app.json file here try webfluent --init to create a new one`
    );
  }

  new HandleArgs(Deno.args).run();
}
