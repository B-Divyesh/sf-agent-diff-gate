FROM node:22-alpine AS frontend
WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm ci
COPY frontend ./frontend
COPY vite.config.ts tsconfig.json ./
RUN npm run build

FROM rust:1-alpine AS build
ARG BUILD_SHA=dev
WORKDIR /app
RUN apk add --no-cache musl-dev
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
RUN cargo build --release

FROM alpine:3.21
ARG BUILD_SHA=dev
ENV BUILD_SHA=$BUILD_SHA PORT=8080
RUN addgroup -S app && adduser -S app -G app && mkdir -p /data /app/dist && chown -R app:app /data /app
COPY --from=build /app/target/release/diff-gate /app/diff-gate
COPY --from=frontend /app/dist /app/dist
USER app
WORKDIR /app
EXPOSE 8080
CMD ["/app/diff-gate"]
