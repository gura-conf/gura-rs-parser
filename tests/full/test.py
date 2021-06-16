from typing import Dict
from gura import ParseError
import unittest
import gura
import math
import os


class TestFullGura(unittest.TestCase):
    file_dir: str
    parsed_data: Dict

    def setUp(self):
        self.file_dir = os.path.dirname(os.path.abspath(__file__))
        self.expected = {
            "a_string": "test string",
            "int1": +99,
            "int2": 42,
            "int3": 0,
            "int4": -17,
            "int5": 1000,
            "int6": 5349221,
            "int7": 5349221,
            "hex1": 3735928559,
            "hex2": 3735928559,
            "hex3": 3735928559,
            "oct1": 342391,
            "oct2": 493,
            "bin1": 214,
            "flt1": +1.0,
            "flt2": 3.1415,
            "flt3": -0.01,
            "flt4": 5e+22,
            "flt5": 1e06,
            "flt6": -2E-2,
            "flt7": 6.626e-34,
            "flt8": 224617.445991228,
            "sf1": math.inf,
            "sf2": math.inf,
            "sf3": -math.inf,
            "null": None,
            "bool1": True,
            "bool2": False,
            "1234": "1234",
            "services": {
                "nginx": {
                    "host": "127.0.0.1",
                    "port": 80
                },
                "apache": {
                    "virtual_host": "10.10.10.4",
                    "port": 81
                }
            },
            "integers": [1, 2, 3],
            "colors": ["red", "yellow", "green"],
            "nested_arrays_of_ints": [[1, 2], [3, 4, 5]],
            "nested_mixed_array": [[1, 2], ["a", "b", "c"]],
            "numbers": [0.1, 0.2, 0.5, 1, 2, 5],
            "tango_singers": [
                {
                    "user1": {
                        "name": "Carlos",
                        "surname": "Gardel",
                        "year_of_birth": 1890
                    }
                }, {
                    "user2": {
                        "name": "Aníbal",
                        "surname": "Troilo",
                        "year_of_birth": 1914
                    }
                }
            ],
            "integers2": [
                1, 2, 3
            ],
            "integers3": [
                1,
                2
            ],
            "my_server": {
                "host": "127.0.0.1",
                "port": 8080,
                "native_auth": True
            },
            "gura_is_cool": "Gura is cool"
        }
        self.maxDiff = 4096999

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

    def test_loads(self):
        """Tests all the common cases except NaNs"""
        parsed_data = self.__get_file_parsed_data('full.ura')
        self.assertDictEqual(parsed_data, self.expected)

    def test_loads_nan(self):
        """Tests NaNs cases as they are an exceptional case"""
        parsed_data = self.__get_file_parsed_data('nan.ura')
        for value in parsed_data.values():
            self.assertTrue(math.isnan(value))

    def test_dumps(self):
        """Tests dumps method"""
        parsed_data = self.__get_file_parsed_data('full.ura')
        string_data = gura.dumps(parsed_data)
        new_parsed_data = gura.loads(string_data)
        self.assertDictEqual(new_parsed_data, self.expected)

    def test_dumps_nan(self):
        """Tests dumps method with NaNs values"""
        parsed_data_nan = self.__get_file_parsed_data('nan.ura')
        string_data_nan = gura.dumps(parsed_data_nan)
        new_parsed_data_nan = gura.loads(string_data_nan)
        for value in new_parsed_data_nan.values():
            self.assertTrue(math.isnan(value))

    def test_empty(self):
        """Tests empty Gura documents"""
        parsed_data = gura.loads('')
        self.assertDictEqual(parsed_data, {})

    def test_empty_2(self):
        """Tests empty Gura documents, even when some data is defined"""
        parsed_data = gura.loads('$unused_var: 5')
        self.assertDictEqual(parsed_data, {})

    def test_invalid_key(self):
        """Tests invalid key"""
        with self.assertRaises(ParseError):
            gura.loads('with.dot: 5')

    def test_invalid_key_2(self):
        """Tests invalid key"""
        with self.assertRaises(ParseError):
            gura.loads('"with_quotes": 5')

    def test_invalid_key_3(self):
        """Tests invalid key"""
        with self.assertRaises(ParseError):
            gura.loads('with-dashes: 5')


if __name__ == '__main__':
    unittest.main()
