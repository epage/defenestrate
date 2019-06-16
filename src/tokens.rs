#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbol<'t> {
    token: &'t str,
    offset: usize,
}

impl<'t> Symbol<'t> {
    pub fn new(token: &'t str, offset: usize) -> Result<Self, failure::Error> {
        let mut itr = Self::parse(token.as_bytes());
        let mut item = itr
            .next()
            .ok_or_else(|| failure::format_err!("Invalid symbol (none found): {:?}", token))?;
        if item.offset != 0 {
            return Err(failure::format_err!(
                "Invalid symbol (padding found): {:?}",
                token
            ));
        }
        item.offset += offset;
        if itr.next().is_some() {
            return Err(failure::format_err!(
                "Invalid symbol (contains more than one): {:?}",
                token
            ));
        }
        Ok(item)
    }

    pub(crate) fn new_unchecked(token: &'t str, offset: usize) -> Self {
        Self { token, offset }
    }

    pub fn parse(content: &[u8]) -> impl Iterator<Item = Symbol<'_>> {
        lazy_static::lazy_static! {
            // Getting false positives for this lint
            #[allow(clippy::invalid_regex)]
            static ref SPLIT: regex::bytes::Regex = regex::bytes::Regex::new(r#"\b(\p{Alphabetic}|\d|_)+\b"#).unwrap();
        }
        SPLIT.find_iter(content).filter_map(|m| {
            let s = std::str::from_utf8(m.as_bytes()).ok();
            s.map(|s| Symbol::new_unchecked(s, m.start()))
        })
    }

    pub fn token(&self) -> &str {
        self.token
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn split(&self) -> impl Iterator<Item = Word<'_>> {
        split_symbol(self.token, self.offset)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Word<'t> {
    token: &'t str,
    offset: usize,
}

impl<'t> Word<'t> {
    pub fn new(token: &'t str, offset: usize) -> Result<Self, failure::Error> {
        Symbol::new(token, offset)?;
        let mut itr = split_symbol(token, 0);
        let mut item = itr
            .next()
            .ok_or_else(|| failure::format_err!("Invalid word (none found): {:?}", token))?;
        if item.offset != 0 {
            return Err(failure::format_err!(
                "Invalid word (padding found): {:?}",
                token
            ));
        }
        item.offset += offset;
        if itr.next().is_some() {
            return Err(failure::format_err!(
                "Invalid word (contains more than one): {:?}",
                token
            ));
        }
        Ok(item)
    }

    pub(crate) fn new_unchecked(token: &'t str, offset: usize) -> Self {
        Self { token, offset }
    }

    pub fn token(&self) -> &str {
        self.token
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

/// Tracks the current 'mode' of the transformation algorithm as it scans the input string.
///
/// The mode is a tri-state which tracks the case of the last cased character of the current
/// word. If there is no cased character (either lowercase or uppercase) since the previous
/// word boundary, than the mode is `Boundary`. If the last cased character is lowercase, then
/// the mode is `Lowercase`. Othertherwise, the mode is `Uppercase`.
#[derive(Clone, Copy, PartialEq, Debug)]
enum WordMode {
    /// There have been no lowercase or uppercase characters in the current word.
    Boundary,
    /// The previous cased character in the current word is lowercase.
    Lowercase,
    /// The previous cased character in the current word is uppercase.
    Uppercase,
    Number,
}

impl WordMode {
    fn classify(c: char) -> Self {
        if c.is_lowercase() {
            WordMode::Lowercase
        } else if c.is_uppercase() {
            WordMode::Uppercase
        } else if c.is_ascii_digit() {
            WordMode::Number
        } else {
            // This assumes all characters are either lower or upper case.
            WordMode::Boundary
        }
    }
}

fn split_symbol(symbol: &str, offset: usize) -> impl Iterator<Item = Word<'_>> {
    let mut result = vec![];

    let mut char_indices = symbol.char_indices().peekable();
    let mut start = 0;
    let mut start_mode = WordMode::Boundary;
    while let Some((i, c)) = char_indices.next() {
        let cur_mode = WordMode::classify(c);
        if cur_mode == WordMode::Boundary {
            if start == i {
                start += 1;
            }
            continue;
        }

        if let Some(&(next_i, next)) = char_indices.peek() {
            // The mode including the current character, assuming the current character does
            // not result in a word boundary.
            let next_mode = WordMode::classify(next);

            match (start_mode, cur_mode, next_mode) {
                // cur_mode is last of current word
                (_, _, WordMode::Boundary)
                | (_, WordMode::Lowercase, WordMode::Number)
                | (_, WordMode::Uppercase, WordMode::Number)
                | (_, WordMode::Number, WordMode::Lowercase)
                | (_, WordMode::Number, WordMode::Uppercase)
                | (_, WordMode::Lowercase, WordMode::Uppercase) => {
                    result.push(Word::new_unchecked(&symbol[start..next_i], start + offset));
                    start = next_i;
                    start_mode = WordMode::Boundary;
                }
                // cur_mode is start of next word
                (WordMode::Uppercase, WordMode::Uppercase, WordMode::Lowercase) => {
                    result.push(Word::new_unchecked(&symbol[start..i], start + offset));
                    start = i;
                    start_mode = WordMode::Boundary;
                }
                // No word boundary
                (_, _, _) => {
                    start_mode = cur_mode;
                }
            }
        } else {
            // Collect trailing characters as a word
            result.push(Word::new_unchecked(&symbol[start..], start + offset));
            break;
        }
    }

    result.into_iter()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tokenize_empty_is_empty() {
        let input = b"";
        let expected: Vec<Symbol> = vec![];
        let actual: Vec<_> = Symbol::parse(input).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn tokenize_word_is_word() {
        let input = b"word";
        let expected: Vec<Symbol> = vec![Symbol::new_unchecked("word", 0)];
        let actual: Vec<_> = Symbol::parse(input).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn tokenize_space_separated_words() {
        let input = b"A B";
        let expected: Vec<Symbol> =
            vec![Symbol::new_unchecked("A", 0), Symbol::new_unchecked("B", 2)];
        let actual: Vec<_> = Symbol::parse(input).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn tokenize_dot_separated_words() {
        let input = b"A.B";
        let expected: Vec<Symbol> =
            vec![Symbol::new_unchecked("A", 0), Symbol::new_unchecked("B", 2)];
        let actual: Vec<_> = Symbol::parse(input).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn tokenize_namespace_separated_words() {
        let input = b"A::B";
        let expected: Vec<Symbol> =
            vec![Symbol::new_unchecked("A", 0), Symbol::new_unchecked("B", 3)];
        let actual: Vec<_> = Symbol::parse(input).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn tokenize_underscore_doesnt_separate() {
        let input = b"A_B";
        let expected: Vec<Symbol> = vec![Symbol::new_unchecked("A_B", 0)];
        let actual: Vec<_> = Symbol::parse(input).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn split_symbol() {
        let cases = [
            ("lowercase", &["lowercase"] as &[&str]),
            ("Class", &["Class"]),
            ("MyClass", &["My", "Class"]),
            ("MyC", &["My", "C"]),
            ("HTML", &["HTML"]),
            ("PDFLoader", &["PDF", "Loader"]),
            ("AString", &["A", "String"]),
            ("SimpleXMLParser", &["Simple", "XML", "Parser"]),
            ("vimRPCPlugin", &["vim", "RPC", "Plugin"]),
            ("GL11Version", &["GL", "11", "Version"]),
            ("99Bottles", &["99", "Bottles"]),
            ("May5", &["May", "5"]),
            ("BFG9000", &["BFG", "9000"]),
        ];
        for (input, expected) in cases.iter() {
            let symbol = Symbol::new(input, 0).unwrap();
            let result: Vec<_> = symbol.split().map(|w| w.token).collect();
            assert_eq!(&result, expected);
        }
    }
}
