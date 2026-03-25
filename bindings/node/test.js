const { Template } = require("./index");

// Test 1: HTML rendering from string
console.log("Test 1: HTML fragment rendering...");
const tpl = Template.fromString(`
Page Test (path: "/", title: "Test") {
    Container {
        Heading("Hello, {name}!", h1)
        for item in items {
            Card(elevated) {
                Card.Body {
                    Text(item.name, bold)
                    Text("Price: $" + "{item.price}")
                }
            }
        }
        if showBadge {
            Badge("Active", success)
        }
    }
}
`);

const html = tpl.renderHtmlFragment({
  name: "World",
  items: [
    { name: "Widget", price: 9.99 },
    { name: "Gadget", price: 24.99 },
  ],
  showBadge: true,
});

console.log(html);
console.assert(html.includes("Hello, World!"), "Should interpolate name");
console.assert(html.includes("Widget"), "Should render loop items");
console.assert(html.includes("Gadget"), "Should render all loop items");
console.assert(html.includes("Active"), "Should render conditional badge");
console.log("  PASS\n");

// Test 2: Full HTML document
console.log("Test 2: Full HTML document...");
const fullHtml = tpl.renderHtml({ name: "Test", items: [], showBadge: false });
console.assert(fullHtml.includes("<!DOCTYPE html>"), "Should have doctype");
console.assert(fullHtml.includes("<style>"), "Should have embedded styles");
console.assert(!fullHtml.includes("Active"), "Should not render conditional");
console.log("  PASS\n");

// Test 3: PDF rendering
console.log("Test 3: PDF rendering...");
const pdf = tpl.renderPdf({ name: "PDF Test", items: [], showBadge: false });
console.assert(Buffer.isBuffer(pdf), "Should return a Buffer");
console.assert(pdf.toString("ascii", 0, 5) === "%PDF-", "Should be valid PDF");
console.log(`  PASS (${pdf.length} bytes)\n`);

// Test 4: Theme
console.log("Test 4: Dark theme...");
const darkHtml = tpl.withTheme("dark").renderHtml({ name: "Dark", items: [], showBadge: false });
console.assert(darkHtml.includes("<!DOCTYPE html>"), "Should render with dark theme");
console.log("  PASS\n");

console.log("All tests passed!");
