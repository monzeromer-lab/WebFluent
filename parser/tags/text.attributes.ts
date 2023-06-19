export function TextAttribute(attribute: Record<string, string>) {
  for (const key in attribute) {
    switch (key) {
      case "class":
      case "style":
      case "id":
      case "dir":
      case "hidden":
      case "size":
      case "value":
        break;
      default:
        console.log(
          "%cCompiler Error:",
          "color: red;",
          `${key} is not a valid text attribute`
        );
        Deno.exit(1);
    }
  }
}
