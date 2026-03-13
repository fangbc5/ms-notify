# ms-notify Docker 镜像（fbc-builder + scratch）
# 构建: docker build -f ms-notify/Dockerfile .（workspace 根目录）
# syntax=docker/dockerfile:1

FROM fbc-builder:latest AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM fbc-builder:latest AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json -p ms-notify
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release -p ms-notify && \
    cp target/release/ms-notify /app/ms-notify-bin && \
    strip /app/ms-notify-bin

FROM scratch
COPY --from=builder /app/ms-notify-bin /app/ms-notify
WORKDIR /app
EXPOSE 30104
ENTRYPOINT ["/app/ms-notify"]
