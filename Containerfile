# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# OikosBot CLI — ecological & economic code analysis.
#
# Published as ghcr.io/hyperpolymath/oikos. NB: the GHCR package is named
# "oikos" while the binary and repo are "oikosbot" — that mismatch is
# intentional (the package being (re)linked is "oikos").
#
# Multi-stage, glibc-consistent (rust:slim builder -> debian:slim runtime),
# non-root. The oikosbot-fleet bridge is excluded from the default workspace,
# so this builds standalone with no gitbot-fleet dependency.
FROM rust:1.86-slim AS builder
WORKDIR /build
COPY . .
# Build only the CLI crate (binary name: oikosbot). --locked honours the
# committed Cargo.lock (matches CI). tree-sitter grammars compile via the
# C toolchain bundled in the official rust image.
RUN cargo build --release --locked -p oikosbot-cli

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -g 1000 oikos \
    && useradd -u 1000 -g oikos -m oikos
COPY --from=builder /build/target/release/oikosbot /usr/local/bin/oikosbot
USER oikos
WORKDIR /home/oikos
# No EXPOSE: this is a batch analysis CLI, not a long-running service.
# There is no `health` subcommand, so use --version as a liveness probe.
HEALTHCHECK --interval=30s --timeout=3s CMD ["oikosbot", "--version"]
ENTRYPOINT ["oikosbot"]
CMD ["--help"]
LABEL org.opencontainers.image.title="oikos (OikosBot CLI)" \
      org.opencontainers.image.description="Ecological & economic code analysis CLI (oikosbot-cli binary)" \
      org.opencontainers.image.source="https://github.com/hyperpolymath/oikosbot" \
      org.opencontainers.image.licenses="MPL-2.0" \
      org.opencontainers.image.authors="Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>"
