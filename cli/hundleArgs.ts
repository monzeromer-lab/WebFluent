import { HTMLCompiler } from "../compiler/htmlElements.ts";
import { Lexer } from "../lexer/lexer.ts";
import { main } from "../main.ts";
import { IASTs } from "../parser/interfaces/IAST.ts";
import { Parser } from "../parser/parser.ts";
import { cliInfo, Commands, GenerateSubs } from "./types/cli.ts";
import { fileExists } from "./helper.ts";

interface Files {
  dir: string;
  name: string;
}
export class HandleArgs {
  private currentArg: Commands | GenerateSubs | null;

  private files: Files[] = [];

  private currentFileIndex: number|null = 0;
  private currentFile: Files|undefined|null;
  // deno-lint-ignore no-inferrable-types
  private currentArgIndex: number = 0;

  private args: string[];

  constructor(args: string[]) {
    this.args = args;
    this.currentArg = args[this.currentArgIndex] as unknown as
      | Commands
      | GenerateSubs;
  }

  private nextArg() {
    this.currentArgIndex++;

    if (this.currentArgIndex < this.args.length) {
      this.currentArg = this.args[this.currentArgIndex] as unknown as
        | Commands
        | GenerateSubs;
    } else {
      this.currentArg = null;
    }
  }

  private nextFile() {

    //@ts-ignore
    this.currentFileIndex++;

    if (this.currentFileIndex as number < this.files.length) {
      //@ts-ignore
      this.currentFile = this.files[this.currentFileIndex];
    } else {
      this.currentFile = null;
    }
  }

  private hundleGenerateCommand() {
    this.nextArg();

    // create new file (maybe create a template and upload it to github and then when this command run download it here)
    while (this.currentArg) {
      switch (this.currentArg) {
        case GenerateSubs.component:
        case GenerateSubs.c:
          console.log("Need a new component huh?");
          this.nextArg();
          break;

        case GenerateSubs.page:
        case GenerateSubs.p:
          console.log("Need a new page huh?");
          this.nextArg();
          break;

        case GenerateSubs.style:
        case GenerateSubs.s:
          console.log("Need a new style huh?");
          this.nextArg();
          break;

        default:
          console.log(cliInfo);
          this.nextArg();

          Deno.exit(1);
      }
    }

    return;
  }

  private async getAllThe_WF_Files(folderDir: string) {
    for await (const dirEntry of Deno.readDir(folderDir)) {
      const filePath = `${folderDir}/${dirEntry.name}`;
      if (dirEntry.isFile && dirEntry.name.endsWith(".wf")) {
        this.files.push({ dir: filePath, name: dirEntry.name });
      } else if (dirEntry.isDirectory) {
        await this.getAllThe_WF_Files(filePath);
      }
    }

    return this.files;
  }

  private async BuildAndWriteFiles() {
    const decoder = new TextDecoder("utf-8");
    //@ts-ignore
    const data = await Deno.readFile(this.currentFile.dir);
    if (data.length === 0) {
      return;
    }

    const code = decoder.decode(data);
    const tokens = new Lexer(code).tokenize();
    const parser: IASTs = Parser.parse(tokens);
    //@ts-ignore
    const output = new HTMLCompiler().compile(parser.MarkupASTL);
    if (!(await fileExists(Deno.cwd() + "/build"))) {
      Deno.mkdir("./build", { recursive: true });
    }

    const encoder = new TextEncoder();
    const html = encoder.encode(output);
    //@ts-ignore
    await Deno.writeFile(`./build/${this.currentFile.name.split(".")[0]}.html`, html, {
      create: true,
    });

    this.nextFile();
    return;
  }

  private async build() {
    console.log("Started Building.");

    try {
      Deno.readDir(Deno.cwd() + "/src");
      this.files = await this.getAllThe_WF_Files(Deno.cwd() + "/src");
      this.currentFile = this.files[0];
      
      while (this.currentFile) {  
        console.log(`Building ${this.currentFile.name}.`);      
        if (this.currentFile) {
          await this.BuildAndWriteFiles();
        }
      }
      console.log("Building Done.");
    } catch (error) {
      console.log(`Build Error: \n ${error}`);
      Deno.exit(1);
    }
  }

  private init() {
    // create new file (maybe create a template and upload it to github and then when this command run download it here)
    return;
  }

  private help() {
    console.log(cliInfo);
    Deno.exit(0);
  }

  run() {
    if (!this.currentArg) {
      console.log(cliInfo);
    }

    while (this.currentArg) {
      switch (this.currentArg) {
        case Commands.generate:
        case Commands.g:
          this.hundleGenerateCommand();
          this.nextArg();
          break;

        case Commands.build:
          this.build();
          this.nextArg();
          break;

        case Commands.init:
          this.init();
          this.nextArg();
          break;

        case Commands.help:
        case Commands.h:
          this.help();
          this.nextArg();
          break;

        default:
          console.log(`Unknown command: ${this.currentArg}`);
          console.log(cliInfo);

          Deno.exit(1);
      }
    }
  }
}

function FileExists(arg0: string) {
  throw new Error("Function not implemented.");
}
