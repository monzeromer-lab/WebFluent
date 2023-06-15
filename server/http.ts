import { HTMLCompiler } from "../compiler/htmlElements.ts";
import { Enviroment } from "../enviroment/eval.ts";
import { IASTNode } from "../parser/interfaces/IAstNode.ts";

export async function WebServer() {
  console.log(`WebFluent Web Server Is Running at: http://localhost:8080`);
  async function handleHttp(conn: Deno.Conn) {
    
    for await (const e of Deno.serveHttp(conn)) {
      if (e.request.url !== "http://localhost:8080/favicon.ico") {
        console.log(`%cvisited: ${ e.request.url}`, "color: gray;");
        const pageAST: IASTNode[] = Enviroment.getPage(e.request.url);

        if (!pageAST) {
          e.respondWith(new Response("Page Not Found!"));
        } else {
          //@ts-expect-error pageAST isn't being saved as an Array so I had to but the pageAST var in one
          const page = await new HTMLCompiler().compile([pageAST]);
          
          e.respondWith(new Response(page, {
            headers: new Headers({
              'Content-Type': 'text/html',
              })
          }));
        }        
      }
    }
  }

  for await (const conn of Deno.listen({ port: 8080 })) {
    handleHttp(conn);
  }
}
