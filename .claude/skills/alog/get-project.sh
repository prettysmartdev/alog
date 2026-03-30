#!/usr/bin/env bash
# Resolve the alog --project name for the current repository.
# Priority: git remote origin > Cargo.toml > package.json > pyproject.toml > folder name
# Outputs a single normalized project name (lowercase, hyphens).

GIT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null)
if [ -z "$GIT_ROOT" ]; then
  echo "unknown"
  exit 0
fi

# 1. Git remote origin URL
PROJECT=$(git remote get-url origin 2>/dev/null \
  | sed 's/.*[:/]\([^/]*\)\.git$/\1/;s/.*[:/]\([^/]*\)$/\1/')

# 2. Cargo.toml name field
if [ -z "$PROJECT" ] && [ -f "$GIT_ROOT/Cargo.toml" ]; then
  PROJECT=$(grep -m1 '^name' "$GIT_ROOT/Cargo.toml" \
    | sed 's/name[[:space:]]*=[[:space:]]*"\([^"]*\)"/\1/')
fi

# 3. package.json name field (strip npm scope prefix e.g. @org/pkg -> pkg)
if [ -z "$PROJECT" ] && [ -f "$GIT_ROOT/package.json" ]; then
  PROJECT=$(grep -m1 '"name"' "$GIT_ROOT/package.json" \
    | sed 's/.*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/' \
    | sed 's|^@[^/]*/||')
fi

# 4. pyproject.toml name field
if [ -z "$PROJECT" ] && [ -f "$GIT_ROOT/pyproject.toml" ]; then
  PROJECT=$(grep -m1 '^name' "$GIT_ROOT/pyproject.toml" \
    | sed 's/name[[:space:]]*=[[:space:]]*"\([^"]*\)"/\1/')
fi

# 5. Fallback: git root folder name
if [ -z "$PROJECT" ]; then
  PROJECT=$(basename "$GIT_ROOT")
fi

# Normalize: lowercase, spaces to hyphens
echo "$PROJECT" | tr '[:upper:]' '[:lower:]' | tr ' ' '-'
