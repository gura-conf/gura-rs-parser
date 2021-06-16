import tempfile
import unittest
from typing import Dict
from gura import DuplicatedImportError, DuplicatedKeyError, DuplicatedVariableError, ParseError
import gura
import os


class TestImportingGura(unittest.TestCase):
    file_dir: str
    expected: Dict

    def setUp(self):
        self.file_dir = os.path.dirname(os.path.abspath(__file__))

        # All tests share the same content
        self.expected = {
            "from_file_one": 1,
            "from_file_two": {
                "name": "Aníbal",
                "surname": "Troilo",
                "year_of_birth": 1914
            },
            "from_original_1": [1, 2, 5],
            "from_original_2": False,
            "from_file_three": True,
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
        """Tests importing from several files"""
        parsed_data = self.__get_file_parsed_data('normal.ura')
        self.assertDictEqual(parsed_data, self.expected)

    def test_with_variables(self):
        """Tests importing from several files with a variable in import sentences"""
        parsed_data = self.__get_file_parsed_data('with_variable.ura')
        self.assertDictEqual(parsed_data, self.expected)

    def test_not_found_error(self):
        """Tests errors importing a non existing file"""
        with self.assertRaises(FileNotFoundError):
            gura.loads('import "invalid_file.ura"')

    def test_duplicated_key_error(self):
        """Tests errors when redefines a key"""
        with self.assertRaises(DuplicatedKeyError):
            self.__get_file_parsed_data('duplicated_key.ura')

    def test_duplicated_variable_error(self):
        """Tests errors when redefines a variable"""
        with self.assertRaises(DuplicatedVariableError):
            self.__get_file_parsed_data('duplicated_variable.ura')

    def test_duplicated_imports(self):
        """Tests errors when imports more than once a file"""
        with self.assertRaises(DuplicatedImportError):
            self.__get_file_parsed_data('duplicated_imports_simple.ura')

    def test_with_absolute_paths(self):
        """Tests that absolute paths works as expected"""
        tmp = tempfile.NamedTemporaryFile()
        with open(tmp.name, 'w') as temp:
            temp.write('from_temp: true')
        parsed_data = gura.loads(f'import "{temp.name}"\n'
                                 f'from_original: false')
        tmp.close()
        self.assertDictEqual(parsed_data, {
            'from_temp': True,
            'from_original': False
        })

    def test_parse_error_1(self):
        """Tests errors invalid importing sentence (there are blanks before import)"""
        with self.assertRaises(ParseError):
            gura.loads('  import "another_file.ura"')

    def test_parse_error_2(self):
        """Tests errors invalid importing sentence (there are more than one whitespace between import and file name)"""
        with self.assertRaises(ParseError):
            gura.loads('import   "another_file.ura"')


if __name__ == '__main__':
    unittest.main()
