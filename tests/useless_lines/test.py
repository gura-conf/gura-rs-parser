import unittest
from typing import Dict
import gura
import os


class TestUselessLinesGura(unittest.TestCase):
    file_dir: str
    expected: Dict
    expected_object: Dict

    def setUp(self):
        self.file_dir = os.path.dirname(os.path.abspath(__file__))

        # All tests share the same content
        self.expected = {
            "a_string": "test string",
            "int1": +99,
            "int2": 42,
            "int3": 0,
            "int4": -17,
            "int5": 1000,
            "int6": 5349221,
            "int7": 5349221
        }

        self.expected_object = {
            'testing': {
                'test_2': 2,
                'test': {
                    'name': 'JWARE',
                    'surname': 'Solutions'
                }
            }
        }

        self.expected_object_complex = {
            'testing': {
                'test': {
                    'name': 'JWARE',
                    'surname': 'Solutions',
                    'skills': {
                        'good_testing': False,
                        'good_programming': False,
                        'good_english': False
                    }
                },
                'test_2': 2,
                'test_3': {
                    'key_1': True,
                    'key_2': False,
                    'key_3': 55.99
                }
            }
        }

    def __check_test_file(self, file_name: str):
        """
        Tests it against the expected data
        :param file_name: File name to get the content and test
        """
        parsed_data = self.__get_file_parsed_data(file_name)
        self.assertDictEqual(parsed_data, self.expected)

    def __check_test_file_object(self, file_name: str):
        """
        Tests it against the expected object data
        :param file_name: File name to get the content and test
        """
        parsed_data = self.__get_file_parsed_data(file_name)
        self.assertDictEqual(parsed_data, self.expected_object)

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

    def test_without(self):
        """Tests without comments or blank lines"""
        self.__check_test_file('without.ura')

    def test_on_top(self):
        """Tests with comments or blank lines on the top of the file"""
        self.__check_test_file('on_top.ura')

    def test_on_bottom(self):
        """Tests with comments or blank lines on the bottom of the file"""
        self.__check_test_file('on_bottom.ura')

    def test_on_both(self):
        """Tests with comments or blank lines on the top and bottom of the file"""
        self.__check_test_file('on_both.ura')

    def test_in_the_middle(self):
        """Tests with comments or blank lines in the middle of valid sentences"""
        self.__check_test_file('in_the_middle.ura')

    def test_without_object(self):
        """Tests without comments or blank lines in the middle of valid object"""
        self.__check_test_file_object('without_object.ura')

    def test_in_the_middle_object(self):
        """Tests with comments or blank lines in the middle of valid object"""
        self.__check_test_file_object('in_the_middle_object.ura')

    def test_in_the_middle_object_complex(self):
        """Tests with comments or blank lines in the middle of valid complex object"""
        parsed_data = self.__get_file_parsed_data('in_the_middle_object_complex.ura')
        self.assertDictEqual(parsed_data, self.expected_object_complex)


if __name__ == '__main__':
    unittest.main()
