
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

