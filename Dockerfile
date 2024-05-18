ARG TARGET=x86_64-unknown-linux-musl
# Stage 1: Build
FROM rust:1.78-alpine3.19 as builder
ARG TARGET
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache pcc-libs-dev musl-dev pkgconfig openssl-dev perl gcompat libstdc++ gcc
RUN rustup target add "$TARGET"
# Create a new cargo project
WORKDIR /usr/src/consul-kv-sync
COPY . .

# Build for release.
RUN cargo build --locked --release --target "$TARGET"
RUN strip target/${TARGET}/release/consul-kv-sync

# Stage 2: Setup distroless
#FROM gcr.io/distroless/static-debian12:nonroot
FROM alpine:3.19
ARG TARGET
RUN apk add --no-cache libgcc

ENV USER=consul
ENV GROUPNAME=$USER
ENV UID=1001
ENV GID=1001

RUN addgroup \
    --gid "$GID" \
    "$GROUPNAME" \
    &&  adduser \
    --disabled-password \
    --gecos "" \
    --home "$(pwd)" \
    --ingroup "$GROUPNAME" \
    --no-create-home \
    --uid "$UID" \
    $USER

RUN mkdir -p /bin
WORKDIR /bin
COPY --from=builder /usr/src/consul-kv-sync/target/${TARGET}/release/consul-kv-sync .
USER $UID:$GID
ENTRYPOINT ["/bin/consul-kv-sync"]
CMD ["-d", "/sync"]