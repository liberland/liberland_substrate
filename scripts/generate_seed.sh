#!/bin/bash

bin=${LIBERLAND_NODE:-'cargo run -q --release --'}
$bin key generate | grep 'Secret phrase' | tr -s ' ' | cut -d' ' -f 3-
