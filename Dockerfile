FROM rust:latest AS builder

WORKDIR /app
ADD sources.list /etc/apt/
# RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list
RUN apt update && apt install lld clang -y
RUN mkdir -vp ${CARGO_HOME:-$HOME/.cargo}

RUN tee -a ${CARGO_HOME:-$HOME/.cargo}/config.toml <<EOF
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
EOF
# RUN cargo check

COPY . .

ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app
ADD sources.list /etc/apt/
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends openssl ca-certificates && \
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod zero2prod

COPY configuration configuration
ENV APP_ENVIRONMENT=production

ENTRYPOINT [ "./zero2prod" ]