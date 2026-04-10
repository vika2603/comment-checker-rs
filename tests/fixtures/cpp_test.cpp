// This is a regular line comment - should be FLAGGED

/* This is a block comment - should be FLAGGED */

/// This is a Doxygen doc comment - should be FLAGGED

// Copyright 2024 Example Corp -- should be ALLOWED

// #region Utility Functions -- should be ALLOWED
// #endregion -- should be ALLOWED

#include <string>
#include <iostream>

/*
 * Multi-line block comment
 * spanning several lines - should be FLAGGED
 */
std::string greet(const std::string& name) {
    // Another inline comment - should be FLAGGED
    std::string s = "// this is not a comment - inside string";
    std::string t = "/* also not a comment */";

    // Raw string with comment-like text - should be FLAGGED (the next line is not)
    std::string raw = R"(// not a comment inside raw string)";

    return "Hello, " + name;
}

// TODO: add exception handling - should be FLAGGED
