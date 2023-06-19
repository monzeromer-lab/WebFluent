export function TableAttribute(attribute: Record<string, string>) {
  for (const key in attribute) {
    switch (key) {
      case "title":
      case "class":
      case "style":
      case "id":
      case "dir":
      case "hidden":
        break;
      default:
        console.log(
          "%cCompiler Error:",
          "color: red;",
          `${key} is not a valid table attribute`
        );
        Deno.exit(1);
    }
  }
}
