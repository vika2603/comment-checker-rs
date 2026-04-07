// This is a regular line comment - should be FLAGGED

/* This is a block comment - should be FLAGGED */

/**
 * This is a JSDoc comment - should be FLAGGED
 * @param {string} name - the name
 */
function greet(name) {
    // eslint-disable-next-line no-console -- should be ALLOWED
    console.log("Hello, " + name);
}

// eslint-disable -- should be ALLOWED

/* Multi-line block comment
   spanning multiple lines - should be FLAGGED */

const url = "https://example.com"; // inline comment - should be FLAGGED

const s = "// this is not a comment - inside string";
const t = '/* also not a comment */';

// TODO: implement this - should be FLAGGED
