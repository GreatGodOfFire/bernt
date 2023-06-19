#!/bin/bash
cargo run --release --bin bernt-movegen --features perftree -- "$@" 2>/dev/null

