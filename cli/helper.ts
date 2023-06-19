import { ClasslessDe, TabBox, Themes } from "./cli.types.ts";
import { CodesTypes, FileTypes } from "./helper.types.ts";

export async function fileExists(dir: string): Promise<boolean> {
    try {
        const directory = await Deno.stat(dir);
        if (directory.isDirectory) {
            return true;
        }
    } catch (_error) {
        return false;
    }
    return false;
}

export async function writeAFile(text: string, fileName: string, type: CodesTypes | FileTypes) {
    const encoder = new TextEncoder();
    const data = encoder.encode(text);

    switch (type) {
        case CodesTypes.Component:
            if (!await fileExists("./src/components")) {
                Deno.mkdir("./src/components");
            }
            await Deno.writeFile(`./src/components/${fileName}Component.wf`, data);
            break;

        case CodesTypes.Page:
            if (!await fileExists("./src/pages")) {
                Deno.mkdir("./src/pages");
            }
            await Deno.writeFile(`./src/pages/${fileName}Page.wf`, data);
            break;

        case CodesTypes.Style:
            if (!await fileExists("./src/styles")) {
                Deno.mkdir("./src/styles");
            }
            await Deno.writeFile(`./src/styles/${fileName}Style.wf`, data);
            break;

        case CodesTypes.Css:
            await Deno.writeFile(`./public/css/${fileName}`, data);
            break;

        case FileTypes.init:
            if(!await fileExists(`./${fileName}`)){
                await Deno.writeFile(`./${fileName}`, data);
            } else {
                console.log("%cCli Error:", "color: red;", `${fileName} is already exist`);
            }
            break;

    }
    console.log(`${type} ${fileName} created!`);
}

export function fixFileName(filename: string) {
    let tempName = '';
    for (let index = 0; index < filename.length; index++) {
        if (index == 0) {
            tempName += filename[index].toUpperCase();
        } else {
            tempName += filename[index];
        }
    }
    return tempName;
}

export async function AddStyleAssets() {
    await Deno.mkdir("public");
    await Deno.mkdir("public/css");
    await Deno.mkdir("public/image");
    await Deno.mkdir("public/js");
    await Deno.mkdir("public/font");
    const Classless = ClasslessDe.replace("[>correct<]", '\\');
    await writeAFile(Classless, "classless.css", CodesTypes.Css);
    await writeAFile(TabBox, "tabbox.css", CodesTypes.Css);
    await writeAFile(Themes, "themes.css", CodesTypes.Css);
}