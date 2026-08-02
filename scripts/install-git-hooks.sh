#!/bin/sh

set -eu

repository_root=$(git rev-parse --show-toplevel 2>/dev/null) || {
    echo "Error: run this command inside the Collatz Lab Git checkout." >&2
    exit 1
}

cd "$repository_root"

for hook in .githooks/pre-commit .githooks/pre-push; do
    if [ ! -x "$hook" ]; then
        echo "Error: required hook '$hook' is missing or not executable." >&2
        exit 1
    fi
done

git config --local core.hooksPath .githooks

configured_path=$(git config --local --get core.hooksPath)
if [ "$configured_path" != ".githooks" ]; then
    echo "Error: failed to configure the repository hook path." >&2
    exit 1
fi

echo "Installed Collatz Lab Git safeguards for this checkout."
