import { HandleArgs } from "./hundleArgs.ts";
import { ConfigFileType } from "./types/cli.ts";

/*
Usage: webfluent [options] [command]

A command-line interface for managing your WebFluent application.

Options:
  -h, --help          Show this help message and exit.

Commands:
  generate [options]  Generate a new file or component for your application.
    Options:
      -c, --component  Generate a new component.
      -p, --page       Generate a new page.
      -s, --style      Generate a new style file.
  build               Build the WebFluent application.
  
Run 'webfluent [command] --help' for more information on a specific command.
 */
export async function Cli() {

  const jsonFilePath = `${Deno.cwd()}/webfluent.app.json`;
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
    Deno.exit(1);
  }

  new HandleArgs(Deno.args).run();
}
