#!/bin/bash
# Bump version in package.json. Accepts argument "patch", "minor" or "major"
npm version "${1:-patch}"

# npm version doesn't create git commit automatically if package.json is in a subfolder (like in our case)
version=$(jq -r '.version' package.json)
git add package.json
git commit -m "Bump npm version to $version"
git tag -a "npm-v$version" -m "Release npm version $version"

echo "Type: npm publish"
