# syntax=docker/dockerfile:1
# The RootMark container image (linux/amd64): the release's Linux binary,
# the default embedder model with its licence, MCP over stdio with the
# watcher. Built by .github/workflows/image.yml from the published release
# archive (dist/rootmark-<version>-x86_64-unknown-linux-gnu.tar.gz in the
# build context, checked against the release's SHA256SUMS before the build),
# so the image's binary is byte for byte the release's.
#
#   docker run --rm IMAGE conformance
#   docker run -i --rm --user "$(id -u):$(id -g)" -v "$HOME/notes:/vault" IMAGE   # MCP stdio + --watch
#
# No HTTP from the container: the API listens on 127.0.0.1 inside it only.
ARG BASE=debian:trixie-slim@sha256:a72b4433de1cf7d6bae0c4cee74d0547f58601a643b3375f070c51600d63b6b2

FROM ${BASE} AS unpack
ARG VERSION
RUN [ -n "$VERSION" ] || { echo "build-arg VERSION is required" >&2; exit 1; }
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl jq \
 && rm -rf /var/lib/apt/lists/*
COPY dist/rootmark-${VERSION}-x86_64-unknown-linux-gnu.tar.gz /tmp/rootmark.tar.gz
RUN tar -xzf /tmp/rootmark.tar.gz -C /tmp \
 && mv "/tmp/rootmark-${VERSION}-x86_64-unknown-linux-gnu" /opt/rootmark \
 && [ "$(/opt/rootmark/rootmark --version)" = "rootmark ${VERSION}" ]
# the default model, fetched at its pinned revision and checked against the
# manifest (size, sha256) by the release's own fetch_model.sh; its licence
# (Apache-2.0, the model card's) and an attribution file go with it
RUN mkdir -p /tmp/share/models \
 && cp /opt/rootmark/models/MANIFEST.json /opt/rootmark/models/fetch_model.sh /tmp/share/models/ \
 && bash /tmp/share/models/fetch_model.sh /tmp/share \
 && cp /opt/rootmark/LICENSE /tmp/share/models/minilm-l6-v2/LICENSE \
 && jq -r '"Model: \(.name)\nSource: https://huggingface.co/\(.source) (revision \(.revision))\nLicence: Apache-2.0 (the model card of \(.source); text in LICENSE)\nAuthors: the sentence-transformers authors; base model nreimers/MiniLM-L6-H384-uncased\nRedistributed unmodified in this image; files and checksums: MANIFEST.json"' \
      /tmp/share/models/MANIFEST.json >/tmp/share/models/minilm-l6-v2/ATTRIBUTION \
 && ROOTMARK_MODELS=/tmp/share/models /opt/rootmark/rootmark conformance >/tmp/conformance.json \
 && jq -e '.level_claimed == 3 and .passed == 39 and .total == 39' /tmp/conformance.json >/dev/null

FROM ${BASE}
ARG VERSION
ARG REVISION=unknown
LABEL org.opencontainers.image.title="RootMark" \
      org.opencontainers.image.description="Block-level lineage, provenance and a deterministic derived index for Markdown (MCP over stdio)" \
      org.opencontainers.image.source="https://github.com/rootmark-ai/rootmark" \
      org.opencontainers.image.documentation="https://docs.rootmark.ai/" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${REVISION}"
RUN groupadd --gid 10001 rootmark \
 && useradd --uid 10001 --gid 10001 --home-dir /home/rootmark --create-home --shell /usr/sbin/nologin rootmark \
 && mkdir -p /vault && chown 10001:10001 /vault
COPY --from=unpack /opt/rootmark/rootmark /usr/local/bin/rootmark
COPY --from=unpack /tmp/share/models/MANIFEST.json /usr/share/rootmark/models/MANIFEST.json
COPY --from=unpack /tmp/share/models/minilm-l6-v2 /usr/share/rootmark/models/minilm-l6-v2
COPY --from=unpack /opt/rootmark/LICENSE /opt/rootmark/NOTICE /opt/rootmark/THIRD_PARTY_LICENSES /usr/share/doc/rootmark/
ENV ROOTMARK_MODELS=/usr/share/rootmark/models
USER 10001:10001
WORKDIR /vault
VOLUME ["/vault"]
ENTRYPOINT ["rootmark"]
CMD ["serve", "--vault", "/vault", "--mcp", "stdio", "--watch"]
