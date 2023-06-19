export function TabPageAttribute(attribute: Record<string, string>) {
    for (const key in attribute) {
      switch (key) {
        case "title":
        case "class":
        case "style":
        case "id":
        case "dir":
        case "hidden":
        case "icon":
          break;
        default:
          console.log(
            "%cCompiler Error:",
            "color: red;",
            `${key} is not a valid tab page attribute`
          );
          Deno.exit(1);
      }
    }
  }