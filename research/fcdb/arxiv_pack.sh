#!/usr/bin/env bash
set -euo pipefail

# Enishi research/fcdb → arXiv packager
# Merkle-DAG note: Packaging is an ops-side artifact and does not mutate
# the runtime topology in story.jsonnet; it only bundles sources for submission.

proj_dir=$(cd "$(dirname "$0")" && pwd)
cd "$proj_dir"

main="main.tex"
timestamp=$(date +%Y%m%d)
out="arxiv_fcdb_${timestamp}.tar.gz"

if [[ ! -f "$main" ]]; then
  echo "error: $main not found in $proj_dir" >&2
  exit 1
fi

# Detect included graphics (paths without extensions are expanded later)
mapfile -t graphics < <(
  grep -Eo "\\\\includegraphics(\\[[^]]*\\])?\\{[^}]+\\}" "$main" \
    | sed -E 's/.*\{([^}]+)\}.*/\1/' \
    | sort -u || true
)

# Detect one-level \input and \include
mapfile -t includes < <(
  grep -Eo "\\\\(input|include)\\{[^}]+\\}" "$main" \
    | sed -E 's/.*\{([^}]+)\}.*/\1/' \
    | sort -u || true
)

# Detect bibliography directives
uses_bibtex=""
if grep -Eq "\\\\(bibliography|addbibresource)\\b" "$main"; then
  uses_bibtex=1
fi

files=("$main")

# Include README if present
[[ -f README.md ]] && files+=("README.md")

# Add includes, defaulting to .tex when extensionless
for inc in "${includes[@]:-}"; do
  [[ -z "$inc" ]] && continue
  if [[ -f "$inc" ]]; then
    files+=("$inc")
  elif [[ -f "${inc}.tex" ]]; then
    files+=("${inc}.tex")
  fi
done

# Add graphics with common extensions
for g in "${graphics[@]:-}"; do
  added=""
  for ext in pdf png jpg jpeg eps; do
    if [[ -f "${g}.${ext}" ]]; then
      files+=("${g}.${ext}")
      added=1
      break
    fi
  done
  if [[ -z "$added" && -f "$g" ]]; then
    files+=("$g")
  fi
done

# If using BibTeX, include all .bib in directory (conservative)
if [[ -n "${uses_bibtex}" ]]; then
  shopt -s nullglob
  for b in *.bib; do files+=("$b"); done
  shopt -u nullglob
fi

# Unique list
mapfile -t uniq_files < <(printf "%s\n" "${files[@]}" | awk 'NF' | sort -u)

# Filter out missing
missing=()
present=()
for f in "${uniq_files[@]}"; do
  if [[ -f "$f" ]]; then
    present+=("$f")
  else
    missing+=("$f")
  fi
done

if (( ${#missing[@]} > 0 )); then
  echo "warning: skipping missing files:" >&2
  printf "  - %s\n" "${missing[@]}" >&2
fi

# Create manifest
manifest="arxiv_manifest.txt"
{
  printf "# arXiv submission manifest\n"
  printf "# Generated: %s\n\n" "$(date -Iseconds)"
  printf "%s\n" "${present[@]}"
} >"$manifest"

# Pack
tar -czf "$out" "${present[@]}" "$manifest"
echo "created: $out"


