# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
FROM rust:latest as cargo-build
RUN apt-get update
RUN apt-get install musl-tools -y
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /build
COPY Cargo.toml Cargo.toml
RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl
RUN rm -f target/x86_64-unknown-linux-musl/release/deps/app*
COPY . .
RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest
RUN addgroup -g 1000 app
RUN adduser -D -s /bin/sh -u 1000 -G app app
WORKDIR /home/app/bin/
COPY --from=cargo-build /build/target/x86_64-unknown-linux-musl/release/prometheus-teamspeak .
RUN chown app:app prometheus-teamspeak
USER app
EXPOSE 8010
CMD ["./prometheus-teamspeak"]