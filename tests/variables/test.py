import unittest
import time
from typing import Dict
import gura
from gura import VariableNotDefinedError, DuplicatedVariableError, ParseError
import os


class TestVariablesGura(unittest.TestCase):
    file_dir: str
    expected: Dict

    def setUp(self):
        self.file_dir = os.path.dirname(os.path.abspath(__file__))

        # All tests share the same content
        self.expected = {
            "plain": 5,
            "in_array_middle": [1, 5, 3],
            "in_array_last": [1, 2, 5],
            "in_object": {
                "name": "Aníbal",
                "surname": "Troilo",
                "year_of_birth": 1914
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
        """Tests variables definition"""
        parsed_data = self.__get_file_parsed_data('normal.ura')
        self.assertDictEqual(parsed_data, self.expected)

    def test_with_error(self):
        """Tests errors in variables definition"""
        with self.assertRaises(VariableNotDefinedError):
            gura.loads(f'test: $false_var_{time.time_ns()}')

    def test_with_duplicated(self):
        """Tests errors when a variable is defined more than once"""
        with self.assertRaises(DuplicatedVariableError):
            gura.loads(f'$a_var: 14\n'
                       f'$a_var: 14')

    def test_env_var(self):
        """Tests using environment variables"""
        # Sets a new environment variable to check the correct value retrieval from Gura
        env_var_name = f'env_var_{time.time_ns()}'
        env_value = "using_env_var"
        os.environ[env_var_name] = env_value

        # Parses and tests
        parsed_data = gura.loads(f'test: ${env_var_name}')
        self.assertDictEqual(parsed_data, {"test": env_value})
        os.unsetenv(env_var_name)

    def test_invalid_variable(self):
        """Tests invalid variable value type"""
        with self.assertRaises(ParseError):
            gura.loads('$invalid: true')

    def test_invalid_variable_2(self):
        """Tests invalid variable value type"""
        with self.assertRaises(ParseError):
            gura.loads('$invalid: false')

    def test_invalid_variable_3(self):
        """Tests invalid variable value type"""
        with self.assertRaises(ParseError):
            gura.loads('$invalid: null')

    def test_invalid_variable_4(self):
        """Tests invalid variable value type"""
        with self.assertRaises(ParseError):
            gura.loads('$invalid: [ 1, 2, 3]')

    def test_invalid_variable_5(self):
        """Tests invalid variable value type"""
        with self.assertRaises(ParseError):
            self.__get_file_parsed_data('invalid_variable_with_object.ura')


if __name__ == '__main__':
    unittest.main()
