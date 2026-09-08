"""Tests for the public greeting interface."""

import unittest

from greet import greet


class GreetTests(unittest.TestCase):
    def test_normal_names(self):
        cases = [
            ("Alice", "Hello, Alice!"),
            ("bob", "Hello, bob!"),
            ("Mary  Jane", "Hello, Mary  Jane!"),
        ]
        for name, expected in cases:
            with self.subTest(name=name):
                self.assertEqual(greet(name), expected)

    def test_surrounding_whitespace_is_stripped(self):
        cases = [
            ("  Alice", "Hello, Alice!"),
            ("Bob  ", "Hello, Bob!"),
            (" \t\nMary  Jane\r\n ", "Hello, Mary  Jane!"),
        ]
        for name, expected in cases:
            with self.subTest(name=name):
                self.assertEqual(greet(name), expected)

    def test_empty_name_raises_value_error(self):
        with self.assertRaises(ValueError):
            greet("")

    def test_whitespace_only_names_raise_value_error(self):
        for name in (" ", "   ", "\t", "\n", " \t\r\n "):
            with self.subTest(name=name):
                with self.assertRaises(ValueError):
                    greet(name)


if __name__ == "__main__":
    unittest.main()
