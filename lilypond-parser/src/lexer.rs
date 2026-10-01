use crate::error::{ParseError, ParseErrorKind};
use crate::token::{Token, TokenKind};

pub fn tokenize(src: &str) -> Result<Vec<Token>, ParseError> {
    let bytes = src.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let start = i;
        let b = bytes[i];

        match b {
            b' ' | b'\t' | b'\n' | b'\r' => {
                i += 1;
            }
            b'%' => {
                i += 1;
                if i < bytes.len() && bytes[i] == b'{' {
                    i += 1;
                    loop {
                        if i + 1 >= bytes.len() {
                            return Err(ParseError::new(
                                ParseErrorKind::UnterminatedBlockComment,
                                start..bytes.len(),
                            ));
                        }
                        if bytes[i] == b'%' && bytes[i + 1] == b'}' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                } else {
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                }
            }
            b'.' => {
                tokens.push(Token::new(TokenKind::Dot, start..start + 1));
                i += 1;
            }
            b'\'' => {
                tokens.push(Token::new(TokenKind::Apostrophe, start..start + 1));
                i += 1;
            }
            b',' => {
                tokens.push(Token::new(TokenKind::Comma, start..start + 1));
                i += 1;
            }
            b'<' => {
                tokens.push(Token::new(TokenKind::LAngle, start..start + 1));
                i += 1;
            }
            b'>' => {
                tokens.push(Token::new(TokenKind::RAngle, start..start + 1));
                i += 1;
            }
            b'{' => {
                tokens.push(Token::new(TokenKind::LBrace, start..start + 1));
                i += 1;
            }
            b'}' => {
                tokens.push(Token::new(TokenKind::RBrace, start..start + 1));
                i += 1;
            }
            b'/' => {
                tokens.push(Token::new(TokenKind::Slash, start..start + 1));
                i += 1;
            }
            b'\\' => {
                i += 1;
                let name_start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphabetic()) {
                    i += 1;
                }
                if i == name_start {
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedChar('\\'),
                        start..i,
                    ));
                }
                let name = std::str::from_utf8(&bytes[name_start..i])
                    .unwrap()
                    .to_string();
                tokens.push(Token::new(TokenKind::Command(name), start..i));
            }
            c if c.is_ascii_digit() => {
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                let n: u32 = std::str::from_utf8(&bytes[start..i])
                    .unwrap()
                    .parse()
                    .unwrap();
                tokens.push(Token::new(TokenKind::Number(n), start..i));
            }
            c if c.is_ascii_alphabetic() => {
                while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                    i += 1;
                }
                let name = std::str::from_utf8(&bytes[start..i]).unwrap().to_string();
                tokens.push(Token::new(TokenKind::Ident(name), start..i));
            }
            other => {
                return Err(ParseError::new(
                    ParseErrorKind::UnexpectedChar(other as char),
                    start..start + 1,
                ));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        tokenize(src).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn simple_note() {
        assert_eq!(
            kinds("c'4"),
            vec![
                TokenKind::Ident("c".into()),
                TokenKind::Apostrophe,
                TokenKind::Number(4),
            ]
        );
    }

    #[test]
    fn chord_and_command() {
        assert_eq!(
            kinds("\\clef treble <c e g>4"),
            vec![
                TokenKind::Command("clef".into()),
                TokenKind::Ident("treble".into()),
                TokenKind::LAngle,
                TokenKind::Ident("c".into()),
                TokenKind::Ident("e".into()),
                TokenKind::Ident("g".into()),
                TokenKind::RAngle,
                TokenKind::Number(4),
            ]
        );
    }

    #[test]
    fn comments_skipped() {
        assert_eq!(
            kinds("c4 % trailing\nd4"),
            vec![
                TokenKind::Ident("c".into()),
                TokenKind::Number(4),
                TokenKind::Ident("d".into()),
                TokenKind::Number(4),
            ]
        );
        assert_eq!(
            kinds("c4 %{ block %} d4"),
            vec![
                TokenKind::Ident("c".into()),
                TokenKind::Number(4),
                TokenKind::Ident("d".into()),
                TokenKind::Number(4),
            ]
        );
    }
}
