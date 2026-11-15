# syntax=docker/dockerfile:1
ARG NODE_IMAGE=node:22-bookworm-slim
ARG NGINX_IMAGE=nginxinc/nginx-unprivileged:stable-alpine
FROM ${NODE_IMAGE} AS build
WORKDIR /src
COPY frontend/package*.json ./
RUN if [ -f package-lock.json ]; then npm ci; else npm install; fi
COPY frontend/ ./
RUN npm run build

FROM ${NGINX_IMAGE}
COPY --from=build /src/dist/ /usr/share/nginx/html/
COPY deploy/containers/nginx.conf.template /etc/nginx/orchestrata.conf.template
COPY --chmod=0755 deploy/containers/start-web.sh /usr/local/bin/start-orchestrata-web
ENV ORCHESTRATA_API_UPSTREAM=orchestrata-api:3000
USER 101:101
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/start-orchestrata-web"]
