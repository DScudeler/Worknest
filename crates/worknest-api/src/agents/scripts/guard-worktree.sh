#!/usr/bin/env bash
# Block Edit/Write/NotebookEdit on paths outside $CLAUDE_PROJECT_DIR.
set -euo pipefail
input="$(cat)"
path="$(echo "$input" | jq -r '.tool_input.file_path // .tool_input.path // empty')"
[ -z "$path" ] && exit 0
case "$path" in
    /*) abs="$path" ;;
    *)  abs="$CLAUDE_PROJECT_DIR/$path" ;;
esac
abs="$(realpath -m "$abs")"
proj="$(realpath -m "$CLAUDE_PROJECT_DIR")"
case "$abs" in
    "$proj"/*|"$proj") exit 0 ;;
    *) echo "guard: refusing edit outside worktree: $abs" >&2; exit 2 ;;
esac
