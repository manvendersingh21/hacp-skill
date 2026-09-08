import unittest

from greet import greet


class GreetNormalNameTests(unittest.TestCase):
    def test_single_name(self):
        self.assertEqual(greet("Alice"), "Hello, Alice!")

    def test_another_single_name(self):
        self.assertEqual(greet("Bob"), "Hello, Bob!")

    def test_name_with_inner_space_preserved(self):
        self.assertEqual(greet("Ada Lovelace"), "Hello, Ada Lovelace!")


class GreetWhitespaceTests(unittest.TestCase):
    def test_leading_and_trailing_spaces_stripped(self):
        self.assertEqual(greet("  Alice "), "Hello, Alice!")

    def test_tabs_and_newlines_stripped(self):
        self.assertEqual(greet("\t Bob \n"), "Hello, Bob!")


class GreetInvalidNameTests(unittest.TestCase):
    def test_empty_name_raises_value_error(self):
        with self.assertRaises(ValueError):
            greet("")

    def test_space_only_name_raises_value_error(self):
        with self.assertRaises(ValueError):
            greet("   ")

    def test_tab_and_newline_only_name_raises_value_error(self):
        for name in ("\t", "\n", " \t \n "):
            with self.assertRaises(ValueError):
                greet(name)


if __name__ == "__main__":
    unittest.main()
