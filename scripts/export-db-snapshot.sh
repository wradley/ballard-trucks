#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
OUT_DIR="${ROOT_DIR}/.local-db"
OUT_FILE="${OUT_DIR}/ballard.sqlite"
VOLUME_NAME="${VOLUME_NAME:-ballard-trucks_app-data}"

mkdir -p "${OUT_DIR}"

docker run --rm \
  -v "${VOLUME_NAME}:/data:ro" \
  -v "${OUT_DIR}:/out" \
  alpine sh -c 'cp /data/ballard.sqlite /out/ballard.sqlite'

echo "Exported SQLite snapshot to ${OUT_FILE}"
