# Build a Docker image

> Build a Docker image of a WebFluent site: one stage installs wf and builds, a second serves the build with nginx.

The page: https://webfluent.monzeromer.dev/recipes/docker-image

This recipe packs a WebFluent site into a Docker image that serves it with
nginx, for a host that runs containers.

You need Docker on the machine that builds the image. The image installs
`wf` itself, as [Install wf on a build machine](https://webfluent.monzeromer.dev/recipes/install-wf-in-ci.md)
shows.

Add a `Dockerfile` at the project's root:

```dockerfile
FROM debian:stable-slim AS build
RUN apt-get update && apt-get install -y curl ca-certificates
RUN curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | WF_VERSION=v6.1.0 bash
WORKDIR /site
COPY . .
RUN /root/.webfluent/bin/wf build

FROM nginx:alpine
COPY --from=build /site/build /usr/share/nginx/html
```

Build and run the image:

```bash
docker build -t site .
docker run -p 8080:80 site
```

## How it works

1. The first stage, `build`, installs `curl`, installs `wf` into
   `/root/.webfluent/bin/`, copies the project in and builds it to
   `/site/build`. `WF_VERSION` pins the release, so the image builds the
   same way until you change it.
2. The second stage starts from `nginx:alpine` and copies only the built
   site into nginx's document root. `wf` and the project's sources stay
   behind in the first stage.
3. `docker run -p 8080:80` serves the site on port 8080 of your machine.

## Variations

To use the settings from [Serve a site with nginx](https://webfluent.monzeromer.dev/recipes/serve-with-nginx.md),
copy them into the second stage as `/etc/nginx/conf.d/default.conf`. When
TLS ends in front of the container, write `listen 80;` in place of the
`listen 443 ssl http2;` line. The second stage then reads:

```dockerfile
FROM nginx:alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /site/build /usr/share/nginx/html
```

Here `nginx.conf` is that `server` block, beside the `Dockerfile`, with its
`root` changed to `/usr/share/nginx/html`.

## Check it

Open `http://localhost:8080` and a deep link, and check the headers with
`curl -I http://localhost:8080/`. Then go through
[Check a deploy](https://webfluent.monzeromer.dev/recipes/check-a-deploy.md).
