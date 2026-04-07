#!/usr/bin/env bash
set -euo pipefail

SUFFIX="${1:?Usage: build-parsers.sh <platform-suffix>}"
OUTDIR="build/parsers"
mkdir -p "$OUTDIR"

declare -A GRAMMARS=(
  [rust]="tree-sitter/tree-sitter-rust@v0.24.0"
  [python]="tree-sitter/tree-sitter-python@v0.25.0"
  [javascript]="tree-sitter/tree-sitter-javascript@v0.25.0"
  [typescript]="tree-sitter-grammars/tree-sitter-typescript@v0.23.2:typescript/src"
  [tsx]="tree-sitter-grammars/tree-sitter-typescript@v0.23.2:tsx/src"
  [go]="tree-sitter/tree-sitter-go@v0.25.0"
  [java]="tree-sitter-grammars/tree-sitter-java@v0.23.5"
  [c]="tree-sitter/tree-sitter-c@v0.24.0"
  [cpp]="tree-sitter-grammars/tree-sitter-cpp@v0.23.4"
  [ruby]="tree-sitter-grammars/tree-sitter-ruby@v0.23.1"
  [bash]="tree-sitter-grammars/tree-sitter-bash@v0.25.0"
)

for lang in "${!GRAMMARS[@]}"; do
  spec="${GRAMMARS[$lang]}"
  repo_tag="${spec%%:*}"
  subdir="${spec#*:}"
  [ "$subdir" = "$spec" ] && subdir="src"
  repo="${repo_tag%%@*}"
  tag="${repo_tag##*@}"

  echo "=== Building $lang from $repo@$tag ($subdir) ==="
  tmpdir=$(mktemp -d)
  git clone --depth 1 --branch "$tag" "https://github.com/$repo.git" "$tmpdir"

  srcdir="$tmpdir/$subdir"
  outfile="$OUTDIR/tree-sitter-${lang}-${SUFFIX}.so"

  sources=("$srcdir/parser.c")
  [ -f "$srcdir/scanner.c" ] && sources+=("$srcdir/scanner.c")

  cc -shared -fPIC -O2 \
    -I "$srcdir" \
    "${sources[@]}" \
    -o "$outfile"

  chmod 755 "$outfile"
  rm -rf "$tmpdir"
  echo "  -> $outfile ($(du -h "$outfile" | cut -f1))"
done

echo "=== Done: $(ls "$OUTDIR"/*.so | wc -l) parsers built ==="
