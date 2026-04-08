#!/usr/bin/env bash
set -euo pipefail

SUFFIX="${1:?Usage: build-parsers.sh <platform-suffix> [cc-flags]}"
CC_FLAGS="${2:-}"
OUTDIR="build/parsers"
mkdir -p "$OUTDIR"

GRAMMARS="
rust          tree-sitter/tree-sitter-rust          v0.24.2
python        tree-sitter/tree-sitter-python        v0.25.0
javascript    tree-sitter/tree-sitter-javascript    v0.25.0
typescript    tree-sitter/tree-sitter-typescript    v0.23.2   typescript/src
tsx           tree-sitter/tree-sitter-typescript    v0.23.2   tsx/src
go            tree-sitter/tree-sitter-go            v0.25.0
java          tree-sitter/tree-sitter-java          v0.23.5
c             tree-sitter/tree-sitter-c             v0.24.1
cpp           tree-sitter/tree-sitter-cpp           v0.23.4
ruby          tree-sitter/tree-sitter-ruby          v0.23.1
bash          tree-sitter/tree-sitter-bash          v0.25.1
kotlin        fwcd/tree-sitter-kotlin               0.3.8
swift         alex-pinkus/tree-sitter-swift         0.7.1
c-sharp       tree-sitter/tree-sitter-c-sharp       v0.23.1
scala         tree-sitter/tree-sitter-scala         v0.23.2
php           tree-sitter/tree-sitter-php           v0.23.11  php/src
lua           tree-sitter-grammars/tree-sitter-lua  v0.2.0
elixir        elixir-lang/tree-sitter-elixir        v0.3.4
haskell       tree-sitter/tree-sitter-haskell       v0.23.1
ocaml         tree-sitter/tree-sitter-ocaml         v0.23.2   grammars/ocaml/src
zig           tree-sitter-grammars/tree-sitter-zig  v1.1.2
dart          UserNobody14/tree-sitter-dart         master
r             r-lib/tree-sitter-r                   v1.1.0
toml          tree-sitter-grammars/tree-sitter-toml v0.7.0
yaml          tree-sitter-grammars/tree-sitter-yaml v0.7.0
html          tree-sitter/tree-sitter-html          v0.23.2
css           tree-sitter/tree-sitter-css           v0.23.2
sql           DerekStride/tree-sitter-sql           v0.3.6
hcl           tree-sitter-grammars/tree-sitter-hcl  v1.1.0
nix           nix-community/tree-sitter-nix         v0.3.0
clojure       sogaiu/tree-sitter-clojure            v0.0.13
erlang        WhatsApp/tree-sitter-erlang           0.15
objc          tree-sitter-grammars/tree-sitter-objc v3.0.2
"

echo "$GRAMMARS" | while read -r lang repo tag subdir; do
  [ -z "$lang" ] && continue
  subdir="${subdir:-src}"

  echo "=== Building $lang from $repo@$tag ($subdir) ==="
  tmpdir=$(mktemp -d)
  git clone --depth 1 --branch "$tag" "https://github.com/$repo.git" "$tmpdir"

  srcdir="$tmpdir/$subdir"
  outfile="$OUTDIR/tree-sitter-${lang}-${SUFFIX}.so"

  if [ ! -f "$srcdir/parser.c" ]; then
    echo "  parser.c not found, running tree-sitter generate..."
    (cd "$tmpdir" && npx tree-sitter-cli generate)
  fi

  objects=""
  # shellcheck disable=SC2086
  cc -c -fPIC -O2 $CC_FLAGS -I "$srcdir" "$srcdir/parser.c" -o "$srcdir/parser.o"
  objects="$srcdir/parser.o"

  if [ -f "$srcdir/scanner.c" ]; then
    # shellcheck disable=SC2086
    cc -c -fPIC -O2 $CC_FLAGS -I "$srcdir" "$srcdir/scanner.c" -o "$srcdir/scanner.o"
    objects="$objects $srcdir/scanner.o"
  fi

  if [ -f "$srcdir/scanner.cc" ]; then
    # shellcheck disable=SC2086
    c++ -c -fPIC -O2 $CC_FLAGS -I "$srcdir" "$srcdir/scanner.cc" -o "$srcdir/scanner_cc.o"
    objects="$objects $srcdir/scanner_cc.o"
  fi

  link_flags=""
  [ -f "$srcdir/scanner.cc" ] && link_flags="-lstdc++"

  # shellcheck disable=SC2086
  cc -shared $CC_FLAGS $objects -o "$outfile" $link_flags

  chmod 755 "$outfile"
  rm -rf "$tmpdir"
  echo "  -> $outfile ($(du -h "$outfile" | cut -f1))"
done

echo "=== Done: $(ls "$OUTDIR"/*.so 2>/dev/null | wc -l) parsers built ==="
