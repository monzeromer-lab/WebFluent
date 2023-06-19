import { Dir, InputTypes, TabButtonsSide, TextSizes } from "./attributes.types.ts";

interface IStaticAttributes {
    class?: string | string[],
    style?: string,
    id?: string | string[],
    dir?: Dir,
    hidden?: boolean
}

export interface IPageAttributes extends IStaticAttributes{
    title: string,
    path?: string,
    favicon?: string
}

export interface ITabAttributes extends IStaticAttributes{
    title?: string,
    side?: TabButtonsSide
}

export interface ITabPageAttributes extends IStaticAttributes {
    title?: string,
    icon?: string
}

export interface ITableAttributes extends IStaticAttributes {
    title?: string
}

export interface IComponentAttributes extends IStaticAttributes {
    path: string
}

export interface IimageAttributes extends IStaticAttributes {
    src: string,
    srcset: string,
    alt: string
}

export interface ITextAttributes extends IStaticAttributes {
    size: TextSizes,
    value: string
}

export interface IDialogAttributes extends IStaticAttributes {
    delay: number,
    path: string
}

export interface IinputAttributes extends IStaticAttributes {
    type: InputTypes,
    name?: string,
    value?: string
}