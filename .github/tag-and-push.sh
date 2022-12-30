#!/usr/bin/env bash

set -euxo pipefail

VERSION=$1

echo "Bumping version to ${VERSION}"

cargo set-version "$VERSION"
cargo generate-lockfile --offline

echo "Committing changes"

git add Cargo.toml Cargo.lock
git commit -m "Updated version to ${VERSION}"
git tag -a "$VERSION" -m "$VERSION"
git push --tags
