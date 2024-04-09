# DOCKERFILE for the `dnd` binary
#   by Lut99


##### BUILD #####
# Start with the Rust image
FROM rust:1-alpine3.19 AS build

# Add additional dependencies
RUN apk add --no-cache \
    musl-dev make cmake \
    openssl-dev openssl-libs-static pkgconf

# Copy over the source
RUN mkdir -p /source/target
COPY Cargo.toml /source/Cargo.toml
COPY Cargo.lock /source/Cargo.lock
COPY src /source/src

# Build it
WORKDIR /source
RUN --mount=type=cache,id=cargoidx,target=/usr/local/cargo/registry \
    --mount=type=cache,id=dndserver,target=/source/target \
    cargo build --release \
 && cp /source/target/release/dnd-server /source/dnd-server



##### RELEASE #####
# The release is alpine-based for quickness
FROM alpine:3.19 AS run

# Define some build args
ARG UID=1000
ARG GID=1000

# Setup a user mirroring the main one
RUN addgroup -g $GID dnd
RUN adduser -u $UID -G dnd -g "DnD" -D dnd

# Copy the binary from the build
COPY --chown=dnd:dnd --from=build /source/dnd-server /dnd-server

# Copy the client files
COPY --chown=dnd:dnd src/client /home/dnd/client

# Alrighty define the entrypoint and be done with it
USER dnd
ENTRYPOINT [ "/dnd-server", "--client-path", "/home/dnd/client" ]
