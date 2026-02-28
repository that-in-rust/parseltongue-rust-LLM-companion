#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "usage: read_code_span_pointer.sh <file:start:end | file start end> [context_lines]" >&2
  exit 1
fi

CONTEXT_LINES="2"
FILE_PATH=""
START_LINE=""
END_LINE=""

if [[ "$1" == *:*:* ]]; then
  CONTEXT_LINES="${2:-2}"
  FILE_PATH="${1%%:*}"
  REST="${1#*:}"
  START_LINE="${REST%%:*}"
  END_LINE="${REST##*:}"
elif [ "$#" -ge 3 ]; then
  FILE_PATH="$1"
  START_LINE="$2"
  END_LINE="$3"
  CONTEXT_LINES="${4:-2}"
else
  echo "error: provide either pointer format file:start:end or 3 args" >&2
  exit 1
fi

if [ ! -f "$FILE_PATH" ]; then
  echo "error: file not found: $FILE_PATH" >&2
  exit 1
fi

if ! [[ "$START_LINE" =~ ^[0-9]+$ ]] || ! [[ "$END_LINE" =~ ^[0-9]+$ ]]; then
  echo "error: start and end must be integers" >&2
  exit 1
fi

if [ "$START_LINE" -gt "$END_LINE" ]; then
  TMP="$START_LINE"
  START_LINE="$END_LINE"
  END_LINE="$TMP"
fi

FROM_LINE=$(( START_LINE - CONTEXT_LINES ))
if [ "$FROM_LINE" -lt 1 ]; then
  FROM_LINE=1
fi
TO_LINE=$(( END_LINE + CONTEXT_LINES ))

echo "# Span"
echo "pointer: ${FILE_PATH}:${START_LINE}:${END_LINE}"
echo "window: ${FROM_LINE}-${TO_LINE}"
echo
nl -ba "$FILE_PATH" | sed -n "${FROM_LINE},${TO_LINE}p"
