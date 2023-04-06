const conn = Deno.listen({ port: 8080 });
const httpConn = Deno.serveHttp(await conn.accept());
const e = await httpConn.nextRequest();
if (e) {
  e.respondWith(new Response("Hello World"));
}

async function handleHttp(conn: Deno.Conn) {
  for await (const e of Deno.serveHttp(conn)) {
    e.respondWith(new Response("Hello World"));
  }
}

for await (const conn of Deno.listen({ port: 80 })) {
  handleHttp(conn);
}
