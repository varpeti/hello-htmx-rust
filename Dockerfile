FROM rust:latest
WORKDIR /app
RUN cargo install cargo-watch
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY edb_orm ./edb_orm
COPY index.html ./
# TODO Copy templates
#RUN cargo build
EXPOSE 8080
