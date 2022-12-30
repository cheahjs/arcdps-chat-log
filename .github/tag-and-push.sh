#!/usr/bin/env bash

set -euxo pipefail

VERSION=$1

echo "Bumping version to ${VERSION}"

cargo set-version "$VERSION"

echo "Committing changes"

git add Cargo.toml
git commit -m "Updated version to ${VERSION}"
git tag -a "$VERSION" -m "$VERSION"
git push
