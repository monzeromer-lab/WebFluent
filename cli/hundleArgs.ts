import { HTMLCompiler } from "../compiler/htmlElements.ts";
import { ProjectConfig } from "../enviroment/config.ts";
import { Enviroment } from "../enviroment/eval.ts";
import { Lexer } from "../lexer/lexer.ts";
import { TokenType } from "../lexer/types.ts";
// import { main } from "../main.ts";
import { IASTNode } from "../parser/interfaces/IAstNode.ts";
import { Parser } from "../parser/parser.ts";
import { cliInfo, Commands, GenerateSubs } from "./cli.types.ts";
import { fileExists, fixFileName, writeAFile } from "./helper.ts";
import { CodesTypes } from "./helper.types.ts";

interface Files {
  dir: string;
  name: string;
}
export class HandleArgs {
  private currentArg: Commands | GenerateSubs | null;

  private files: Files[] = [];

  private currentFileIndex: number | null = 0;
  private currentFile: Files | undefined | null;
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
    // console.log(`current Arg is: ${this.currentArg} Next is ${this.args[this.currentArgIndex]}`);

    if (this.currentArgIndex < this.args.length) {
      this.currentArg = this.args[this.currentArgIndex] as unknown as
        | Commands
        | GenerateSubs;
    } else {
      this.currentArg = null;
    }
  }

  private nextFile() {
    //@ts-ignore this var could be null
    this.currentFileIndex++;

    if ((this.currentFileIndex as number) < this.files.length) {
      //@ts-ignore setting it to the current file
      this.currentFile = this.files[this.currentFileIndex];
    } else {
      this.currentFile = null;
    }
  }

  private async hundleGenerateCommand() {
    this.nextArg();

    // create new file (maybe create a template and upload it to github and then when this command run download it here)
    while (this.currentArg) {
      switch (this.currentArg) {
        case GenerateSubs.component:
        case GenerateSubs.c:
          this.nextArg();
          await writeAFile(
            `Component ${fixFileName(this.currentArg)} () {
  Text(value: "Component ${fixFileName(this.currentArg)} Works!")
}`,
            fixFileName(this.currentArg),
            CodesTypes.Component
          );
          this.nextArg();
          break;

        case GenerateSubs.page:
        case GenerateSubs.p:
          this.nextArg();
          await writeAFile(
            `Page ${fixFileName(this.currentArg)} () {
  Text(value: "Page ${fixFileName(this.currentArg)} Works!")
}`,
            fixFileName(this.currentArg),
            CodesTypes.Page
          );
          this.nextArg();
          break;

        case GenerateSubs.style:
        case GenerateSubs.s:
          this.nextArg();
          await writeAFile(
            `Style(${fixFileName(this.currentArg)}).BackgroundColor(#1e1e1e)`,
            fixFileName(this.currentArg),
            CodesTypes.Style
          );
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

  //get one file name and type as a param
  private async BuildAndWriteFile(
    name: string,
    type: TokenType.Component | TokenType.Page
  ) {
    let astNode;

    //get the AstNode from the enviroment
    if (type == TokenType.Page) {
      astNode = Enviroment.getPage(name);
    }

    if (type == TokenType.Component) {
      astNode = Enviroment.getComponent(name);
    }

    // compile it
    const output = new HTMLCompiler().compile(astNode as unknown as IASTNode[]);

    // make sure build dir is available
    if (!(await fileExists(Deno.cwd() + "/build"))) {
      Deno.mkdir("./build", { recursive: true });
    }

    // create a new text encoder and encode the compiled ast node
    const encoder = new TextEncoder();
    const html = encoder.encode(output);

    // write the file with it's name
    await Deno.writeFile(`./build/${name}.html`, html, {
      create: true,
    });

    // return
    return;
  }

  private async ParseFile(dir: string, fileName: string) {
    const decoder = new TextDecoder("utf-8");

    const data = await Deno.readFile(dir);
    if (data.length === 0) {
      return;
    }

    const code = decoder.decode(data);
    const tokens = new Lexer(code, fileName).tokenize();

    const tempAst = Parser.parse(tokens).MarkupASTL as unknown as IASTNode[];

    switch (tempAst[0].type) {
      case TokenType.Page:
        Enviroment.pages.push(tempAst[0].value as string);
        Enviroment.setPage(tempAst[0].value as string, tempAst);

        this.BuildAndWriteFile(tempAst[0].value as string, TokenType.Page);
        break;

      case TokenType.Component:
        Enviroment.components.push(tempAst[0].value as string);
        Enviroment.setIdentifier(tempAst[0].value as string, tempAst);
        break;

      default:
        console.log(
          `Unexpected Error Happend while parsing ${
            this.currentFile?.name as string
          }:\n found a -> ${
            tempAst[0].type
          } <- type, which for some reason i didn't implement here yet`
        );
        Deno.exit(1);
    }

    this.nextFile();
    return;
  }

  private async parseFiles() {
    if (this.currentFile) {
      console.log("%cBuilding:", "color: green;", this.currentFile.name);

      await this.ParseFile(
        this.currentFile?.dir as string,
        this.currentFile.name
      );

      await this.parseFiles();
    }
  }

  private async build() {
    // start building
    console.log("%cStarted Building...", "color: green;");

    try {
      // get all the files available
      // store them in files array
      this.files = await this.getAllThe_WF_Files(Deno.cwd() + "/src");

      // set the current file to the first one
      this.currentFile = this.files[0];

      // parse them file after another
      // store all the page names in an Array and add there names in a new env Map
      await this.parseFiles();

      // build all the pages and write thier files

      // end the building process
      console.log("%cBuilding Done!", "color: green;");
    } catch (error) {
      console.log(
        `%cCli Error:`,
        "color: red;",
        `Unexpected Deno Error:\n${error}`
      );
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

        case Commands.debug:
          ProjectConfig.debug = true;
          this.nextArg();
          break;

          case Commands.serve:
            ProjectConfig.serve = true;
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
          console.log(
            `%cCli Error:`,
            "color: red;",
            `Unknown command:\n${this.currentArg}`
          );
          console.log(cliInfo);

          Deno.exit(1);
      }
    }
  }
}
