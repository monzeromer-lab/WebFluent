export function ButtonAttribute(attribute: Record<string, string>) {
    for (const key in attribute) {
      switch (key) {
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
            `${key} is not a valid button attribute`
          );
          Deno.exit(1);
      }
    }
  }