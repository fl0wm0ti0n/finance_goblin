# syntax=docker/dockerfile:1

ARG BUILD_ID=unknown
ARG RELEASE_TAG=unknown
ARG BUILD_TIMESTAMP=unknown

FROM rust:1.88-bookworm AS builder
WORKDIR /app

ARG BUILD_ID
ARG RELEASE_TAG
ARG BUILD_TIMESTAMP

ENV BUILD_ID=$BUILD_ID
ENV RELEASE_TAG=$RELEASE_TAG
ENV BUILD_TIMESTAMP=$BUILD_TIMESTAMP

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY backend/Cargo.toml backend/Cargo.lock* ./
RUN mkdir src && echo "fn main(){}" > src/main.rs && cargo build --release 2>/dev/null || true

COPY backend/ ./
RUN cargo build --release

FROM node:20-bookworm AS frontend
WORKDIR /frontend

ARG BUILD_ID
ARG RELEASE_TAG
ARG BUILD_TIMESTAMP

COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci --ignore-scripts 2>/dev/null || npm install
COPY frontend/ ./
RUN BUILD_ID=$BUILD_ID RELEASE_TAG=$RELEASE_TAG BUILD_TIMESTAMP=$BUILD_TIMESTAMP npm run build

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app

ARG BUILD_ID
ARG RELEASE_TAG
ARG BUILD_TIMESTAMP

LABEL org.opencontainers.image.revision="${BUILD_ID}"
LABEL org.opencontainers.image.version="${RELEASE_TAG}"
LABEL org.opencontainers.image.created="${BUILD_TIMESTAMP}"

COPY --from=builder /app/target/release/flow-finance-ai /usr/local/bin/flow-finance-ai
COPY backend/config/default.toml /app/config/default.toml
COPY backend/migrations /app/migrations
COPY --from=frontend /frontend/dist /app/static

ENV APP_CONFIG=/app/config/default.toml
ENV RUST_LOG=info

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --retries=3 CMD curl -f http://localhost:8080/health || exit 1

CMD ["flow-finance-ai"]
