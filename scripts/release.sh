#!/bin/bash

# Ensure we're on the main branch
if [[ $(git rev-parse --abbrev-ref HEAD) != "main" ]]; then
    echo "Error: Not on main branch. Please checkout main before tagging."
    exit 1
fi

# Fetch the latest changes from origin
git fetch origin main

# Check if there are any incoming or outgoing changes
local_commit=$(git rev-parse HEAD)
remote_commit=$(git rev-parse origin/main)
base_commit=$(git merge-base HEAD origin/main)

if [[ $local_commit != $remote_commit ]]; then
    if [[ $local_commit == $base_commit ]]; then
        echo "Error: Local main branch is behind origin/main. Please pull latest changes."
        exit 1
    elif [[ $remote_commit != $base_commit ]]; then
        echo "Warning: Local main branch has diverged from origin/main."
        echo "Local and remote have different commits. Please make sure this is intended."
        read -p "Do you want to continue? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    else
        echo "Local main branch is ahead of origin/main. Proceeding with tagging."
    fi
fi

year=$(date +"%y")
month=$(date +"%m" | sed 's/^0//')
day=$(date +"%d" | sed 's/^0//')
version="${year}.${month}.${day}"

month_padded=$(date +"%m")
day_padded=$(date +"%d")
version_tag="v${year}.${month_padded}.${day_padded}"

if [[ -f "package.json" ]]; then
    sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"${version}\"/" package.json
fi
if [[ -f "src-tauri/Cargo.toml" ]]; then
    sed -i '' "s/^version = \"[^\"]*\"/version = \"${version}\"/" src-tauri/Cargo.toml
fi

git add package.json src-tauri/Cargo.toml
git commit -m "🚀 release ${version_tag}"

git tag "$version_tag"

git push origin main
git push origin "$version_tag"