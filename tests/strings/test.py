import time
import unittest
from typing import Dict
import gura
from gura import VariableNotDefinedError
import os


class TestStringsGura(unittest.TestCase):
    file_dir: str
    expected_basic: Dict
    expected_multiline_basic: Dict
    expected_literal: Dict
    expected_multiline_literal: Dict

    def setUp(self):
        self.file_dir = os.path.dirname(os.path.abspath(__file__))

        escaped_value = '$name is cool'

        self.expected_basic = {
            "str": "I'm a string. \"You can quote me\". Na\bme\tJos\u00E9\nLocation\tSF.",
            "str_2": "I'm a string. \"You can quote me\". Na\bme\tJos\U000000E9\nLocation\tSF.",
            "with_var": "Gura is cool",
            "escaped_var": escaped_value,
            "with_env_var": "Gura is very cool"
        }

        multiline_value = "Roses are red\nViolets are blue"
        multiline_value_without_newline = "The quick brown fox jumps over the lazy dog."
        self.expected_multiline_basic = {
            "str": multiline_value,
            "str_2": multiline_value,
            "str_3": multiline_value,
            "with_var": multiline_value,
            "with_env_var": multiline_value,
            "str_with_backslash": multiline_value_without_newline,
            "str_with_backslash_2": multiline_value_without_newline,
            'str_4': 'Here are two quotation marks: "". Simple enough.',
            'str_5': 'Here are three quotation marks: """.',
            'str_6': 'Here are fifteen quotation marks: """"""""""""""".',
            "escaped_var": escaped_value,
        }

        self.expected_literal = {
            'quoted': 'John "Dog lover" Wick',
            'regex': '<\\i\\c*\\s*>',
            'winpath': 'C:\\Users\\nodejs\\templates',
            'winpath2': '\\\\ServerX\\admin$\\system32\\',
            'with_var': '$no_parsed variable!',
            "escaped_var": escaped_value
        }

        self.expected_multiline_literal = {
            'lines': 'The first newline is\ntrimmed in raw strings.\n   All other whitespace\n   is preserved.\n',
            'regex2': "I [dw]on't need \\d{2} apples",
            'with_var': '$no_parsed variable!',
            "escaped_var": escaped_value
        }
        self.maxDiff = 4096

    def __get_file_parsed_data(self, file_name) -> Dict:
        """
        Gets the content of a specific file parsed
        :param file_name: File name to get the content
        :return: Parsed data
        """
        full_test_path = os.path.join(self.file_dir, f'tests-files/{file_name}')
        with open(full_test_path, 'r') as file:
            content = file.read()
        return gura.loads(content)

    def test_basic_strings(self):
        """Tests basic strings"""
        env_var_name = 'env_var_value'
        os.environ[env_var_name] = 'very'
        parsed_data = self.__get_file_parsed_data('basic.ura')
        os.unsetenv(env_var_name)
        self.assertDictEqual(parsed_data, self.expected_basic)

    def test_multiline_basic_strings(self):
        """Tests multiline basic strings"""
        env_var_name = 'env_var_value'
        os.environ[env_var_name] = 'Roses'
        parsed_data = self.__get_file_parsed_data('multiline_basic.ura')
        os.unsetenv(env_var_name)
        self.assertDictEqual(parsed_data, self.expected_multiline_basic)

    def test_basic_strings_errors(self):
        """Tests errors in basic strings"""
        with self.assertRaises(VariableNotDefinedError):
            gura.loads(f'test: "$false_var_{time.time_ns()}"')

    def test_literal_strings(self):
        """Tests literal strings"""
        parsed_data = self.__get_file_parsed_data('literal.ura')
        self.assertDictEqual(parsed_data, self.expected_literal)

    def test_multiline_literal_strings(self):
        """Tests multiline literal strings"""
        parsed_data = self.__get_file_parsed_data('multiline_literal.ura')
        self.assertDictEqual(parsed_data, self.expected_multiline_literal)


if __name__ == '__main__':
    unittest.main()
