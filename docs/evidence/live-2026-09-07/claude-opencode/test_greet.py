import unittest

from greet import greet


class GreetTests(unittest.TestCase):
    def test_normal_name(self):
        self.assertEqual(greet("Alice"), "Hello, Alice!")

    def test_another_normal_name(self):
        self.assertEqual(greet("Bob"), "Hello, Bob!")

    def test_internal_spaces_preserved(self):
        self.assertEqual(greet("Mary Jane"), "Hello, Mary Jane!")

    def test_surrounding_whitespace_stripped(self):
        self.assertEqual(greet("  Alice  "), "Hello, Alice!")

    def test_tabs_and_newlines_stripped(self):
        self.assertEqual(greet("\t\n Carol \t\n"), "Hello, Carol!")

    def test_empty_name_raises_value_error(self):
        with self.assertRaises(ValueError):
            greet("")

    def test_whitespace_only_name_raises_value_error(self):
        with self.assertRaises(ValueError):
            greet("   ")


if __name__ == "__main__":
    unittest.main()
