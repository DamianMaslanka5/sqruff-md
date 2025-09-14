#!/usr/bin/env bash

set -euo pipefail

mkdir -p tmp

curl -L https://github.com/ClickHouse/clickhouse-docs/archive/2b001e05b6fe2dd50c3c1e587999d9d11d0c3785.tar.gz -o clickhouse-docs.tar.gz

tar -xzf clickhouse-docs.tar.gz -C ./tmp --strip-components=1

output=$(cargo run -- lint --paths "./tmp/docs/**/*.md" 2>&1 || true)

echo "$output" | tail -10

if [[ $output == *"Unparsable found: "* ]]; then
    exit 0
else
    exit 1
fi
