"""This is a module docstring - should be FLAGGED"""

# This is a regular comment - should be FLAGGED
# TODO: fix this - should be FLAGGED

# noqa -- should be ALLOWED
# noqa: E501 -- should be ALLOWED
# type: ignore -- should be ALLOWED
# Copyright 2024 Example Corp -- should be ALLOWED


def greet(name):
    """This is a function docstring - should be FLAGGED"""
    # Another regular comment - should be FLAGGED
    return f"Hello, {name}"


class MyClass:
    """This is a class docstring - should be FLAGGED"""

    def method(self):
        # pylint: disable=no-member -- should be ALLOWED
        x = "# this is not a comment - inside string"
        y = '# also not a comment'
        return x + y


# Multi-line block using consecutive lines:
# Line one of block - should be FLAGGED
# Line two of block - should be FLAGGED
