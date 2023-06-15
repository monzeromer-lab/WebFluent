enum StyleMode {
  css,
  tailwind,
}

enum StylePlace {
  inTag,
  file,
}

type StyleConfig = {
  mode: StyleMode;
  place: StylePlace;
};

export type ConfigFileType = {
  name?: string;
  auther?: string;
  version?: string;
  style?: StyleConfig;
};

export enum Commands {
  generate = "generate",
  g = "g",
  init = "init",
  build = "build",
  help = "help",
  h = "h",
  debug = "debug",
  serve= "serve"
}

export enum GenerateSubs {
  component = "component",
  c = "c",
  page = "page",
  p = "p",
  style = "style",
  s = "s",
}

// deno-lint-ignore no-inferrable-types
export const cliInfo: string = `Usage: webfluent [options] [command]

  A command-line interface for managing your WebFluent application.
  
  Options:
    -h, --help          Show this help message and exit.
  
  Commands:
    generate [options]  Generate a new file or component for your application.
      Options:
        c, component  Generate a new component.
        p, page       Generate a new page.
        s, style      Generate a new style file.
    init              Initialize a new WebFluent project.
    build               Build the WebFluent application.
    
  Run 'webfluent [command] --help' for more information on a specific command.`;
