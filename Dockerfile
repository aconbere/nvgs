FROM rustlang/rust:nightly-slim AS build
RUN rustup target add x86_64-unknown-linux-musl && \
    apt update && \
    apt install -y perl make libsqlite3-dev && \
    update-ca-certificates
COPY ./src ./src
COPY ./data ./data
COPY ./Cargo.lock .
COPY ./Cargo.toml .
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "nvgs"
RUN ["cargo", "build", "--release"]

FROM debian:bookworm-slim
RUN apt update && \
    apt install -y libsqlite3-dev
COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group
COPY --from=build --chown=nvgs:nvgs ./target/release/api /app/api
EXPOSE 80/tcp
ENTRYPOINT ["/app/api", "--path", "/index", "--address", "0.0.0.0:80"]
