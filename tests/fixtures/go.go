// Package main is the entry point - should be FLAGGED
package main

//go:generate stringer -type=Direction -- should be ALLOWED
//go:build linux -- should be ALLOWED

import "fmt"

/* Block comment - should be FLAGGED */

// Greet prints a greeting message - should be FLAGGED
func Greet(name string) {
	// Regular inline comment - should be FLAGGED
	fmt.Printf("Hello, %s\n", name)
}

/*
Multi-line block comment
spanning several lines - should be FLAGGED
*/
func AnotherFunc() {}

// TODO: add error handling - should be FLAGGED

const s = "// this is not a comment - inside string"
const t = "/* also not a comment */"
