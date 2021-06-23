use std::collections::HashMap;

// Number chars
const BASIC_NUMBERS_CHARS: &str = "0-9";
const HEX_OCT_BIN: &str = "A-Fa-fxob";
const INF_AND_NAN: &str = "in"; // The rest of the chars are defined in hex_oct_bin
// IMPORTANT: '-' char must be last, otherwise it will be interpreted as a range
const ACCEPTABLE_NUMBER_CHARS: &str = BASIC_NUMBERS_CHARS + HEX_OCT_BIN + INF_AND_NAN + "Ee+._-";

// Acceptable chars for keys





/* Match expression method's result */

struct GuraParser {

}

impl GuraParser {

	

 

  

  



  







  

  /**
   * Matches with a list.
   *
   * @returns Matched list.
   */
  list (): MatchResult {
    const result = []

    this.maybeMatch([this.ws])
    this.keyword(['['])
    while (true) {
      // Discards useless lines between elements of array
      const uselessLine = this.maybeMatch([this.uselessLine])
      if (uselessLine !== null) {
        continue
      }

      let item: MatchResult | null = this.maybeMatch([this.anyType])
      if (item === null) {
        break
      }

      if (item.resultType === MatchResultType.EXPRESSION) {
        item = item.value[0]
      } else {
        item = item.value
      }

      result.push(item)

      this.maybeMatch([this.ws])
      this.maybeMatch([this.newLine])
      if (!this.maybeKeyword([','])) {
        break
      }
    }

    this.maybeMatch([this.ws])
    this.maybeMatch([this.newLine])
    this.keyword([']'])
    return { resultType: MatchResultType.LIST, value: result }
  }



  /**
   * Match any Gura expression.
   *
   * @throws DuplicatedKeyError if any of the defined key was declared more than once.
   * @returns Object with Gura string data.
   */
  expression (): MatchResult {
    const result = {}
    let indentationLevel = 0
    while (this.pos < this.len) {
      const item: MatchResult | null = this.maybeMatch([this.variable, this.pair, this.uselessLine])

      if (item === null) {
        break
      }

      if (item.resultType === MatchResultType.PAIR) {
        // It is a key / value pair
        const [key, value, indentation] = item.value
        if (result[key] !== undefined) {
          throw new DuplicatedKeyError(`The key '${key}' has been already defined`)
        }

        result[key] = value
        indentationLevel = indentation
      }

      if (this.maybeKeyword([']', ',']) !== null) {
        // Breaks if it is the end of a list
        this.removeLastIndentationLevel()
        this.pos -= 1
        break
      }
    }

    return Object.keys(result).length > 0
      ? { resultType: MatchResultType.EXPRESSION, value: [result, indentationLevel] }
      : null
  }

  /**
   * Removes, if exists, the last indentation level.
   */
  private removeLastIndentationLevel () {
    if (this.indentationLevels.length > 0) {
      this.indentationLevels.pop()
    }
  }

  /**
   * Matches with a key.A key is an unquoted string followed by a colon (:).
   *
   * @throws ParseError if key is not a valid string.
   * @returns Matched key.
   */
  key (): string {
    const key = this.match([this.unquotedString])

    if (typeof key !== 'string') {
      throw new ParseError(
        this.pos + 1,
        this.line,
        `Expected string but got "${this.text.substring(this.pos + 1)}"`
      )
    }

    this.keyword([':'])
    return key
  }

  /**
   * Matches with a key - value pair taking into consideration the indentation levels.
   *
   * @returns Matched key - value pair.null if the indentation level is lower than the last one(to indicate the ending of a parent object).
   */
  pair (): MatchResult | null {
    const posBeforePair = this.pos
    const currentIndentationLevel = this.maybeMatch([this.wsWithIndentation])

    const key = this.match([this.key])
    this.maybeMatch([this.ws])
    this.maybeMatch([this.newLine])

    // Check indentation
    const lastIndentationBlock = this.getLastIndentationLevel()

    // Check if indentation is divisible by 4
    if (currentIndentationLevel % 4 !== 0) {
      throw new InvalidIndentationError(`Indentation block (${currentIndentationLevel}) must be divisible by 4`)
    }

    if (lastIndentationBlock === null || currentIndentationLevel > lastIndentationBlock) {
      this.indentationLevels.push(currentIndentationLevel)
    } else {
      if (currentIndentationLevel < lastIndentationBlock) {
        this.removeLastIndentationLevel()

        // As the indentation was consumed, it is needed to return to line beginning to get the indentation level
        // again in the previous matching.Otherwise, the other match would get indentation level = 0
        this.pos = posBeforePair
        return null // This breaks the parent loop
      }
    }

    // If it === null then is an empty expression, and therefore invalid
    let result: MatchResult | null = this.match([this.anyType])
    if (result === null) {
      throw new ParseError(
        this.pos + 1,
        this.line,
        'Invalid pair'
      )
    }

    // Checks indentation against parent level
    if (result.resultType === MatchResultType.EXPRESSION) {
      const [objectValues, indentationLevel] = result.value
      if (indentationLevel === currentIndentationLevel) {
        throw new InvalidIndentationError(`Wrong level for parent with key ${key}`)
      } else {
        if (Math.abs(currentIndentationLevel - indentationLevel) !== 4) {
          throw new InvalidIndentationError('Difference between different indentation levels must be 4')
        }
      }

      result = objectValues
    } else {
      result = result.value
    }

    this.maybeMatch([this.newLine])

    return { resultType: MatchResultType.PAIR, value: [key, result, currentIndentationLevel] }
  }

  /**
   * Gets the last indentation level or null in case it does not exist.
   *
   * @returns Last indentation level or null if it does not exist.
   */
  private getLastIndentationLevel (): number | null {
    return this.indentationLevels.length === 0 ? null : this.indentationLevels[this.indentationLevels.length - 1]
  }

  

  /**
   * Parses an unquoted string.Useful for keys.
   *
   * @returns Parsed unquoted string.
   */
  unquotedString (): string {
    const chars = [this.char(KEY_ACCEPTABLE_CHARS)]

    while (true) {
      const char = this.maybeChar(KEY_ACCEPTABLE_CHARS)
      if (char === null) {
        break
      }

      chars.push(char)
    }

    return chars.join('').trimRight()
  }

  /**
   * Parses a string checking if it is a number and get its correct value.
   *
   * @throws ParseError if the extracted string is not a valid number.
   * @returns Returns an number or a float depending of type inference.
   */
  number (): MatchResult {
    let numberType: 'integer' | 'float' = 'integer'

    const chars = [this.char(ACCEPTABLE_NUMBER_CHARS)]

    while (true) {
      const char = this.maybeChar(ACCEPTABLE_NUMBER_CHARS)
      if (char === null) {
        break
      }

      if ('Ee.'.includes(char)) {
        numberType = 'float'
      }

      chars.push(char)
    }

    // Replaces underscores as JS does not support them
    const result = chars.join('').trimRight().replace(/_/g, '')

    // Checks hexadecimal and octal format
    const prefix = result.substring(0, 2)
    if (['0x', '0o', '0b'].includes(prefix)) {
      let base: number
      const withoutPrefix = result.substring(2)
      switch (prefix) {
        case '0x':
          base = 16
          break
        case '0o':
          base = 8
          break
        default:
          base = 2
          break
      }

      return { resultType: MatchResultType.PRIMITIVE, value: parseInt(withoutPrefix, base) }
    }

    // Checks inf or NaN
    const lastThreeChars = result.substring(result.length - 3)
    if (lastThreeChars === 'inf') {
      return { resultType: MatchResultType.PRIMITIVE, value: result[0] === '-' ? -Infinity : Infinity }
    } else {
      if (lastThreeChars === 'nan') {
        return { resultType: MatchResultType.PRIMITIVE, value: NaN }
      }
    }

    // NOTE: JS does not raise a parsing error when an invalid value is casted to number. That's why it's checked here
    const resultValue = numberType === 'integer' ? parseInt(result) : parseFloat(result)
    if (isNaN(resultValue)) {
      throw new ParseError(
        this.pos + 1,
        this.line,
        `'${result}' is not a valid number`
      )
    }

    return { resultType: MatchResultType.PRIMITIVE, value: resultValue }
  }

  

  /**
   * Matches with a simple / multiline literal string.
   *
   * @returns Matched string.
   */
  literalString (): MatchResult {
    const quote = this.keyword(["'''", "'"])

    const isMultiline = quote === "'''"

    // NOTE: A newline immediately following the opening delimiter will be trimmed.All other whitespace and
    // newline characters remain intact.
    if (isMultiline) {
      this.maybeChar('\n')
    }

    const chars = []

    while (true) {
      const closingQuote = this.maybeKeyword([quote])
      if (closingQuote !== null) {
        break
      }

      const char = this.char()
      chars.push(char)
    }

    return { resultType: MatchResultType.PRIMITIVE, value: chars.join('') }
  }

  /**
   * Takes a value, check its type and returns its correct value.
   *
   * @param indentationLevel - Current indentation level to compute indentation in string.
   * @param value - Value retrieved from dict to transform in string.
   * @returns String representation of the received value.
   */
  private getValueForString (indentationLevel: number, value: any): string {
    if (value === null) {
      return 'null'
    }

    const valueType = typeof value
    switch (valueType) {
      case 'string':
        return `"${value}"`
      case 'number':
        // Special cases
        if (value === Number.POSITIVE_INFINITY) {
          return 'inf'
        } else {
          if (value === Number.NEGATIVE_INFINITY) {
            return '-inf'
          } else {
            if (isNaN(value)) {
              return 'nan'
            }
          }
        }

        // Normal number
        return value.toString()
      case 'boolean':
        return value ? 'true' : 'false'
      case 'object':
        // Checks if it is an array as typeof [] === 'object'
        if (Array.isArray(value)) {
          const list = value as any[]
          const listValues = list.map((listElem) => this.getValueForString(indentationLevel, listElem))
          return '[' + listValues.join(', ') + ']'
        }

        return '\n' + this.dump(value, indentationLevel + 1)
    }

    return ''
  }

  /**
   * Generates a Gura string from a dictionary(aka.stringify).
   *
   * @param data - Object data to stringify.
   * @param indentationLevel - Current indentation level.
   * @returns String with the data in Gura format.
   */
  dump (data: Object, indentationLevel: number = 0): string {
    let result = ''
    Object.entries(data).forEach(([key, value]) => {
      const indentation = ' '.repeat(indentationLevel * 4)
      result += `${indentation}${key}: `
      result += this.getValueForString(indentationLevel, value)
      result += '\n'
    })

    return result
  }
}

/* ++++++++++++++++++++ PARSER ++++++++++++++++++++ */

/**
 * Parses a text in Gura format.
 *
 * @param text - Text to be parsed.
 * @throws ParseError if the syntax of text is invalid.
 * @returns Dict with all the parsed values.
 */
const parse = (text: string): Object => {
  return new GuraParser().parse(text)
}

/**
 * Generates a Gura string from a dictionary(aka.stringify).
 *
 * @param data - Object to stringify.
 * @returns String with the data in Gura format.
 */
const dump = (data: Object): string => {
  return new GuraParser().dump(data)
}

