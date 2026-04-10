// This is a regular line comment - should be FLAGGED

/* This is a block comment - should be FLAGGED */

// Copyright 2024 Example Corp -- should be ALLOWED

#include <stdio.h>

/*
 * Multi-line block comment
 * spanning several lines - should be FLAGGED
 */
int greet(const char *name) {
    // Another regular comment - should be FLAGGED
    printf("Hello, %s\n", name);

    // label for string constants below - should be FLAGGED
    const char *s = "// this is not a comment - inside string";
    const char *t = "/* also not a comment */";

    return 0;
}

// TODO: handle null pointer - should be FLAGGED
