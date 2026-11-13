---
kind: "recipe"
route: "/recipes/docker-image"
group: "ship"
description: "1."
from: ["/docs/guide/deploying#build-a-docker-image"]
---
# Build a Docker image

1. Add a `Dockerfile`. The first stage installs `wf` and builds the site;
   the second serves it with nginx:

   ```dockerfile
   FROM debian:stable-slim AS build
   RUN apt-get update && apt-get install -y curl ca-certificates
   RUN curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
   WORKDIR /site
   COPY . .
   RUN /root/.webfluent/bin/wf build

   FROM nginx:alpine
   COPY --from=build /site/build /usr/share/nginx/html
   ```

2. To use the settings from [Serve with nginx](serve-with-nginx.md), copy
   them into the second stage as `/etc/nginx/conf.d/default.conf`. When TLS
   ends in front of the container, write `listen 80;` in place of the
   `listen 443 ssl http2;` line.
3. Build and run the image:

   ```bash
   docker build -t site .
   docker run -p 8080:80 site
   ```

**Verify it:** open `http://localhost:8080` and a deep link, and check the
headers with `curl -I http://localhost:8080/`.
