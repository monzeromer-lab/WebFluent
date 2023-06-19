import { StyleMode } from "../../enviroment/config.types.ts";

export function getStyle(type: StyleMode){
    switch (type) {
case StyleMode.classless:
    return `<link rel="stylesheet" href="/public/css/classless.css">
    <link rel="stylesheet" href="/public/css/tabbox.css">
    <link rel="stylesheet" href="/public/css/themes.css">`;
case StyleMode.tailwind:
    console.log("%cCompiler Info:", "color: blue;", `${StyleMode.tailwind} is not vailable yet, will use classless for now`);
    return `<link rel="stylesheet" href="/public/css/classless.css">
    <link rel="stylesheet" href="/public/css/tabbox.css">
    <link rel="stylesheet" href="/public/css/themes.css">`;

case StyleMode.material:
    console.log("%cCompiler Info:", "color: blue;", `${StyleMode.material} is not vailable yet, will use classless for now`);
    return `<link rel="stylesheet" href="/public/css/classless.css">
    <link rel="stylesheet" href="/public/css/tabbox.css">
    <link rel="stylesheet" href="/public/css/themes.css">`;

    default:
        console.log("%cCompiler Error:", "color: red;", `${type} is not a valid css style mode`);
        Deno.exit(1);
        
    }
}