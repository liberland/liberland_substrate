#!/bin/bash

set -e
tmpfile="$(mktemp)"
subxt metadata --pallets LLMBridge,LLDBridge > "$tmpfile"
subxt codegen --file "$tmpfile" | rustfmt --emit=stdout > src/liberland_api.rs
