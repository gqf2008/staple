#!/usr/bin/env bash
set -euo pipefail

# Manual upstream sync for gqf2008/paperclip:
#   fetch paperclipai/paperclip, fast-forward local master, push to your fork.
# Requires: git remotes `upstream` (paperclipai/paperclip) and `origin` (gqf2008/paperclip).
# The GitHub Actions workflow .github/workflows/sync-upstream.yml automates this.

cd "$(git rev-parse --show-toplevel)"

git fetch upstream master
behind="$(git rev-list --count HEAD..upstream/master)"
ahead="$(git rev-list --count upstream/master..HEAD)"

echo "master: $ahead commit(s) ahead, $behind commit(s) behind upstream/master"

if [ "$behind" -eq 0 ]; then
  echo "Already up to date with upstream/master."
  exit 0
fi

if [ "$ahead" -ne 0 ]; then
  echo "Local commits on master block fast-forward. Resolve manually:"
  echo "  git merge upstream/master   # then fix conflicts"
  exit 1
fi

git merge --ff-only upstream/master
git push origin master
echo "Synced: $(git rev-parse --short HEAD)"
