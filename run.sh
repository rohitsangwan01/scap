#!/bin/bash
# Wrapper script to run scap with proper Swift library paths
export DYLD_FALLBACK_LIBRARY_PATH=/usr/lib/swift:$DYLD_FALLBACK_LIBRARY_PATH
exec cargo run "$@"

