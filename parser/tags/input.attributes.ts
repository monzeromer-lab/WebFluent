export function InputAttribute(attribute: Record<string, string>) {
  for (const key in attribute) {
    switch (key) {
      case "class":
      case "style":
      case "id":
      case "dir":
      case "hidden":
      case "type":
      case "name":
      case "value":
      case "alt":
        break;
      default:
        console.log(
          "%cCompiler Error:",
          "color: red;",
          `${key} is not a valid input attribute`
        );
        Deno.exit(1);
    }
  }
}
