#!/usr/bin/env bash
# Publish the current state of main to GitHub as a squashed single-commit
# branch. The GitHub Actions workflow (.github/workflows/deploy.yml) then
# builds and deploys the demo to GitHub Pages.
#
# Usage:
#   git remote add github git@github.com:<user>/<repo>.git   # once
#   ./publish-github.sh                                      # to publish/update

set -e

# the remote must be configured
if ! git remote get-url github > /dev/null 2>&1; then
    echo "Error: no 'github' remote configured."
    echo "Add it once with: git remote add github git@github.com:<user>/<repo>.git"
    exit 1
fi

# we must be on main, with a clean working tree
current=$(git rev-parse --abbrev-ref HEAD)
if [ "$current" != "main" ]; then
    echo "Error: switch to main first (current branch: $current)"
    exit 1
fi
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: the working tree is not clean — commit your changes first."
    exit 1
fi

# create a fresh single root commit from the current tree of main
commit=$(git commit-tree "main^{tree}" -m "Forecasting routing inputs demo")
git branch -f github-publish "$commit"

# publish: push the squashed branch to GitHub's main (force, since the
# history is always rewritten)
git push github github-publish:main --force

echo ""
echo "Pushed a fresh single-commit history to github/main."
echo "GitHub Actions will now build and deploy the site."
