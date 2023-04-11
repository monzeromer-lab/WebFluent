import { main } from "../main.ts";
import { cliInfo, Commands, GenerateSubs } from "./types/cli.ts";

export class HandleArgs {
  private currentArg: Commands | GenerateSubs | null;

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

  private build() {
    main();
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
          break;

        case Commands.build:
          this.build();
          break;

        case Commands.init:
          this.init();
          break;

        case Commands.help:
        case Commands.h:
          this.help();
          break;

        default:
          console.log(`Unknown command: ${this.currentArg}`);
          console.log(cliInfo);
          Deno.exit(1);
      }
    }
  }
}
