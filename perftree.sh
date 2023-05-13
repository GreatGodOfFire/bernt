#!/bin/bash
cargo run --release --features perftree -- "$@" 2>/dev/null
