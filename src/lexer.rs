use std::str;

use nom::{IResult, alpha, alphanumeric, anychar, multispace, not_line_ending, digit};

use errors::*;

pub struct Lexer<'a> {
    data: &'a [u8],
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    // Comparison operators
    Equal,
    Greater,
    GreaterEqual,
    Lesser,
    LesserEqual,
    NotEqual,

    // Assign operators
    Assign,
    AddAssign,
    DivAssign,
    MultAssign,
    SubAssign,

    // Arithmetic operators
    Minus,
    Percent,
    Plus,
    Slash,
    Star,

    // Delimiters
    CloseBrace,
    CloseIndex,
    CloseParen,
    OpenBrace,
    OpenBracket,
    OpenParen,

    // Keywords
    If,
    ElseIf,
    Else,
    For,
    While,
    
    Char(char),
    Comment,
    Eof,
    Ident(String),
    Integer(i32),
    String(String),
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Lexer { data }
    }

    pub fn generate_tokens(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = match get_token(self.data) {
                IResult::Done(remaining, token) => {
                    self.data = remaining;
                    token
                }
                IResult::Incomplete(needed) =>
                    return Err(format!("Incomplete parsing, {:?} bytes missing", needed).into()),
                IResult::Error(e) => return Err(format!("Parsing error: {}", e).into()), 
            };

            match token {
                Token::Eof => {
                    debug!("Eof");
                    break;
                }
                token => {
                    debug!("{:?}", token);
                    tokens.push(token);
                },
            }
        }

        Ok(tokens)
    }
}

named!(get_token<Token>,
       alt!(
           file_end
               | string
               | delimiter
               | keyword
               | ident
               | comment
               | comp_op
               | assign_op
               | arith_op
               | integer
               | any
       )
);

named!(delimiter<Token>,
       map!(ws!(one_of!("(){}[]")),
            |delim: char| match delim {
                '{' => Token::OpenBrace,
                '}' => Token::CloseBrace,
                '(' => Token::OpenParen,
                ')' => Token::CloseParen,
                '[' => Token::OpenBracket,
                ']' => Token::CloseIndex,
                _ => unreachable!(),
            }
       )
);

named!(arith_op<Token>,
       map!(ws!(one_of!("+-*/%")),
            |delim: char| match delim {
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Star,
                '/' => Token::Slash,
                '%' => Token::Percent,
                _ => unreachable!(),
            }
       )
);

named!(any<Token>,
       do_parse!(
           ch: ws!(anychar) >>
           (Token::Char(ch))
       )
);

named!(keyword<Token>,
       map!(
           map_res!(ws!(alt!(tag!("if")
                             | tag!("else if")
                             | tag!("else")
                             | tag!("for")
                             | tag!("while"))),
                    str::from_utf8
           ),
           |word: &str| match word {
               "if" => Token::If,
               "else if" => Token::ElseIf,
               "else" => Token::Else,
               "for" => Token::For,
               "while" => Token::While,
               _ => unreachable!(),
           }
       )
);

named!(ident<Token>,
       do_parse!(
           many0!(multispace) >>
           init: map!(alpha, |init: &[u8]| init.to_vec()) >>
           result: map_res!(
               fold_many0!(
                   alt!(alphanumeric | tag!("_")),
                   init, |mut acc: Vec<_>, part| {
                       acc.extend(part);
                       acc
                   }
               ),
               String::from_utf8
           ) >>
           (Token::Ident(result))
       )
);

named!(comp_op<Token>,
       map!(
           map_res!(
               ws!(alt!(
                   tag!("<=")
                       | tag!(">=")
                       | tag!("!=")
                       | tag!("==")
                       | tag!("<")
                       | tag!(">")
               )),
               str::from_utf8
           ),
           |op: &str| match op {
               "<=" => Token::LesserEqual,
               ">=" => Token::GreaterEqual,
               "==" => Token::Equal,
               "<" => Token::Lesser,
               ">" => Token::Greater,
               "!=" => Token::NotEqual,
               _ => unreachable!(),
           }
       ) 
);

named!(string<Token>,
       do_parse!(
           string: map_res!(
               map!(
                   ws!(delimited!(char!('"'), is_not!("\""), char!('"'))),
                   |array: &[u8]| array.to_vec()
               ),
               String::from_utf8
           ) >>
           (Token::String(string))
       )
);

named!(assign_op<Token>,
       map!(
           map_res!(
               ws!(alt!(tag!(":=") | tag!("+=") | tag!("-=") | tag!("*=") | tag!("/="))),
               str::from_utf8
           ),
           |op: &str| match op {
               ":=" => Token::Assign,
               "+=" => Token::AddAssign,
               "-=" => Token::SubAssign,
               "*=" => Token::MultAssign,
               "/=" => Token::DivAssign,
               _ => unreachable!(),
           }
       )
);

named!(file_end<Token>,
       do_parse!(
           ws!(eof!()) >>
           (Token::Eof)
       )
);

named!(comment<Token>,
       do_parse!(
           preceded!(ws!(tag!("//")), not_line_ending) >>
           (Token::Comment)
       )
);

named!(integer<Token>,
       do_parse!(
           as_digit: map_res!(
               map_res!(
                   ws!(digit),
                   str::from_utf8
               ),
               str::parse
           ) >>
           (Token::Integer(as_digit))
       )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_end() {
        let source = b"  ";
        let result = get_token(source);
        assert!(result.is_done());
        assert_eq!(Token::Eof, result.unwrap().1);
    }

    #[test]
    fn test_parse_keyword() {
        let source = b" if else if else for while";

        let mut result = get_token(source);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::If, token);
        
        result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::ElseIf, token);

        result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::Else, token);

        result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::For, token);

        result = get_token(remaining);
        assert!(result.is_done());
        let (_, token) = result.unwrap();
        assert_eq!(Token::While, token);

    }

    #[test]
    fn test_parse_delimiter() {
        let source = b" () [] {} ";

        let mut result = get_token(source);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::OpenParen, token);
        
        result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::CloseParen, token);

        result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::OpenBracket, token);

        result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::CloseIndex, token);

        result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::OpenBrace, token);

        result = get_token(remaining);
        assert!(result.is_done());
        let (_, token) = result.unwrap();
        assert_eq!(Token::CloseBrace, token);
    }

    #[test]
    fn test_parse_ident() {
        let source = b" name num_rows_10 ";
        let result = get_token(source);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::Ident("name".to_owned()), token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (_, token) = result.unwrap();
        assert_eq!(Token::Ident("num_rows_10".to_owned()), token);
    }

    #[test]
    fn test_parse_arithmetic_operator() {
        let source = b" + - * / %";

        let result = get_token(source);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::Plus, token);
        
        let result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::Minus, token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::Star, token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::Slash, token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (_, token) = result.unwrap();
        assert_eq!(Token::Percent, token);
        
    }

    #[test]
    fn test_parse_string() {
        let source = b" \"Hello friend\"";
        let result = get_token(source);
        assert!(result.is_done());
        let (_, token) = result.unwrap();
        assert_eq!(Token::String("Hello friend".to_owned()) , token);
    }

    #[test]
    fn test_parse_int() {
        let source = b" 457";
        let result = get_token(source);
        assert!(result.is_done());
        assert_eq!(Token::Integer(457), result.unwrap().1);
    }

    #[test]
    fn test_parse_assign_operator() {
        let source = b" := += -= *= /=";

        let result = get_token(source);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::Assign, token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::AddAssign, token);
        
        let result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::SubAssign, token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::MultAssign, token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (_, token) = result.unwrap();
        assert_eq!(Token::DivAssign, token);
    }

    #[test]
    fn test_parse_comparison_operator() {
        let source = b" < > <= >= == != ";

        let result = get_token(source);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::Lesser, token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::Greater, token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::LesserEqual, token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::GreaterEqual, token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (remaining, token) = result.unwrap();
        assert_eq!(Token::Equal, token);

        let result = get_token(remaining);
        assert!(result.is_done());
        let (_, token) = result.unwrap();
        assert_eq!(Token::NotEqual, token);
    }

    #[test]
    fn test_parse_comment() {
        let source = b" // hello there!!\n";
        let result = get_token(source);
        assert!(result.is_done());
        assert_eq!(Token::Comment, result.unwrap().1);
    }
}
