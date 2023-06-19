export async function getPublicUrls(url: string, request: Deno.RequestEvent) {
  let visitedUrls: string[] = url.split("/");
  if (visitedUrls.includes("public")) {
    visitedUrls = visitedUrls.slice(3, visitedUrls.length);
    // console.log(visitedUrls);
    switch (visitedUrls[0]) {
      case "public": {
        if (visitedUrls[1] === "css") {
          const data = await Deno.readTextFile(`./${visitedUrls.join("/")}`);
          // console.log(data);

          const response = new Response(data, {
            headers: new Headers({
              "Content-Type": "text/css",
            }),
          });
          request.respondWith(response);
        } else if (visitedUrls[1] === "image") {
          try {
            const fileData = await Deno.readFile(`./${visitedUrls.join("/")}`);

            request.respondWith(new Response(fileData));
          } catch (error) {
            console.log("%cWeb Server Info:", "color: blue;", `couldn't find the file "${visitedUrls.join("/")}"`);
            request.respondWith(new Response("File not found"));
          }
        } else {
          const response = new Response(
            "only getting css public request for now",
            {
              headers: new Headers({
                "Content-Type": "text/plain",
              }),
            }
          );
          request.respondWith(response);
        }
      }
    }
  }
}
