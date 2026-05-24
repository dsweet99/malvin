#!/bin/sh
set -eu

FILENAME=.kissconfig

replace_first() {
    key=$1
    value=$2
    tmp=$(mktemp "${TMPDIR:-/tmp}/kc.XXXXXX") || exit 1

    if sed -e "/^[[:space:]]*$key[[:space:]]*=.*/{" \
        -e 'x' \
        -e '/./{' \
        -e 'x' \
        -e 'b' \
        -e '}' \
        -e 'x' \
        -e "s/^[[:space:]]*$key[[:space:]]*=.*/$key = $value/" \
        -e 'h' \
        -e 's/.*/1/' \
        -e 'x' \
        -e '}' "$FILENAME" > "$tmp"; then
        mv "$tmp" "$FILENAME"
    else
        rm -f "$tmp"
        exit 1
    fi
}

replace_first statements_per_function 25
replace_first max_indentation 4
replace_first cycle_size 0
