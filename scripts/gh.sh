#!/bin/sh
# Runs the gh CLI as the gqf2008 account for this repository, without
# switching the globally active account.
#
# Usage: ./scripts/gh.sh pr create ...   (or: ./scripts/gh.sh issue list ...)

if command -v gh >/dev/null 2>&1; then
    export GH_TOKEN="$(gh auth token --user gqf2008 2>/dev/null || gh auth token 2>/dev/null)"
    exec gh "$@"
fi
echo "gh CLI not found" >&2
exit 127
