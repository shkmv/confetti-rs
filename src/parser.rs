use crate::{ConfDirective, ConfUnit, ConfArgument, ConfComment, ConfError, ConfOptions};
use crate::lexer::{Lexer, Token, TokenType};

/// Parser for the configuration language.
pub struct Parser<'a> {
    /// The lexer used to tokenize the input.
    lexer: Lexer<'a>,
    /// The current token.
    current_token: Token,
    /// The options for the parser.
    options: ConfOptions,
    /// The current depth of nested directives.
    current_depth: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new parser.
    pub fn new(input: &'a str, options: ConfOptions) -> Result<Self, ConfError> {
        let mut lexer = Lexer::new(input, options.clone());
        let current_token = lexer.next_token()?;
        
        Ok(Self {
            lexer,
            current_token,
            options,
            current_depth: 0,
        })
    }

    /// Advances to the next token.
    fn advance(&mut self) -> Result<(), ConfError> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    /// Parses a configuration unit.
    pub fn parse(&mut self) -> Result<ConfUnit, ConfError> {
        let mut directives = Vec::new();
        let mut comments = Vec::new();

        while self.current_token.token_type != TokenType::Eof {
            match self.current_token.token_type {
                TokenType::Comment => {
                    let comment = self.parse_comment()?;
                    comments.push(comment);
                }
                TokenType::Newline | TokenType::Whitespace | TokenType::Continuation => {
                    self.advance()?;
                }
                _ => {
                    let directive = self.parse_directive()?;
                    directives.push(directive);
                }
            }
        }

        Ok(ConfUnit { directives, comments })
    }

    /// Parses a comment.
    fn parse_comment(&mut self) -> Result<ConfComment, ConfError> {
        if self.current_token.token_type != TokenType::Comment {
            return Err(ConfError::ParserError {
                position: self.current_token.span.start,
                message: "Expected comment".to_string(),
            });
        }

        let span = self.current_token.span.clone();
        let content = self.lexer.input()[span.clone()].to_string();
        let is_multi_line = content.starts_with("/*");

        self.advance()?;

        Ok(ConfComment {
            content,
            span,
            is_multi_line,
        })
    }

    /// Parses a directive.
    fn parse_directive(&mut self) -> Result<ConfDirective, ConfError> {
        // Check max depth
        if self.current_depth >= self.options.max_depth {
            return Err(ConfError::ParserError {
                position: self.current_token.span.start,
                message: format!("Maximum directive depth of {} exceeded", self.options.max_depth),
            });
        }

        // Parse the directive name
        if self.current_token.token_type != TokenType::Argument {
            return Err(ConfError::ParserError {
                position: self.current_token.span.start,
                message: "Expected directive name".to_string(),
            });
        }

        let name_span = self.current_token.span.clone();
        let name_value = self.lexer.input()[name_span.clone()].to_string();
        let name = ConfArgument {
            value: name_value,
            span: name_span,
            is_quoted: self.current_token.is_quoted,
            is_triple_quoted: self.current_token.is_triple_quoted,
            is_expression: self.current_token.is_expression,
        };

        self.advance()?;

        // Parse arguments
        let mut arguments = Vec::new();
        while self.current_token.token_type == TokenType::Argument || 
              self.current_token.token_type == TokenType::Continuation {
            
            // Если это токен продолжения строки, пропускаем его и продолжаем
            if self.current_token.token_type == TokenType::Continuation {
                self.advance()?;
                continue;
            }
            
            let arg_span = self.current_token.span.clone();
            let arg_value = self.lexer.input()[arg_span.clone()].to_string();
            let argument = ConfArgument {
                value: arg_value,
                span: arg_span,
                is_quoted: self.current_token.is_quoted,
                is_triple_quoted: self.current_token.is_triple_quoted,
                is_expression: self.current_token.is_expression,
            };

            arguments.push(argument);
            self.advance()?;
        }

        // Parse child directives if this is a block directive
        let mut children = Vec::new();
        if self.current_token.token_type == TokenType::LeftCurlyBrace {
            self.advance()?; // Skip '{'
            self.current_depth += 1;

            // Skip newlines after opening brace
            while self.current_token.token_type == TokenType::Newline {
                self.advance()?;
            }

            // Parse child directives
            while self.current_token.token_type != TokenType::RightCurlyBrace && 
                  self.current_token.token_type != TokenType::Eof {
                match self.current_token.token_type {
                    TokenType::Comment => {
                        let _comment = self.parse_comment()?;
                        // We don't add comments to children, they go to the ConfUnit
                    }
                    TokenType::Newline | TokenType::Whitespace => {
                        self.advance()?;
                    }
                    _ => {
                        let directive = self.parse_directive()?;
                        children.push(directive);
                    }
                }
            }

            // Expect closing brace
            if self.current_token.token_type != TokenType::RightCurlyBrace {
                return Err(ConfError::ParserError {
                    position: self.current_token.span.start,
                    message: "Expected '}'".to_string(),
                });
            }

            self.advance()?; // Skip '}'
            self.current_depth -= 1;
        } else if self.current_token.token_type == TokenType::Semicolon {
            self.advance()?; // Skip ';'
        } else if self.current_token.token_type != TokenType::Newline && 
                  self.current_token.token_type != TokenType::Eof &&
                  self.current_token.token_type != TokenType::Continuation {
            return Err(ConfError::ParserError {
                position: self.current_token.span.start,
                message: "Expected ';', '{', or newline".to_string(),
            });
        }

        Ok(ConfDirective {
            name,
            arguments,
            children,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_new() {
        let input = "test";
        let options = ConfOptions::default();
        let parser = Parser::new(input, options);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parser_parse_empty() {
        let input = "";
        let options = ConfOptions::default();
        let mut parser = Parser::new(input, options).unwrap();
        let result = parser.parse();
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 0);
        assert_eq!(conf_unit.comments.len(), 0);
    }

    #[test]
    fn test_parser_parse_simple_directive() {
        let input = "server localhost";
        let options = ConfOptions::default();
        let mut parser = Parser::new(input, options).unwrap();
        let result = parser.parse();
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].name.value, "server");
        assert_eq!(conf_unit.directives[0].arguments.len(), 1);
        assert_eq!(conf_unit.directives[0].arguments[0].value, "localhost");
    }

    #[test]
    fn test_parser_parse_block_directive() {
        let input = "server {\n  listen 80;\n}";
        let options = ConfOptions::default();
        let mut parser = Parser::new(input, options).unwrap();
        let result = parser.parse();
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].name.value, "server");
        assert_eq!(conf_unit.directives[0].arguments.len(), 0);
        assert_eq!(conf_unit.directives[0].children.len(), 1);
        assert_eq!(conf_unit.directives[0].children[0].name.value, "listen");
        assert_eq!(conf_unit.directives[0].children[0].arguments.len(), 1);
        assert_eq!(conf_unit.directives[0].children[0].arguments[0].value, "80");
    }

    #[test]
    fn test_parser_parse_with_comments() {
        let input = "# Comment\nserver localhost";
        let options = ConfOptions {
            allow_c_style_comments: true,
            ..Default::default()
        };
        let mut parser = Parser::new(input, options).unwrap();
        let result = parser.parse();
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.comments.len(), 1);
        assert_eq!(conf_unit.comments[0].content, "# Comment");
    }

    #[test]
    fn test_parser_max_depth() {
        let input = "a { b { c { d { e { f { g { h { i { j { k { } } } } } } } } } } }";
        let options = ConfOptions {
            max_depth: 5,
            ..Default::default()
        };
        let mut parser = Parser::new(input, options).unwrap();
        let result = parser.parse();
        assert!(result.is_err());
        if let Err(ConfError::ParserError { message, .. }) = result {
            assert!(message.contains("Maximum directive depth"));
        } else {
            panic!("Expected ParserError");
        }
    }
}