import { HTMLCompiler } from "../compiler/htmlElements.ts";
import { ProjectConfig } from "../enviroment/config.ts";
import { Enviroment } from "../enviroment/eval.ts";
import { IASTNode } from "../parser/interfaces/IAstNode.ts";
import { getPublicUrls } from "./http.utils.ts";

export async function WebServer() {
  console.log(`WebFluent Web Server Is Running at: http://localhost:${ProjectConfig.port}`);
  async function handleHttp(conn: Deno.Conn) {
    
    for await (const e of Deno.serveHttp(conn)) {
      if (e.request.url !== `http://localhost:${ProjectConfig.port}/favicon.ico`) {
        console.log(`%cvisited: ${ e.request.url}`, "color: gray;");
        const pageAST: IASTNode[] = Enviroment.getPage(e.request.url);
        if (!pageAST && e.request.url.split("/").includes("public")) {
          getPublicUrls(e.request.url, e);
        } else if(!pageAST) {
          e.respondWith(new Response("Page Not Found!"));
        } else {
          //@ts-expect-error pageAST isn't being saved as an Array so I had to but the pageAST var in one
          const page = HTMLCompiler.compile([pageAST], true);
          
          e.respondWith(new Response(page, {
            headers: new Headers({
              'Content-Type': 'text/html',
              })
          }));
        }        
      }
    }
  }

  for await (const conn of Deno.listen({ port: ProjectConfig.port })) {
    handleHttp(conn);
  }
}
