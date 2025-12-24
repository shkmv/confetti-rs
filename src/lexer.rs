use super::ConfError;
use std::ops::Range;
use unicode_general_category::{get_general_category, GeneralCategory};

/// Represents a token in the configuration language.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    /// End of file.
    Eof,
    /// A comment.
    Comment,
    /// Whitespace.
    Whitespace,
    /// A newline.
    Newline,
    /// An argument.
    Argument,
    /// A continuation.
    Continuation,
    /// A semicolon.
    Semicolon,
    /// A left curly brace.
    LeftCurlyBrace,
    /// A right curly brace.
    RightCurlyBrace,
}

/// Represents a token in the configuration language.
#[derive(Debug, Clone)]
pub struct Token {
    /// The type of the token.
    pub token_type: TokenType,
    /// The span of the token in the source text.
    pub span: Range<usize>,
    /// Whether the token is quoted.
    pub is_quoted: bool,
    /// Whether the token is triple-quoted.
    pub is_triple_quoted: bool,
    /// Whether the token is an expression.
    pub is_expression: bool,
}

/// A lexer for the configuration language.
pub struct Lexer<'a> {
    /// The input string.
    input: &'a str,
    /// The current position in the input string.
    position: usize,
    /// The options for the lexer.
    options: super::ConfOptions,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer.
    pub fn new(input: &'a str, options: super::ConfOptions) -> Self {
        Self {
            input,
            position: 0,
            options,
        }
    }

    /// Returns the input string.
    pub fn input(&self) -> &'a str {
        self.input
    }

    /// Returns the next token in the input string.
    pub fn next_token(&mut self) -> Result<Token, ConfError> {
        // Check for forbidden characters
        if let Some(c) = self.current_char() {
            if self.is_forbidden_char(c) {
                return Err(ConfError::LexerError {
                    position: self.position,
                    message: format!("Forbidden character: U+{:04X}", c as u32),
                });
            }
        }

        // Skip whitespace
        while self.position < self.input.len() && self.is_whitespace() && !self.is_newline() {
            self.advance();
        }

        // Check for end of input
        if self.position >= self.input.len() {
            return Ok(Token {
                token_type: TokenType::Eof,
                span: self.position..self.position,
                is_quoted: false,
                is_triple_quoted: false,
                is_expression: false,
            });
        }

        // Process comments
        if self.is_comment() {
            let start = self.position;
            self.scan_comment()?;
            return Ok(Token {
                token_type: TokenType::Comment,
                span: start..self.position,
                is_quoted: false,
                is_triple_quoted: false,
                is_expression: false,
            });
        }

        // Determine the token type based on the current character
        let start = self.position;
        let (token_type, is_quoted, is_triple_quoted, is_expression) = match self.current_char() {
            Some(c) if self.is_line_terminator(c) => {
                self.advance();
                // Handle CRLF as a single newline
                if c == '\r' && self.current_char() == Some('\n') {
                    self.advance();
                }
                (TokenType::Newline, false, false, false)
            }
            Some(';') => {
                self.advance();
                (TokenType::Semicolon, false, false, false)
            }
            Some('{') => {
                self.advance();
                (TokenType::LeftCurlyBrace, false, false, false)
            }
            Some('}') => {
                self.advance();
                (TokenType::RightCurlyBrace, false, false, false)
            }
            Some('\\') => {
                self.advance();
                // Check if this is a line continuation
                if self
                    .current_char()
                    .is_some_and(|c| self.is_line_terminator(c))
                {
                    let continuation_start = start;
                    // Skip the newline
                    self.advance();
                    // Handle CRLF as a single newline
                    if self.input.as_bytes().get(self.position - 1) == Some(&b'\r')
                        && self.current_char() == Some('\n')
                    {
                        self.advance();
                    }

                    // Skip any whitespace after the line continuation
                    while self.current_char().is_some_and(|_| self.is_whitespace()) {
                        self.advance();
                    }

                    // Return the continuation token
                    return Ok(Token {
                        token_type: TokenType::Continuation,
                        span: continuation_start..continuation_start + 1, // Только обратный слеш
                        is_quoted: false,
                        is_triple_quoted: false,
                        is_expression: false,
                    });
                } else {
                    // This is a backslash that's part of an argument
                    self.position = start; // Rewind
                    let is_expression = self.scan_argument()?;
                    (TokenType::Argument, false, false, is_expression)
                }
            }
            Some('"') => {
                let (is_triple_quoted, is_expression) = self.scan_quoted_argument()?;
                (TokenType::Argument, true, is_triple_quoted, is_expression)
            }
            _ => {
                let is_expression = self.scan_argument()?;
                (TokenType::Argument, false, false, is_expression)
            }
        };

        Ok(Token {
            token_type,
            span: start..self.position,
            is_quoted,
            is_triple_quoted,
            is_expression,
        })
    }

    /// Returns the current character in the input string.
    fn current_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            self.input[self.position..].chars().next()
        } else {
            None
        }
    }

    /// Returns the next character in the input string.
    fn next_char(&self) -> Option<char> {
        if let Some(c) = self.current_char() {
            let next_pos = self.position + c.len_utf8();
            if next_pos < self.input.len() {
                self.input[next_pos..].chars().next()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Advances the position by one character.
    fn advance(&mut self) {
        if let Some(c) = self.current_char() {
            self.position += c.len_utf8();
        }
    }

    /// Returns whether the current character is a whitespace character.
    fn is_whitespace(&self) -> bool {
        self.current_char()
            .is_some_and(|c| c.is_whitespace() && !self.is_line_terminator(c))
    }

    /// Returns whether the character is a line terminator.
    fn is_line_terminator(&self, c: char) -> bool {
        // According to the spec, these are the line terminators
        matches!(
            c,
            '\u{000A}' | // LF
            '\u{000B}' | // VT
            '\u{000C}' | // FF
            '\u{000D}' | // CR
            '\u{0085}' | // NEL
            '\u{2028}' | // LS
            '\u{2029}' // PS
        )
    }

    /// Returns whether the current character is a newline character.
    fn is_newline(&self) -> bool {
        self.current_char()
            .is_some_and(|c| self.is_line_terminator(c))
    }

    /// Returns whether the character is a forbidden character.
    ///
    /// Per the Confetti specification, forbidden characters are Unicode scalar values
    /// with general category Control, Surrogate, and Unassigned, excluding characters
    /// with the Whitespace property.
    fn is_forbidden_char(&self, c: char) -> bool {
        // Use unicode-general-category for proper Unicode category detection
        let category = get_general_category(c);

        // Check for forbidden Unicode categories (Control, Surrogate, Unassigned)
        // Note: Rust's char type cannot represent surrogates, so we only check Control and Unassigned
        let is_forbidden_category = matches!(
            category,
            GeneralCategory::Control | GeneralCategory::Unassigned
        ) && !c.is_whitespace();

        // Check for bidirectional formatting characters if forbidden
        let is_bidi = if self.options.forbid_bidi_characters {
            // Unicode bidirectional formatting characters
            matches!(
                c,
                '\u{061C}' | // ARABIC LETTER MARK
                '\u{200E}' | // LEFT-TO-RIGHT MARK
                '\u{200F}' | // RIGHT-TO-LEFT MARK
                '\u{2066}' | // LEFT-TO-RIGHT ISOLATE
                '\u{2067}' | // RIGHT-TO-LEFT ISOLATE
                '\u{2068}' | // FIRST STRONG ISOLATE
                '\u{2069}' | // POP DIRECTIONAL ISOLATE
                '\u{202A}' | // LEFT-TO-RIGHT EMBEDDING
                '\u{202B}' | // RIGHT-TO-LEFT EMBEDDING
                '\u{202C}' | // POP DIRECTIONAL FORMATTING
                '\u{202D}' | // LEFT-TO-RIGHT OVERRIDE
                '\u{202E}' // RIGHT-TO-LEFT OVERRIDE
            )
        } else {
            false
        };

        is_forbidden_category || is_bidi
    }

    /// Returns whether the current character is a comment character.
    fn is_comment(&self) -> bool {
        self.current_char().is_some_and(|c| {
            c == '#'
                || (self.options.allow_c_style_comments
                    && c == '/'
                    && (self.next_char() == Some('*') || self.next_char() == Some('/')))
        })
    }

    /// Scans a comment.
    fn scan_comment(&mut self) -> Result<(), ConfError> {
        let start = self.position;
        match self.current_char() {
            Some('#') => {
                // Single-line comment with #
                self.advance();
                while let Some(c) = self.current_char() {
                    if self.is_line_terminator(c) {
                        break;
                    }
                    if self.is_forbidden_char(c) {
                        return Err(ConfError::LexerError {
                            position: self.position,
                            message: format!("Forbidden character in comment: U+{:04X}", c as u32),
                        });
                    }
                    self.advance();
                }
            }
            Some('/') if self.next_char() == Some('/') && self.options.allow_c_style_comments => {
                // C-style single-line comment with //
                self.advance(); // Skip first '/'
                self.advance(); // Skip second '/'
                while let Some(c) = self.current_char() {
                    if self.is_line_terminator(c) {
                        break;
                    }
                    if self.is_forbidden_char(c) {
                        return Err(ConfError::LexerError {
                            position: self.position,
                            message: format!("Forbidden character in comment: U+{:04X}", c as u32),
                        });
                    }
                    self.advance();
                }
            }
            Some('/') if self.next_char() == Some('*') && self.options.allow_c_style_comments => {
                // Multi-line comment with /* */
                self.advance(); // Skip '/'
                self.advance(); // Skip '*'
                let mut found_end = false;
                while let Some(c) = self.current_char() {
                    if self.is_forbidden_char(c) {
                        return Err(ConfError::LexerError {
                            position: self.position,
                            message: format!("Forbidden character in comment: U+{:04X}", c as u32),
                        });
                    }
                    if c == '*' && self.next_char() == Some('/') {
                        self.advance(); // Skip '*'
                        self.advance(); // Skip '/'
                        found_end = true;
                        break;
                    }
                    self.advance();
                }
                if !found_end {
                    return Err(ConfError::LexerError {
                        position: start,
                        message: "Unterminated multi-line comment".to_string(),
                    });
                }
            }
            _ => {
                return Err(ConfError::LexerError {
                    position: start,
                    message: "Expected comment".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Scans a quoted argument.
    fn scan_quoted_argument(&mut self) -> Result<(bool, bool), ConfError> {
        let start = self.position;
        self.advance(); // Skip opening quote

        // Check for triple quote
        let is_triple_quoted = self.current_char() == Some('"') && self.next_char() == Some('"');
        if is_triple_quoted {
            self.advance(); // Skip second quote
            self.advance(); // Skip third quote
        }

        let mut found_end = false;
        while let Some(c) = self.current_char() {
            if self.is_forbidden_char(c) && !(is_triple_quoted && self.is_line_terminator(c)) {
                return Err(ConfError::LexerError {
                    position: self.position,
                    message: format!("Forbidden character in quoted argument: U+{:04X}", c as u32),
                });
            }

            if c == '\\' {
                // Handle escape sequence
                self.advance(); // Skip backslash
                if let Some(escaped) = self.current_char() {
                    // In quoted arguments, we allow escaping any character
                    // Line continuations are handled specially
                    if is_triple_quoted && self.is_line_terminator(escaped) {
                        // Line continuation in triple-quoted string
                        self.advance(); // Skip the line terminator
                                        // Handle CRLF as a single newline
                        if escaped == '\r' && self.current_char() == Some('\n') {
                            self.advance();
                        }
                    } else {
                        self.advance(); // Skip escaped character
                    }
                } else {
                    return Err(ConfError::LexerError {
                        position: self.position,
                        message: "Unterminated escape sequence".to_string(),
                    });
                }
            } else if c == '"' {
                if is_triple_quoted {
                    // Check for triple quote end
                    self.advance(); // Skip first quote
                    if self.current_char() == Some('"') {
                        self.advance(); // Skip second quote
                        if self.current_char() == Some('"') {
                            self.advance(); // Skip third quote
                            found_end = true;
                            break;
                        }
                    }
                    // Not a triple quote end, rewind position (using saturating_sub for safety)
                    self.position = self.position.saturating_sub(1);
                } else {
                    self.advance(); // Skip closing quote
                    found_end = true;
                    break;
                }
            } else {
                // In triple-quoted strings, we allow line terminators
                if !is_triple_quoted && self.is_line_terminator(c) {
                    return Err(ConfError::LexerError {
                        position: self.position,
                        message: "Newline in quoted string".to_string(),
                    });
                }
                self.advance();
            }
        }

        if !found_end {
            return Err(ConfError::LexerError {
                position: start,
                message: if is_triple_quoted {
                    "Unterminated triple-quoted string".to_string()
                } else {
                    "Unterminated quoted string".to_string()
                },
            });
        }

        // Check if this is an expression argument
        let is_expression = if self.options.allow_expression_arguments {
            self.current_char() == Some('(')
        } else {
            false
        };

        Ok((is_triple_quoted, is_expression))
    }

    /// Scans an argument.
    fn scan_argument(&mut self) -> Result<bool, ConfError> {
        let start = self.position;
        while let Some(c) = self.current_char() {
            // Arguments are terminated by whitespace, reserved punctuators, or EOF
            if c.is_whitespace()
                || c == ';'
                || c == '{'
                || c == '}'
                || c == '('
                || c == '"'
                || c == '#'
            {
                break;
            }

            if self.is_forbidden_char(c) {
                return Err(ConfError::LexerError {
                    position: self.position,
                    message: format!("Forbidden character in argument: U+{:04X}", c as u32),
                });
            }

            if c == '\\' {
                // Handle escape sequence
                self.advance(); // Skip backslash
                if let Some(escaped) = self.current_char() {
                    if self.is_line_terminator(escaped) {
                        // Line continuation
                        self.advance(); // Skip the line terminator
                                        // Handle CRLF as a single newline
                        if escaped == '\r' && self.current_char() == Some('\n') {
                            self.advance();
                        }
                        // Skip any whitespace after the line continuation
                        while self.current_char().is_some_and(|_| self.is_whitespace()) {
                            self.advance();
                        }
                    } else {
                        self.advance(); // Skip escaped character
                    }
                } else {
                    return Err(ConfError::LexerError {
                        position: self.position,
                        message: "Unterminated escape sequence".to_string(),
                    });
                }
            } else {
                self.advance();
            }
        }

        // If we didn't advance at all, this is an error
        if self.position == start {
            return Err(ConfError::LexerError {
                position: start,
                message: "Expected argument".to_string(),
            });
        }

        // Check if this is an expression argument
        let is_expression = if self.options.allow_expression_arguments {
            self.current_char() == Some('(')
        } else {
            false
        };

        Ok(is_expression)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_new() {
        let input = "test";
        let options = super::super::ConfOptions::default();
        let lexer = Lexer::new(input, options);
        assert_eq!(lexer.input, input);
        assert_eq!(lexer.position, 0);
    }

    #[test]
    fn test_lexer_current_char() {
        let input = "test";
        let options = super::super::ConfOptions::default();
        let lexer = Lexer::new(input, options);
        assert_eq!(lexer.current_char(), Some('t'));
    }

    #[test]
    fn test_lexer_next_char() {
        let input = "test";
        let options = super::super::ConfOptions::default();
        let lexer = Lexer::new(input, options);
        assert_eq!(lexer.next_char(), Some('e'));
    }

    #[test]
    fn test_lexer_advance() {
        let input = "test";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        lexer.advance();
        assert_eq!(lexer.position, 1);
    }

    #[test]
    fn test_lexer_is_whitespace() {
        let input = " ";
        let options = super::super::ConfOptions::default();
        let lexer = Lexer::new(input, options);
        assert!(lexer.is_whitespace());
    }

    #[test]
    fn test_lexer_is_newline() {
        let input = "\n";
        let options = super::super::ConfOptions::default();
        let lexer = Lexer::new(input, options);
        assert!(lexer.is_newline());
    }

    #[test]
    fn test_lexer_is_comment() {
        let input = "#";
        let options = super::super::ConfOptions {
            allow_c_style_comments: true,
            ..Default::default()
        };
        let lexer = Lexer::new(input, options);
        assert!(lexer.is_comment());
    }

    #[test]
    fn test_lexer_is_comment_multi_line() {
        let input = "/*";
        let options = super::super::ConfOptions {
            allow_c_style_comments: true,
            ..Default::default()
        };
        let lexer = Lexer::new(input, options);
        assert!(lexer.is_comment());
    }

    #[test]
    fn test_lexer_scan_comment_single_line() {
        let input = "# This is a comment\n";
        let options = super::super::ConfOptions {
            allow_c_style_comments: true,
            ..Default::default()
        };
        let mut lexer = Lexer::new(input, options);
        assert!(lexer.scan_comment().is_ok());
        assert_eq!(lexer.position, input.len() - 1);
    }

    #[test]
    fn test_lexer_scan_comment_multi_line() {
        let input = "/* This is a\nmulti-line\ncomment */";
        let options = super::super::ConfOptions {
            allow_c_style_comments: true,
            ..Default::default()
        };
        let mut lexer = Lexer::new(input, options);
        assert!(lexer.scan_comment().is_ok());
        assert_eq!(lexer.position, input.len());
    }

    #[test]
    fn test_lexer_scan_comment_multi_line_unterminated() {
        let input = "/* This is an unterminated comment";
        let options = super::super::ConfOptions {
            allow_c_style_comments: true,
            ..Default::default()
        };
        let mut lexer = Lexer::new(input, options);
        assert!(lexer.scan_comment().is_err());
    }

    #[test]
    fn test_lexer_scan_quoted_argument() {
        let input = "\"test\"";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let (is_triple_quoted, is_expression) = lexer.scan_quoted_argument().unwrap();
        assert!(!is_triple_quoted);
        assert!(!is_expression);
        assert_eq!(lexer.position, input.len());
    }

    #[test]
    fn test_lexer_scan_quoted_argument_with_escape() {
        let input = "\"test\\n\"";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let (is_triple_quoted, is_expression) = lexer.scan_quoted_argument().unwrap();
        assert!(!is_triple_quoted);
        assert!(!is_expression);
        assert_eq!(lexer.position, input.len());
    }

    #[test]
    fn test_lexer_scan_quoted_argument_unterminated() {
        let input = "\"test";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        assert!(lexer.scan_quoted_argument().is_err());
    }

    #[test]
    fn test_lexer_scan_quoted_argument_triple() {
        let input = "\"\"\"test\"\"\"";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let (is_triple_quoted, is_expression) = lexer.scan_quoted_argument().unwrap();
        assert!(is_triple_quoted);
        assert!(!is_expression);
        assert_eq!(lexer.position, input.len());
    }

    #[test]
    fn test_lexer_scan_quoted_argument_triple_unterminated() {
        let input = "\"\"\"test";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        assert!(lexer.scan_quoted_argument().is_err());
    }

    #[test]
    fn test_lexer_scan_argument() {
        let input = "test";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let is_expression = lexer.scan_argument().unwrap();
        assert!(!is_expression);
        assert_eq!(lexer.position, input.len());
    }

    #[test]
    fn test_lexer_scan_argument_with_escape() {
        let input = "test\\n";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        lexer.scan_argument().unwrap();
        assert_eq!(lexer.position, 6); // Должно быть 6 символов: 't', 'e', 's', 't', '\', 'n'
    }

    #[test]
    fn test_lexer_scan_argument_with_space() {
        let input = "test ";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let is_expression = lexer.scan_argument().unwrap();
        assert!(!is_expression);
        assert_eq!(lexer.position, input.len() - 1);
    }

    #[test]
    fn test_lexer_scan_argument_with_expression() {
        let input = "test(";
        let options = super::super::ConfOptions {
            allow_expression_arguments: true,
            ..Default::default()
        };
        let mut lexer = Lexer::new(input, options);
        let is_expression = lexer.scan_argument().unwrap();
        assert!(is_expression);
        assert_eq!(lexer.position, 4); // Только 'test', без '('
    }

    #[test]
    fn test_lexer_next_token_eof() {
        let input = "";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Eof);
        assert_eq!(token.span, 0..0);
        assert!(!token.is_quoted);
        assert!(!token.is_triple_quoted);
        assert!(!token.is_expression);
    }

    #[test]
    fn test_lexer_next_token_newline() {
        let input = "\n";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Newline);
        assert_eq!(token.span, 0..1);
        assert!(!token.is_quoted);
        assert!(!token.is_triple_quoted);
        assert!(!token.is_expression);
    }

    #[test]
    fn test_lexer_next_token_semicolon() {
        let input = ";";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Semicolon);
        assert_eq!(token.span, 0..1);
        assert!(!token.is_quoted);
        assert!(!token.is_triple_quoted);
        assert!(!token.is_expression);
    }

    #[test]
    fn test_lexer_next_token_left_curly_brace() {
        let input = "{";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::LeftCurlyBrace);
        assert_eq!(token.span, 0..1);
        assert!(!token.is_quoted);
        assert!(!token.is_triple_quoted);
        assert!(!token.is_expression);
    }

    #[test]
    fn test_lexer_next_token_right_curly_brace() {
        let input = "}";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::RightCurlyBrace);
        assert_eq!(token.span, 0..1);
        assert!(!token.is_quoted);
        assert!(!token.is_triple_quoted);
        assert!(!token.is_expression);
    }

    #[test]
    fn test_lexer_next_token_continuation() {
        let input = "\\\n"; // Обратный слеш + перевод строки
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Continuation);
        assert_eq!(token.span, 0..1); // Только обратный слеш
        assert!(!token.is_quoted);
        assert!(!token.is_triple_quoted);
        assert!(!token.is_expression);
    }

    #[test]
    fn test_lexer_next_token_quoted_argument() {
        let input = "\"test\"";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Argument);
        assert_eq!(token.span, 0..input.len());
        assert!(token.is_quoted);
        assert!(!token.is_triple_quoted);
        assert!(!token.is_expression);
    }

    #[test]
    fn test_lexer_next_token_triple_quoted_argument() {
        let input = "\"\"\"test\"\"\"";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Argument);
        assert_eq!(token.span, 0..input.len());
        assert!(token.is_quoted);
        assert!(token.is_triple_quoted);
        assert!(!token.is_expression);
    }

    #[test]
    fn test_lexer_next_token_argument() {
        let input = "test";
        let options = super::super::ConfOptions::default();
        let mut lexer = Lexer::new(input, options);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Argument);
        assert_eq!(token.span, 0..input.len());
        assert!(!token.is_quoted);
        assert!(!token.is_triple_quoted);
        assert!(!token.is_expression);
    }

    #[test]
    fn test_lexer_next_token_argument_with_expression() {
        let input = "test(";
        let options = super::super::ConfOptions {
            allow_expression_arguments: true,
            ..Default::default()
        };
        let mut lexer = Lexer::new(input, options);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Argument);
        assert_eq!(token.span, 0..4); // Только 'test', без '('
        assert!(!token.is_quoted);
        assert!(!token.is_triple_quoted);
        assert!(token.is_expression);
    }
}
