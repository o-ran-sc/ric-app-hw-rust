# First get the builder image
FROM rust:1.65-bullseye as xapp-rust-builder

# libclang is a dependency of the bindgen crate. See https://internals.rust-lang.org/t/consider-shipping-libclang-with-rust-toolchain/12241
RUN apt-get update && apt-get install -y libclang-dev protobuf-compiler redis-server

ARG RMRTAG=4.8.2
# Get RMR library from Packagecloud
# Note: O-RAN-SC only publishes packages for Debian stretch, but stretch does not get official Rust 1.63 updates.
#       Installing this on Debian bullseye seems to work... Should probably request newer packages from O-RAN-SC.
RUN wget --content-disposition https://packagecloud.io/o-ran-sc/staging/packages/debian/stretch/rmr_${RMRTAG}_amd64.deb/download.deb && dpkg -i rmr_${RMRTAG}_amd64.deb && rm -rf rmr_${RMRTAG}_amd64.deb
RUN wget --content-disposition https://packagecloud.io/o-ran-sc/staging/packages/debian/stretch/rmr-dev_${RMRTAG}_amd64.deb/download.deb && dpkg -i rmr-dev_${RMRTAG}_amd64.deb && rm -rf rmr-dev_${RMRTAG}_amd64.deb


ENV PATH=/usr/local/cargo/bin:$PATH

RUN mkdir -p /ws
RUN mkdir -p /src
RUN mkdir -p /config
WORKDIR "/ws"

# Copy
COPY src/* /ws/src/
COPY config/* /ws/config/
COPY Cargo.toml /ws/

# Build Release Binary
RUN cargo build --release

RUN cargo test --release

# Final deployment container
FROM ubuntu:22.04

ENV CFG_FILE=config/config-file.json
ENV RMR_SEED_RT=config/uta_rtg.rt

RUN mkdir /config

COPY --from=xapp-rust-builder /ws/target/release/hw-rust /
COPY --from=xapp-rust-builder /ws/config/* /config/
COPY --from=xapp-rust-builder /usr/local/lib /usr/local/lib

RUN ldconfig

RUN chmod 755 /hw-rust
CMD /hw-rust
