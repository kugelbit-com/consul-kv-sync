FROM alpine:3.19
ARG TARGET=x86_64-unknown-linux-musl
ARG BINARY_NAME=consul-kv-sync
RUN apk add --no-cache libgcc curl

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
WORKDIR /tmp

COPY downloads/$BINARY_NAME /bin/

WORKDIR /

USER $UID:$GID
ENTRYPOINT ["/bin/${BINARY_NAME}"]
CMD ["-d", "/sync"]