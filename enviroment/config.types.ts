export enum StyleMode{
    classless = "classless",
    tailwind = "tailwind",
    material = "material"
}
export enum StylePlace {
    styleTag = "styleTag",
    file = "file"
}
export type StyleConfig = {
    mode: StyleMode,
    place: StylePlace
}

export enum OutputConfig {
    pages = "pages",
    components = "components",
    both = "both"
}

export enum ThemeConfig {
    dark = "dark",
    light = "light",
    both = "both"
}