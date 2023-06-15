import { ProjectConfig } from "../enviroment/config.ts";

export function log(msg: string) {
    if (ProjectConfig.debug) {
        console.log("%cDebug", "color: pink;", msg);
    }
}