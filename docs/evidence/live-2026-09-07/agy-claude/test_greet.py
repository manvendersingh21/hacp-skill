import unittest

from greet import greet


class TestGreet(unittest.TestCase):
    def test_normal_name(self):
        self.assertEqual(greet("Alice"), "Hello, Alice!")

    def test_another_normal_name(self):
        self.assertEqual(greet("Bob"), "Hello, Bob!")

    def test_strips_surrounding_whitespace(self):
        self.assertEqual(greet("  Alice  "), "Hello, Alice!")

    def test_strips_tabs_and_newlines(self):
        self.assertEqual(greet("\t Carol \n"), "Hello, Carol!")

    def test_empty_string_raises_value_error(self):
        with self.assertRaises(ValueError):
            greet("")

    def test_whitespace_only_raises_value_error(self):
        with self.assertRaises(ValueError):
            greet("   ")

    def test_tabs_and_newlines_only_raises_value_error(self):
        with self.assertRaises(ValueError):
            greet("\t\n")


if __name__ == "__main__":
    unittest.main()
