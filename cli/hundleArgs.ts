import { main } from "../main.ts";
import { cliInfo, Commands, GenerateSubs } from "./types/cli.ts";

export class HandleArgs {
  private currentArg: Commands | GenerateSubs | null;

  private currentArgIndex: number = 0;

  private args: string[];

  constructor(args: string[]) {
    this.args = args;
    this.currentArg = args[this.currentArgIndex] as unknown as Commands | GenerateSubs;
  }

  private nextArg() {
    this.currentArgIndex++;

    if (this.currentArgIndex < this.args.length) {
      this.currentArg = this.args[this.currentArgIndex] as unknown as Commands | GenerateSubs;
    } else {
      this.currentArg = null;
    }
  }

  private hundleGenerateCommand(){
console.log("umm");

  }

  private build(){
    main();

  }

  private init(){

  }

  private help(){
    console.log(cliInfo);
  }

  run() {
    while(this.currentArg){
        switch(this.currentArg){
            case Commands.generate:
                this.hundleGenerateCommand();
                this.nextArg();
                break;

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
                this.help();
                this.nextArg();
                break;

                case Commands.h:
                    this.help();
                    this.nextArg();
                    break;

            default:
                console.log(`Unknown command: ${this.currentArg}`)
                console.log(cliInfo);
                Deno.exit(1);
        }
    }
  }
}
