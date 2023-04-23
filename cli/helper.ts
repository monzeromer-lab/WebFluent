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