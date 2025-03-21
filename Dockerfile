FROM linuxcontainers/debian-slim:latest
COPY ./target/release/api /usr/bin/api
COPY ./target/release/cli /usr/bin/cli
EXPOSE 80/tcp
ENTRYPOINT ["/usr/bin/api", "--path", "/index", "--address", "localhost:80"]

