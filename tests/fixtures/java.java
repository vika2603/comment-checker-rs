// This is a regular line comment - should be FLAGGED

/* This is a block comment - should be FLAGGED */

/**
 * This is a Javadoc comment - should be FLAGGED
 * @param name the name to greet
 * @return a greeting string
 */
public class JavaFixture {

    // noinspection unchecked -- should be ALLOWED

    /* Multi-line block comment
       spanning multiple lines - should be FLAGGED */
    public static String greet(String name) {
        // Copyright 2024 Example Corp -- should be ALLOWED
        String s = "// this is not a comment - inside string";
        String t = "/* also not a comment */";
        return "Hello, " + name;
    }

    // TODO: add validation - should be FLAGGED
}
