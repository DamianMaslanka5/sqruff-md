#!/usr/bin/env bash

set -euo pipefail

artifacts_dir="release-artifacts"
tag=$(git describe --exact-match --tags || echo "0.0.1-test.4")

echo "Creating release for tag: $tag"

mkdir -p "$artifacts_dir"

echo "Downloading artifacts from build workflow..."

latestWorkflowId=$(gh run list --branch main --workflow Build --limit 1 --json databaseId | jq '.[0].databaseId')

GITHUB_TOKEN=$(gh auth token)

echo "Getting artifact download URLs..."
artifacts_json=$(curl -L \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: Bearer $GITHUB_TOKEN" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    "https://api.github.com/repos/DamianMaslanka5/sqruff-md/actions/runs/$latestWorkflowId/artifacts")

echo "$artifacts_json" | jq -r '.artifacts[] | "\(.name) \(.archive_download_url)"' | while read -r name url; do
    echo "Downloading artifact $name from $url..."
    curl -L \
        -H "Authorization: Bearer $GITHUB_TOKEN" \
        -o "$artifacts_dir/$name.zip" \
        "$url"
done

gh release create "$tag" \
              --generate-notes \
              --draft \
              --prerelease \
              "$artifacts_dir"/*
