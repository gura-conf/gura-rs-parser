import unittest
from typing import Dict
import gura
import os


class TestArraysGura(unittest.TestCase):
    file_dir: str
    expected: Dict

    def setUp(self):
        self.file_dir = os.path.dirname(os.path.abspath(__file__))

        self.maxDiff = 1024

        # All tests share the same content
        self.expected = {
            "colors": ["red", "yellow", "green"],
            "integers": [1, 2, 3],
            "integers_with_new_line": [1, 2, 3],
            "nested_arrays_of_ints": [[1, 2], [3, 4, 5]],
            "nested_mixed_array": [[1, 2], ["a", "b", "c"]],
            "numbers": [0.1, 0.2, 0.5, 1, 2, 5],
            "tango_singers": [{
                "user1": {
                    "name": "Carlos",
                    "surname": "Gardel",
                    "testing_nested": {
                        "nested_1": 1,
                        "nested_2": 2
                    },
                    "year_of_birth": 1890
                }
            }, {
                "user2": {
                    "name": "Aníbal",
                    "surname": "Troilo",
                    "year_of_birth": 1914
                }
            }],
            "mixed_with_object": [
                1,
                {'test': {'genaro': 'Camele'}},
                2,
                [4, 5, 6],
                3
            ],
            "separator": [
                {"a": 1, "b": 2},
                {"a": 1},
                {"b": 2}
            ]
        }

        self.expected_inside_object = {
            "model": {
                "columns": [
                    ["var1", "str"],
                    ["var2", "str"]
                ]
            }
        }

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

    def test_normal(self):
        """Tests all kind of arrays"""
        parsed_data = self.__get_file_parsed_data('normal.ura')
        self.assertDictEqual(parsed_data, self.expected)

    def test_with_comments(self):
        """Tests all kind of arrays with comments between elements"""
        parsed_data = self.__get_file_parsed_data('with_comments.ura')
        self.assertDictEqual(parsed_data, self.expected)

    def test_array_in_object(self):
        """Tests issue https://github.com/gura-conf/gura/issues/1"""
        parsed_data = self.__get_file_parsed_data('array_in_object.ura')
        self.assertDictEqual(parsed_data, self.expected_inside_object)
        parsed_data = self.__get_file_parsed_data('array_in_object_trailing_comma.ura')
        self.assertDictEqual(parsed_data, self.expected_inside_object)


if __name__ == '__main__':
    unittest.main()
