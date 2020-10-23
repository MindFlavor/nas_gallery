FROM rust:latest AS rust
WORKDIR /usr/src

RUN rustup default nightly
RUN rustup target add x86_64-unknown-linux-musl

# build rust backend
WORKDIR /usr/src/nas_gallery/rust
COPY rust/src ./src
COPY rust/Cargo.toml ./Cargo.toml
COPY rust/Cargo.lock ./Cargo.lock
COPY rust/Rocket.toml ./Rocket.toml
COPY rust/play256.png ./play256.png
RUN touch ./nas_gallery.log
RUN chown 1000:1000 ./nas_gallery.log
RUN touch ./audit.log
RUN chown 1000:1000 ./audit.log
RUN cargo install --target x86_64-unknown-linux-musl --path .

# build angular website
FROM node:latest AS angular

WORKDIR /usr/src/nas_gallery/typescript
COPY typescript .
RUN npm install -g @angular/cli
RUN npm install
RUN ng build --aot=true --buildOptimizer=true --prod

## Copy the statically-linked binary into a scratch container.
FROM alpine:latest AS final
RUN apk add  --no-cache ffmpeg
RUN apk add  --no-cache imagemagick
COPY --from=rust /usr/local/cargo/bin/nas_gallery .
COPY rust/Rocket.toml ./Rocket.toml
COPY rust/play256.png ./play256.png
COPY rust/example_config.toml /etc/nas_gallery/config.toml
COPY --from=rust /usr/src/nas_gallery/rust/nas_gallery.log /var/log/nas_gallery/nas_gallery.log
COPY --from=rust /usr/src/nas_gallery/rust/audit.log /var/log/nas_gallery/audit.log
COPY --from=angular /usr/src/nas_gallery/typescript/dist/simplegal/ /var/www/nas_gallery/.
USER 1000
CMD ["./nas_gallery", "/etc/nas_gallery/config.toml"]
EXPOSE 8000/tcp
