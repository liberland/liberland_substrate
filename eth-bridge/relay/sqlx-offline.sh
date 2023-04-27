#!/bin/bash

set -euo pipefail

tmp=$(mktemp)
export DATABASE_URL="sqlite://$tmp"
sqlx database create
sqlx migrate run
cargo sqlx prepare
rm $tmp
