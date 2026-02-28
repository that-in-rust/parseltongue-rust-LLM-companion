#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${1:-.}"
OUT_DIR="${2:-.code-map}"
MAX_GRAPH_EDGES="${MAX_GRAPH_EDGES:-160}"

if ! command -v rg >/dev/null 2>&1; then
  echo "error: ripgrep (rg) is required" >&2
  exit 1
fi

ROOT_DIR="$(cd "$ROOT_DIR" && pwd)"
OUT_DIR="$(mkdir -p "$OUT_DIR" && cd "$OUT_DIR" && pwd)"

ALL_FILES_TXT="$OUT_DIR/all_files.txt"
CODE_FILES_TXT="$OUT_DIR/code_files.txt"
FILES_TSV="$OUT_DIR/files.tsv"
SYMBOLS_TSV="$OUT_DIR/symbols.tsv"
IMPORTS_TSV="$OUT_DIR/import_edges.tsv"
INTERNAL_TSV="$OUT_DIR/internal_file_edges.tsv"
EXTERNAL_TSV="$OUT_DIR/external_ref_edges.tsv"
MERMAID_MMD="$OUT_DIR/dependency_graph.mmd"
GRAPH_DOT="$OUT_DIR/dependency_graph.dot"
GRAPH_SVG="$OUT_DIR/dependency_graph.svg"
SUMMARY_MD="$OUT_DIR/summary.md"
TOOLS_TSV="$OUT_DIR/tooling.tsv"

if git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  git -C "$ROOT_DIR" ls-files > "$ALL_FILES_TXT"
else
  (
    cd "$ROOT_DIR"
    rg --files
  ) > "$ALL_FILES_TXT"
fi

rg -N '\.(rs|py|js|jsx|ts|tsx|go|java|c|cc|cpp|h|hpp)$' "$ALL_FILES_TXT" > "$CODE_FILES_TXT" || true

if [ ! -s "$CODE_FILES_TXT" ]; then
  echo "error: no supported source files found under $ROOT_DIR" >&2
  exit 1
fi

python3 - "$ROOT_DIR" "$CODE_FILES_TXT" "$FILES_TSV" <<'PY'
import os
import sys

root_dir, code_files, out_tsv = sys.argv[1:4]
with open(code_files, "r", encoding="utf-8") as fh, open(out_tsv, "w", encoding="utf-8") as out:
    out.write("path\tline_count\n")
    for rel in fh:
        rel = rel.strip()
        if not rel:
            continue
        path = os.path.join(root_dir, rel)
        try:
            with open(path, "r", encoding="utf-8", errors="replace") as fsrc:
                line_count = sum(1 for _ in fsrc)
        except OSError:
            line_count = 0
        out.write(f"{rel}\t{line_count}\n")
PY

CTAGS_AVAILABLE="no"
AST_GREP_AVAILABLE="no"
DOT_AVAILABLE="no"
if command -v ctags >/dev/null 2>&1 && ctags --output-format=json -f - /dev/null >/dev/null 2>&1; then
  CTAGS_AVAILABLE="yes"
fi
if command -v ast-grep >/dev/null 2>&1 || command -v sg >/dev/null 2>&1; then
  AST_GREP_AVAILABLE="yes"
fi
if command -v dot >/dev/null 2>&1; then
  DOT_AVAILABLE="yes"
fi

printf "tool\tavailable\nrg\tyes\nctags\t%s\nast-grep\t%s\ndot\t%s\n" "$CTAGS_AVAILABLE" "$AST_GREP_AVAILABLE" "$DOT_AVAILABLE" > "$TOOLS_TSV"

if [ "$CTAGS_AVAILABLE" = "yes" ]; then
  CTAGS_JSONL="$OUT_DIR/ctags.jsonl"
  (
    cd "$ROOT_DIR"
    ctags --output-format=json --fields=+neK --extras=+q -L "$CODE_FILES_TXT" -f -
  ) > "$CTAGS_JSONL" 2>/dev/null || true

  if [ -s "$CTAGS_JSONL" ]; then
    python3 - "$CTAGS_JSONL" "$SYMBOLS_TSV" <<'PY'
import json
import sys

ctags_jsonl, out_tsv = sys.argv[1:3]
with open(out_tsv, "w", encoding="utf-8") as out:
    out.write("symbol\tpath\tstart_line\tend_line\tkind\tscope\n")
    with open(ctags_jsonl, "r", encoding="utf-8", errors="replace") as fh:
        for raw in fh:
            raw = raw.strip()
            if not raw:
                continue
            try:
                obj = json.loads(raw)
            except json.JSONDecodeError:
                continue
            path = obj.get("path")
            name = obj.get("name")
            if not path or not name:
                continue
            start_line = obj.get("line", 0)
            end_line = obj.get("end", start_line)
            kind = obj.get("kind", "unknown")
            scope = obj.get("scope", "")
            out.write(f"{name}\t{path}\t{start_line}\t{end_line}\t{kind}\t{scope}\n")
PY
  fi
fi

if [ ! -s "$SYMBOLS_TSV" ]; then
  python3 - "$ROOT_DIR" "$CODE_FILES_TXT" "$SYMBOLS_TSV" <<'PY'
import os
import re
import sys

root_dir, code_files, out_tsv = sys.argv[1:4]

patterns = {
    ".rs": [
        re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"),
        re.compile(r"^\s*(?:pub\s+)?(?:struct|enum|trait)\s+([A-Za-z_][A-Za-z0-9_]*)"),
    ],
    ".py": [
        re.compile(r"^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("),
        re.compile(r"^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    ],
    ".js": [re.compile(r"^\s*(?:export\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("), re.compile(r"^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)\b")],
    ".jsx": [re.compile(r"^\s*(?:export\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("), re.compile(r"^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)\b")],
    ".ts": [re.compile(r"^\s*(?:export\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("), re.compile(r"^\s*(?:export\s+)?(?:class|interface|type)\s+([A-Za-z_][A-Za-z0-9_]*)\b")],
    ".tsx": [re.compile(r"^\s*(?:export\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("), re.compile(r"^\s*(?:export\s+)?(?:class|interface|type)\s+([A-Za-z_][A-Za-z0-9_]*)\b")],
    ".go": [re.compile(r"^\s*func\s+(?:\([^)]+\)\s*)?([A-Za-z_][A-Za-z0-9_]*)\s*\("), re.compile(r"^\s*type\s+([A-Za-z_][A-Za-z0-9_]*)\b")],
    ".java": [re.compile(r"^\s*(?:public\s+|private\s+|protected\s+)?(?:class|interface|enum)\s+([A-Za-z_][A-Za-z0-9_]*)\b"), re.compile(r"^\s*(?:public|private|protected)?\s*(?:static\s+)?[A-Za-z0-9_<>,\[\]]+\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")],
    ".c": [re.compile(r"^\s*[A-Za-z_][A-Za-z0-9_\s\*]+\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^;]*\)\s*\{"),],
    ".cc": [re.compile(r"^\s*[A-Za-z_][A-Za-z0-9_\s:<>,\*]+\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^;]*\)\s*\{"),],
    ".cpp": [re.compile(r"^\s*[A-Za-z_][A-Za-z0-9_\s:<>,\*]+\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^;]*\)\s*\{"),],
    ".h": [re.compile(r"^\s*[A-Za-z_][A-Za-z0-9_\s\*]+\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^;]*\)\s*;"),],
    ".hpp": [re.compile(r"^\s*[A-Za-z_][A-Za-z0-9_\s:<>,\*]+\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^;]*\)\s*;"),],
}

with open(code_files, "r", encoding="utf-8") as fh, open(out_tsv, "w", encoding="utf-8") as out:
    out.write("symbol\tpath\tstart_line\tend_line\tkind\tscope\n")
    for rel in fh:
        rel = rel.strip()
        if not rel:
            continue
        ext = os.path.splitext(rel)[1].lower()
        file_patterns = patterns.get(ext, [])
        if not file_patterns:
            continue
        abs_path = os.path.join(root_dir, rel)
        try:
            with open(abs_path, "r", encoding="utf-8", errors="replace") as src:
                for idx, line in enumerate(src, start=1):
                    for pattern in file_patterns:
                        m = pattern.search(line)
                        if m:
                            symbol = m.group(1)
                            out.write(f"{symbol}\t{rel}\t{idx}\t{idx}\tregex\t\n")
                            break
        except OSError:
            continue
PY
fi

python3 - "$ROOT_DIR" "$CODE_FILES_TXT" "$IMPORTS_TSV" "$INTERNAL_TSV" "$EXTERNAL_TSV" "$MERMAID_MMD" "$GRAPH_DOT" "$MAX_GRAPH_EDGES" <<'PY'
import os
import re
import sys
from collections import Counter

(
    root_dir,
    code_files,
    imports_tsv,
    internal_tsv,
    external_tsv,
    mermaid_mmd,
    graph_dot,
    max_graph_edges,
) = sys.argv[1:9]
max_graph_edges = int(max_graph_edges)

code_set = set()
with open(code_files, "r", encoding="utf-8") as fh:
    for line in fh:
        rel = line.strip()
        if rel:
            code_set.add(rel)

def resolve_local_target(src_rel: str, target: str):
    src_dir = os.path.dirname(src_rel)
    target = target.strip()

    # JS/TS relative imports
    if target.startswith("."):
        base = os.path.normpath(os.path.join(src_dir, target))
        candidates = [
            base,
            base + ".ts",
            base + ".tsx",
            base + ".js",
            base + ".jsx",
            base + ".py",
            base + ".go",
            base + ".rs",
            os.path.join(base, "index.ts"),
            os.path.join(base, "index.tsx"),
            os.path.join(base, "index.js"),
            os.path.join(base, "__init__.py"),
            os.path.join(base, "mod.rs"),
        ]
        for cand in candidates:
            cand = os.path.normpath(cand)
            if cand in code_set:
                return cand

    # Python module import path
    if re.match(r"^[A-Za-z_][A-Za-z0-9_\.]*$", target) and "." in target:
        mod = target.replace(".", "/")
        for cand in (mod + ".py", os.path.join(mod, "__init__.py")):
            cand = os.path.normpath(cand)
            if cand in code_set:
                return cand

    # Rust crate/module style
    if target.startswith("crate::"):
        body = target[len("crate::") :].replace("::", "/")
        for cand in (
            body + ".rs",
            os.path.join(body, "mod.rs"),
            os.path.join("src", body + ".rs"),
            os.path.join("src", body, "mod.rs"),
        ):
            cand = os.path.normpath(cand)
            if cand in code_set:
                return cand

    # Java package style
    if re.match(r"^[A-Za-z_][A-Za-z0-9_\.]*$", target) and "." in target:
        java_path = target.replace(".", "/") + ".java"
        java_path = os.path.normpath(java_path)
        if java_path in code_set:
            return java_path

    # C/C++ includes using relative header path
    if "/" in target or target.endswith((".h", ".hpp", ".hh")):
        rel_candidate = os.path.normpath(os.path.join(src_dir, target))
        if rel_candidate in code_set:
            return rel_candidate
        target_norm = os.path.normpath(target)
        if target_norm in code_set:
            return target_norm

    return None

patterns = {
    ".rs": [
        (re.compile(r"^\s*use\s+([^;]+);"), "import"),
        (re.compile(r"^\s*mod\s+([A-Za-z0-9_]+)\s*;"), "module"),
    ],
    ".py": [
        (re.compile(r"^\s*import\s+([A-Za-z0-9_\.,\s]+)"), "import"),
        (re.compile(r"^\s*from\s+([A-Za-z0-9_\.]+)\s+import\s+"), "import"),
    ],
    ".js": [
        (re.compile(r"^\s*import\s+.*?\s+from\s+[\"\']([^\"\']+)[\"\']"), "import"),
        (re.compile(r"^\s*import\s+[\"\']([^\"\']+)[\"\']"), "import"),
        (re.compile(r"require\(\s*[\"\']([^\"\']+)[\"\']\s*\)"), "require"),
    ],
    ".jsx": [
        (re.compile(r"^\s*import\s+.*?\s+from\s+[\"\']([^\"\']+)[\"\']"), "import"),
        (re.compile(r"^\s*import\s+[\"\']([^\"\']+)[\"\']"), "import"),
        (re.compile(r"require\(\s*[\"\']([^\"\']+)[\"\']\s*\)"), "require"),
    ],
    ".ts": [
        (re.compile(r"^\s*import\s+.*?\s+from\s+[\"\']([^\"\']+)[\"\']"), "import"),
        (re.compile(r"^\s*import\s+[\"\']([^\"\']+)[\"\']"), "import"),
        (re.compile(r"require\(\s*[\"\']([^\"\']+)[\"\']\s*\)"), "require"),
    ],
    ".tsx": [
        (re.compile(r"^\s*import\s+.*?\s+from\s+[\"\']([^\"\']+)[\"\']"), "import"),
        (re.compile(r"^\s*import\s+[\"\']([^\"\']+)[\"\']"), "import"),
        (re.compile(r"require\(\s*[\"\']([^\"\']+)[\"\']\s*\)"), "require"),
    ],
    ".go": [
        (re.compile(r"^\s*import\s+[\"\']([^\"\']+)[\"\']"), "import"),
    ],
    ".java": [
        (re.compile(r"^\s*import\s+([^;]+);"), "import"),
    ],
    ".c": [
        (re.compile(r"^\s*#include\s+[<\"]([^>\"]+)[>\"]"), "include"),
    ],
    ".cc": [
        (re.compile(r"^\s*#include\s+[<\"]([^>\"]+)[>\"]"), "include"),
    ],
    ".cpp": [
        (re.compile(r"^\s*#include\s+[<\"]([^>\"]+)[>\"]"), "include"),
    ],
    ".h": [
        (re.compile(r"^\s*#include\s+[<\"]([^>\"]+)[>\"]"), "include"),
    ],
    ".hpp": [
        (re.compile(r"^\s*#include\s+[<\"]([^>\"]+)[>\"]"), "include"),
    ],
}

rows = []
for rel in sorted(code_set):
    ext = os.path.splitext(rel)[1].lower()
    lang_patterns = patterns.get(ext, [])
    if not lang_patterns:
        continue

    abs_path = os.path.join(root_dir, rel)
    try:
        with open(abs_path, "r", encoding="utf-8", errors="replace") as fh:
            in_go_import_block = False
            for idx, line in enumerate(fh, start=1):
                if ext == ".go":
                    stripped = line.strip()
                    if stripped.startswith("import ("):
                        in_go_import_block = True
                    elif in_go_import_block and stripped == ")":
                        in_go_import_block = False
                    elif in_go_import_block:
                        m = re.search(r"[\"\']([^\"\']+)[\"\']", stripped)
                        if m:
                            rows.append((rel, idx, m.group(1), "import"))
                            continue

                for patt, edge_kind in lang_patterns:
                    m = patt.search(line)
                    if not m:
                        continue
                    target = m.group(1).strip()
                    if ext == ".py" and edge_kind == "import" and "," in target:
                        for part in target.split(","):
                            clean = part.strip().split()[0] if part.strip() else ""
                            if clean:
                                rows.append((rel, idx, clean, edge_kind))
                    elif target:
                        rows.append((rel, idx, target, edge_kind))
                    break
    except OSError:
        continue

with open(imports_tsv, "w", encoding="utf-8") as out:
    out.write("source_file\tsource_line\ttarget_ref\tedge_kind\n")
    for src, ln, tgt, kind in rows:
        out.write(f"{src}\t{ln}\t{tgt}\t{kind}\n")

internal_rows = []
external_rows = []
for src, ln, tgt, kind in rows:
    resolved = resolve_local_target(src, tgt)
    if resolved:
        internal_rows.append((src, ln, resolved, kind))
    else:
        external_rows.append((src, ln, tgt, kind))

with open(internal_tsv, "w", encoding="utf-8") as out:
    out.write("source_file\tsource_line\ttarget_file\tedge_kind\n")
    for src, ln, tgt, kind in internal_rows:
        out.write(f"{src}\t{ln}\t{tgt}\t{kind}\n")

with open(external_tsv, "w", encoding="utf-8") as out:
    out.write("source_file\tsource_line\ttarget_ref\tedge_kind\n")
    for src, ln, tgt, kind in external_rows:
        out.write(f"{src}\t{ln}\t{tgt}\t{kind}\n")

# Prepare graph edges ranked by frequency.
pair_counts = Counter((src, tgt) for src, _, tgt, _ in internal_rows)
if not pair_counts:
    pair_counts = Counter((src, tgt) for src, _, tgt, _ in external_rows)

top_pairs = pair_counts.most_common(max_graph_edges)

node_ids = {}
def node_id(label: str) -> str:
    if label not in node_ids:
        node_ids[label] = f"N{len(node_ids)}"
    return node_ids[label]

with open(mermaid_mmd, "w", encoding="utf-8") as out:
    out.write("graph TD\n")
    for src, tgt in top_pairs:
        src_id = node_id(src)
        tgt_id = node_id(tgt)
        out.write(f"  {src_id}[\"{src}\"] --> {tgt_id}[\"{tgt}\"]\n")

with open(graph_dot, "w", encoding="utf-8") as out:
    out.write("digraph G {\n")
    out.write('  rankdir="LR";\n')
    out.write('  node [shape="box", style="rounded", fontsize=10];\n')
    for src, tgt in top_pairs:
        out.write(f'  "{src}" -> "{tgt}";\n')
    out.write("}\n")
PY

if [ "$DOT_AVAILABLE" = "yes" ]; then
  dot -Tsvg "$GRAPH_DOT" -o "$GRAPH_SVG" || true
fi

python3 - "$FILES_TSV" "$SYMBOLS_TSV" "$IMPORTS_TSV" "$INTERNAL_TSV" "$EXTERNAL_TSV" "$TOOLS_TSV" "$SUMMARY_MD" "$MAX_GRAPH_EDGES" <<'PY'
import csv
import sys
from collections import Counter

(
    files_tsv,
    symbols_tsv,
    imports_tsv,
    internal_tsv,
    external_tsv,
    tools_tsv,
    summary_md,
    max_graph_edges,
) = sys.argv[1:9]

max_graph_edges = int(max_graph_edges)

def count_rows(path):
    with open(path, "r", encoding="utf-8") as fh:
        return max(0, sum(1 for _ in fh) - 1)

file_count = count_rows(files_tsv)
symbol_count = count_rows(symbols_tsv)
import_count = count_rows(imports_tsv)
internal_count = count_rows(internal_tsv)
external_count = count_rows(external_tsv)

tools = {}
with open(tools_tsv, "r", encoding="utf-8") as fh:
    reader = csv.DictReader(fh, delimiter="\t")
    for row in reader:
        tools[row["tool"]] = row["available"]

fan_out = Counter()
fan_in = Counter()

with open(internal_tsv, "r", encoding="utf-8") as fh:
    reader = csv.DictReader(fh, delimiter="\t")
    for row in reader:
        fan_out[row["source_file"]] += 1
        fan_in[row["target_file"]] += 1

if not fan_out:
    with open(external_tsv, "r", encoding="utf-8") as fh:
        reader = csv.DictReader(fh, delimiter="\t")
        for row in reader:
            fan_out[row["source_file"]] += 1

with open(summary_md, "w", encoding="utf-8") as out:
    out.write("# Rough Codebase Map Summary\n\n")
    out.write("## Counts\n")
    out.write(f"- Code files: {file_count}\n")
    out.write(f"- Symbols: {symbol_count}\n")
    out.write(f"- Import/include edges: {import_count}\n")
    out.write(f"- Internal file edges: {internal_count}\n")
    out.write(f"- External reference edges: {external_count}\n")
    out.write(f"- Graph edge cap used: {max_graph_edges}\n\n")

    out.write("## Tooling\n")
    for name in ("rg", "ctags", "ast-grep", "dot"):
        out.write(f"- {name}: {tools.get(name, 'no')}\n")
    out.write("\n")

    out.write("## Top Fan-Out Files\n")
    for file_name, cnt in fan_out.most_common(10):
        out.write(f"- {file_name}: {cnt}\n")
    if not fan_out:
        out.write("- none\n")
    out.write("\n")

    out.write("## Top Fan-In Files\n")
    for file_name, cnt in fan_in.most_common(10):
        out.write(f"- {file_name}: {cnt}\n")
    if not fan_in:
        out.write("- none\n")
    out.write("\n")

    out.write("## Pointer-First Retrieval Pattern\n")
    out.write("Use `symbols.tsv` and `internal_file_edges.tsv` first. Read code spans only when needed via `file:start:end`.\n")
PY

echo "Built map in: $OUT_DIR"
echo "- $SUMMARY_MD"
echo "- $SYMBOLS_TSV"
echo "- $IMPORTS_TSV"
echo "- $INTERNAL_TSV"
echo "- $MERMAID_MMD"
if [ -f "$GRAPH_SVG" ]; then
  echo "- $GRAPH_SVG"
fi
