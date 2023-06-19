export function ImageAttribute(attribute: Record<string, string>) {
  for (const key in attribute) {
    switch (key) {
      case "class":
      case "style":
      case "id":
      case "dir":
      case "hidden":
      case "src":
      case "srcset":
      case "alt":
        break;
      default:
        console.log(
          "%cCompiler Error:",
          "color: red;",
          `${key} is not a valid Image attribute`
        );
        Deno.exit(1);
    }
  }
}
