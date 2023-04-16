# Build Stage
FROM ghcr.io/evanrichter/cargo-fuzz:latest AS builder

# Add source code to the build stage.
ADD . /src
WORKDIR /src

# Compile the fuzzers.
RUN cd crates/rome_js_parser && cargo +nightly fuzz build --verbose

# Package Stage
FROM ubuntu:latest
COPY --from=builder /src/mayhem/corpus/ /corpus/
COPY --from=builder /src/crates/rome_js_parser/fuzz/target/x86_64-unknown-linux-gnu/release/fuzz_* /fuzzers/