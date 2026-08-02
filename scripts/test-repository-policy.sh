#!/bin/sh

set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
policy_tmp=$(mktemp -d "${TMPDIR:-/tmp}/collatz-policy.XXXXXX")
trap 'rm -rf "$policy_tmp"' EXIT HUP INT TERM

fixture="$policy_tmp/repository"
mkdir -p "$fixture/.githooks"

git init -q -b main "$fixture"
git -C "$fixture" config user.name "Repository Policy Test"
git -C "$fixture" config user.email "policy-test@example.invalid"

printf '%s\n' "baseline" > "$fixture/tracked.txt"
git -C "$fixture" add tracked.txt
git -C "$fixture" commit -q -m "Create test baseline"

cp "$repository_root/.githooks/pre-commit" "$fixture/.githooks/pre-commit"
cp "$repository_root/.githooks/pre-push" "$fixture/.githooks/pre-push"
chmod +x "$fixture/.githooks/pre-commit" "$fixture/.githooks/pre-push"

(
    cd "$fixture"
    "$repository_root/scripts/install-git-hooks.sh"
)

configured_path=$(git -C "$fixture" config --local --get core.hooksPath)
if [ "$configured_path" != ".githooks" ]; then
    echo "FAIL: installer did not select the tracked hook directory." >&2
    exit 1
fi

printf '%s\n' "blocked on main" >> "$fixture/tracked.txt"
git -C "$fixture" add tracked.txt
if git -C "$fixture" commit -m "Attempt direct main commit" >"$policy_tmp/pre-commit.log" 2>&1; then
    echo "FAIL: pre-commit allowed a commit on main." >&2
    exit 1
fi

git -C "$fixture" switch -q -c codex/policy-test
git -C "$fixture" commit -q -m "Allow topic branch commit"

local_oid=$(git -C "$fixture" rev-parse HEAD)
zero_oid=0000000000000000000000000000000000000000

if printf '%s %s %s %s\n' \
    refs/heads/codex/policy-test "$local_oid" refs/heads/main "$zero_oid" \
    | (cd "$fixture" && .githooks/pre-push origin example.invalid) \
        >"$policy_tmp/pre-push-main.log" 2>&1; then
    echo "FAIL: pre-push allowed a direct update of remote main." >&2
    exit 1
fi

if ! printf '%s %s %s %s\n' \
    refs/heads/codex/policy-test "$local_oid" \
    refs/heads/codex/policy-test "$zero_oid" \
    | (cd "$fixture" && .githooks/pre-push origin example.invalid) \
        >"$policy_tmp/pre-push-topic.log" 2>&1; then
    echo "FAIL: pre-push rejected a topic branch." >&2
    exit 1
fi

echo "Repository policy checks passed."
