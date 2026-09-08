import unittest
from greet import greet


class TestGreet(unittest.TestCase):

    def test_normal_names(self):
        self.assertEqual(greet("Alice"), "Hello, Alice!")
        self.assertEqual(greet("Bob Smith"), "Hello, Bob Smith!")

    def test_surrounding_whitespace(self):
        self.assertEqual(greet("   Alice  "), "Hello, Alice!")
        self.assertEqual(greet("\tBob\n"), "Hello, Bob!")

    def test_empty_and_whitespace_names(self):
        with self.assertRaises(ValueError):
            greet("")
        with self.assertRaises(ValueError):
            greet("   ")
        with self.assertRaises(ValueError):
            greet("\t\n")


if __name__ == "__main__":
    unittest.main()
