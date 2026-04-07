// This is a regular line comment - should be FLAGGED

/* This is a block comment - should be FLAGGED */

/**
 * This is a TSDoc comment - should be FLAGGED
 * @param name - the name parameter
 */
function greet(name: string): string {
    // @ts-nocheck -- should be ALLOWED
    return `Hello, ${name}`;
}

// eslint-disable -- should be ALLOWED
// eslint-disable-next-line @typescript-eslint/no-explicit-any -- should be ALLOWED

interface User {
    /* block comment on interface - should be FLAGGED */
    name: string;
}

const s: string = "// not a comment - inside string";
const t: string = "/* also not a comment */";

// TODO: add validation - should be FLAGGED
