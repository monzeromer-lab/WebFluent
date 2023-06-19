export function DialogAttribute(attribute: Record<string, string>) {
    for (const key in attribute) {
      switch (key) {
        case "class":
        case "style":
        case "id":
        case "dir":
        case "hidden":
        case "delay":
        case "path":
          break;
        default:
          console.log(
            "%cCompiler Error:",
            "color: red;",
            `${key} is not a valid dialog attribute`
          );
          Deno.exit(1);
      }
    }
  }
  