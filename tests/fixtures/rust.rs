// This is a regular comment - should be FLAGGED
/// This is a doc comment - should be FLAGGED
//! This is a module doc comment - should be FLAGGED
/* Block comment - should be FLAGGED */
/** Doc block comment - should be FLAGGED */
// eslint-disable-next-line -- should be ALLOWED (linter directive)
// noqa -- should be ALLOWED
// SPDX-License-Identifier: MIT -- should be ALLOWED
// Copyright 2024 Example Corp -- should be ALLOWED
// #region Main -- should be ALLOWED
// #endregion -- should be ALLOWED

fn main() {
    let s = "// this is not a comment - inside string";
    let r = r#"/* also not a comment */"#;
    let x = 42; // inline comment - should be FLAGGED
}

// TODO: fix this -- should be FLAGGED (not in allowlist)
