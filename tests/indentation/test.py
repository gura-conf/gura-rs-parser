import unittest
from typing import Dict
import gura
from gura import InvalidIndentationError
import os


class TestIndentationGura(unittest.TestCase):
    file_dir: str
    expected: Dict
    expected_object: Dict

    def setUp(self):
        self.file_dir = os.path.dirname(os.path.abspath(__file__))

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

    def test_wrong_indentation_char(self):
        """Tests raising an error when both whitespace and tabs are used at the time for indentation"""
        with self.assertRaises(InvalidIndentationError):
            self.__get_file_parsed_data('different_chars.ura')

    def test_indentation_not_divisible_by_4(self):
        """Tests raising an error when indentation is not divisible by 4"""
        with self.assertRaises(InvalidIndentationError):
            self.__get_file_parsed_data('not_divisible_by_4.ura')

    def test_indentation_non_consecutive_blocks(self):
        """Tests raising an error when two levels of an object are not separated by only 4 spaces of difference"""
        with self.assertRaises(InvalidIndentationError):
            self.__get_file_parsed_data('more_than_4_difference.ura')

    def test_indentation_with_tabs(self):
        """Tests raising an error when tab character is used as indentation"""
        with self.assertRaises(InvalidIndentationError):
            self.__get_file_parsed_data('with_tabs.ura')


if __name__ == '__main__':
    unittest.main()
