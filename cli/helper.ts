import { CodesTypes } from "./helper.types.ts";

export async function fileExists(dir: string): Promise<boolean> {
    try {
        const directory = await Deno.stat(dir);
        if (directory.isDirectory) {
            return true;
        }
    } catch (error) {
        return false;
    }
    return false;
}

export async function writeAFile(text: string, fileName: string, type: CodesTypes) {
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