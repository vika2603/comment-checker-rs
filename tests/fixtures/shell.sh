#!/bin/bash

# This is a regular comment - should be FLAGGED
# TODO: add error handling - should be FLAGGED

# Copyright 2024 Example Corp -- should be ALLOWED

greet() {
    local name="$1"
    # Another regular comment - should be FLAGGED
    s="# this is not a comment - inside string"
    echo "Hello, ${name}"
}

greet "world"

# License: MIT -- should be ALLOWED
