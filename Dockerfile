FROM rust:1.33

# I know this file doesn't do much but it at least enables
# testing in a clean environment, eh?
COPY src ./src
COPY Cargo.toml ./Cargo.toml

RUN cargo build --release

CMD ["target/release/milk"]
