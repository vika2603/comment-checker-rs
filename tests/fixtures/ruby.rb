#!/usr/bin/env ruby

# This is a regular comment - should be FLAGGED
# TODO: implement this - should be FLAGGED

# Copyright 2024 Example Corp -- should be ALLOWED
# rubocop:disable Style/StringLiterals -- should be ALLOWED

def greet(name)
  # Another regular comment - should be FLAGGED
  s = "# this is not a comment - inside string"
  t = '# also not a comment'
  puts "Hello, #{name}"
end

# rubocop:enable Style/StringLiterals -- should be ALLOWED

=begin
This is a multi-line comment block in Ruby
It spans multiple lines - should be FLAGGED
=end
