FROM alpine:3.19
RUN apk add --no-cache libgcc curl

ENV USER=consul
ENV GROUPNAME=$USER
ENV UID=1000
ENV GID=1000

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

COPY downloads/consul-kv-sync /bin/

WORKDIR /

USER $UID:$GID
ENTRYPOINT ["/bin/consul-kv-sync"]
CMD ["-d", "/sync"]