use std::error::Error as StdError;
use std::fmt;
use std::fmt::Formatter;
use std::iter::Peekable;
use std::str::FromStr;
use thiserror::Error;

/// 位置情報。 .0 から .1 までの区間を表す
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Loc(usize, usize);

impl Loc {
    fn merge(&self, other: &Loc) -> Loc {
        use std::cmp::{max, min};
        Loc(min(self.0, other.0), max(self.1, other.1))
    }
}

/// アノテーション。値に様々なデータを持たせたもの。
/// ここでは Loc を持たせている。
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Annotation<T> {
    value: T,
    loc: Loc,
}

impl<T> Annotation<T> {
    fn new(value: T, loc: Loc) -> Self {
        Self { value, loc }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TokenKind {
    Number(u64),
    Plus,
    Minus,
    Asterisk,
    Slash,
    LParen,
    RParen,
}

/// TokenKind にアノテーションを付けたものを Token として定義する。
type Token = Annotation<TokenKind>;

impl Token {
    fn number(n: u64, loc: Loc) -> Self {
        Self::new(TokenKind::Number(n), loc)
    }

    fn plus(loc: Loc) -> Self {
        Self::new(TokenKind::Plus, loc)
    }

    fn minus(loc: Loc) -> Self {
        Self::new(TokenKind::Minus, loc)
    }

    fn asterisk(loc: Loc) -> Self {
        Self::new(TokenKind::Asterisk, loc)
    }

    fn slash(loc: Loc) -> Self {
        Self::new(TokenKind::Slash, loc)
    }

    fn lparen(loc: Loc) -> Self {
        Self::new(TokenKind::LParen, loc)
    }

    fn rparen(loc: Loc) -> Self {
        Self::new(TokenKind::RParen, loc)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum LexErrorKind {
    InvalidChar(char),
    Eof,
}

type LexError = Annotation<LexErrorKind>;

impl LexError {
    fn invalid_char(c: char, loc: Loc) -> Self {
        Self::new(LexErrorKind::InvalidChar(c), loc)
    }

    fn eof(loc: Loc) -> Self {
        Self::new(LexErrorKind::Eof, loc)
    }
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let input = input.as_bytes();
    let mut pos = 0;
    macro_rules! lex_a_token {
        ($lexer:expr) => {{
            let (tok, p) = $lexer?;
            tokens.push(tok);
            pos = p;
        }};
    }

    while pos < input.len() {
        match input[pos] {
            b'0'..=b'9' => lex_a_token!(lex_number(input, pos)),
            b'+' => lex_a_token!(lex_plus(input, pos)),
            b'-' => lex_a_token!(lex_minus(input, pos)),
            b'*' => lex_a_token!(lex_asterisk(input, pos)),
            b'/' => lex_a_token!(lex_slash(input, pos)),
            b'(' => lex_a_token!(lex_lparen(input, pos)),
            b')' => lex_a_token!(lex_rparen(input, pos)),
            b' ' | b'\n' | b'\t' => {
                let ((), p) = skip_spaces(input, pos)?;
                pos = p;
            }
            b => return Err(LexError::invalid_char(b as char, Loc(pos, pos + 1))),
        }
    }

    Ok(tokens)
}

fn consume_byte(input: &[u8], pos: usize, b: u8) -> Result<(u8, usize), LexError> {
    if input.len() <= pos {
        return Err(LexError::eof(Loc(pos, pos)));
    }

    if input[pos] != b {
        return Err(LexError::invalid_char(
            input[pos] as char,
            Loc(pos, pos + 1),
        ));
    }

    Ok((b, pos + 1))
}

fn lex_plus(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'+').map(|(_, end)| (Token::plus(Loc(start, end)), end))
}

fn lex_minus(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'-').map(|(_, end)| (Token::minus(Loc(start, end)), end))
}

fn lex_asterisk(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'*').map(|(_, end)| (Token::asterisk(Loc(start, end)), end))
}

fn lex_slash(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'/').map(|(_, end)| (Token::slash(Loc(start, end)), end))
}

fn lex_lparen(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'(').map(|(_, end)| (Token::lparen(Loc(start, end)), end))
}

fn lex_rparen(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b')').map(|(_, end)| (Token::rparen(Loc(start, end)), end))
}

fn lex_number(input: &[u8], pos: usize) -> Result<(Token, usize), LexError> {
    use std::str::from_utf8;

    let start = pos;
    let end = recognize_many(input, start, |b| b"1234567890".contains(&b));

    let n = from_utf8(&input[start..end]).unwrap().parse().unwrap();
    Ok((Token::number(n, Loc(start, end)), end))
}

fn skip_spaces(input: &[u8], pos: usize) -> Result<((), usize), LexError> {
    let pos = recognize_many(input, pos, |b| b" \n\t".contains(&b));
    Ok(((), pos))
}

fn recognize_many(input: &[u8], mut pos: usize, mut f: impl FnMut(u8) -> bool) -> usize {
    while pos < input.len() && f(input[pos]) {
        pos += 1;
    }
    pos
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AstKind {
    /// 数値
    Num(u64),
    /// 単項演算
    UniOp { op: UniOp, e: Box<Ast> },
    /// 二項演算
    BinOp { op: BinOp, l: Box<Ast>, r: Box<Ast> },
}

pub type Ast = Annotation<AstKind>;

impl Ast {
    fn num(n: u64, loc: Loc) -> Self {
        Self::new(AstKind::Num(n), loc)
    }

    fn uni_op(op: UniOp, e: Ast, loc: Loc) -> Self {
        Self::new(AstKind::UniOp { op, e: Box::new(e) }, loc)
    }

    fn bin_op(op: BinOp, l: Ast, r: Ast, loc: Loc) -> Self {
        Self::new(
            AstKind::BinOp {
                op,
                l: Box::new(l),
                r: Box::new(r),
            },
            loc,
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum UniOpKind {
    /// 正号
    Plus,
    /// 負号
    Minus,
}

type UniOp = Annotation<UniOpKind>;

impl UniOp {
    fn plus(loc: Loc) -> Self {
        Self::new(UniOpKind::Plus, loc)
    }

    fn minus(loc: Loc) -> Self {
        Self::new(UniOpKind::Minus, loc)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BinOpKind {
    /// 加算
    Add,
    /// 減算
    Sub,
    /// 乗算
    Multi,
    /// 除算
    Div,
}

type BinOp = Annotation<BinOpKind>;

impl BinOp {
    fn add(loc: Loc) -> Self {
        Self::new(BinOpKind::Add, loc)
    }

    fn sub(loc: Loc) -> Self {
        Self::new(BinOpKind::Sub, loc)
    }

    fn multi(loc: Loc) -> Self {
        Self::new(BinOpKind::Multi, loc)
    }

    fn div(loc: Loc) -> Self {
        Self::new(BinOpKind::Div, loc)
    }
}

#[derive(Error, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ParseError {
    /// 予期しないトークンがきた
    #[error("{}: {} is not expected", .0.loc, .0.value)]
    UnexpectedToken(Token),
    /// 式を期待していたのに式でないものがきた
    #[error("{}: '{}' is not a start of expression", .0.loc, .0.value)]
    NotExpression(Token),
    /// 演算子を期待していたのに演算子でないものがきた
    #[error("{}: '{}' is not an operator", .0.loc, .0.value)]
    NotOperator(Token),
    /// 括弧が閉じられていない
    #[error("{}: '{}' is not closed", .0.loc, .0.value)]
    UnclosedOpenParen(Token),
    /// 式の解析が終わったのにまだトークンが残っている
    #[error("{}: expression after '{}' is redundant", .0.loc, .0.value)]
    RedundantExpression(Token),
    /// パース途中で入力が終わった
    #[error("End of file")]
    Eof,
}

pub fn parse(tokens: Vec<Token>) -> Result<Ast, ParseError> {
    let mut tokens = tokens.into_iter().peekable();
    let ret = parse_expr(&mut tokens)?;
    match tokens.next() {
        Some(tok) => Err(ParseError::RedundantExpression(tok)),
        None => Ok(ret),
    }
}

fn parse_expr<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
where
    Tokens: Iterator<Item = Token>,
{
    parse_expr3(tokens)
}

fn parse_expr3<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
where
    Tokens: Iterator<Item = Token>,
{
    fn parse_expr3_op<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<BinOp, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let op = tokens
            .peek()
            .ok_or(ParseError::Eof)
            .and_then(|tok| match tok.value {
                TokenKind::Plus => Ok(BinOp::add(tok.loc.clone())),
                TokenKind::Minus => Ok(BinOp::sub(tok.loc.clone())),
                _ => Err(ParseError::NotOperator(tok.clone())),
            })?;
        tokens.next();
        Ok(op)
    }

    parse_left_binop(tokens, parse_expr2, parse_expr3_op)
}

fn parse_expr2<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
where
    Tokens: Iterator<Item = Token>,
{
    let mut e = parse_expr1(tokens)?;
    loop {
        match tokens.peek().map(|tok| tok.value) {
            Some(TokenKind::Asterisk) | Some(TokenKind::Slash) => {
                let op = match tokens.next().unwrap() {
                    Token {
                        value: TokenKind::Asterisk,
                        loc,
                    } => BinOp::multi(loc),
                    Token {
                        value: TokenKind::Slash,
                        loc,
                    } => BinOp::div(loc),
                    _ => unreachable!(),
                };
                let r = parse_expr1(tokens)?;
                let loc = e.loc.merge(&r.loc);
                e = Ast::bin_op(op, e, r, loc);
            }
            _ => return Ok(e),
        }
    }
}

fn parse_expr1<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
where
    Tokens: Iterator<Item = Token>,
{
    match tokens.peek().map(|tok| tok.value) {
        Some(TokenKind::Plus) | Some(TokenKind::Minus) => {
            let op = match tokens.next() {
                Some(Token {
                    value: TokenKind::Plus,
                    loc,
                }) => UniOp::plus(loc),
                Some(Token {
                    value: TokenKind::Minus,
                    loc,
                }) => UniOp::minus(loc),
                _ => unreachable!(),
            };

            let e = parse_atom(tokens)?;
            let loc = op.loc.merge(&e.loc);
            Ok(Ast::uni_op(op, e, loc))
        }
        _ => parse_atom(tokens),
    }
}

fn parse_atom<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
where
    Tokens: Iterator<Item = Token>,
{
    tokens
        .next()
        .ok_or(ParseError::Eof)
        .and_then(|tok| match tok.value {
            TokenKind::Number(n) => Ok(Ast::num(n, tok.loc)),
            TokenKind::LParen => {
                let e = parse_expr(tokens)?;
                match tokens.next() {
                    Some(Token {
                        value: TokenKind::RParen,
                        ..
                    }) => Ok(e),
                    Some(t) => Err(ParseError::RedundantExpression(t)),
                    _ => Err(ParseError::UnclosedOpenParen(tok)),
                }
            }
            _ => Err(ParseError::NotExpression(tok)),
        })
}

fn parse_left_binop<Tokens>(
    tokens: &mut Peekable<Tokens>,
    sub_expr_parser: fn(&mut Peekable<Tokens>) -> Result<Ast, ParseError>,
    op_parser: fn(&mut Peekable<Tokens>) -> Result<BinOp, ParseError>,
) -> Result<Ast, ParseError>
where
    Tokens: Iterator<Item = Token>,
{
    let mut e = sub_expr_parser(tokens)?;
    while tokens.peek().is_some() {
        let op = match op_parser(tokens) {
            Ok(op) => op,
            Err(_) => break,
        };
        let r = sub_expr_parser(tokens)?;
        let loc = e.loc.merge(&r.loc);
        e = Ast::bin_op(op, e, r, loc);
    }

    Ok(e)
}

#[derive(Error, Debug, Clone, Eq, PartialEq, Hash)]
pub enum Error {
    #[error("lexer error")]
    Lexer(#[from] LexError),
    #[error("parser error")]
    Parser(#[from] ParseError),
}

impl FromStr for Ast {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = lex(s)?;
        let ast = parse(tokens)?;
        Ok(ast)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::TokenKind::*;
        match self {
            Number(n) => n.fmt(f),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Asterisk => write!(f, "*"),
            Slash => write!(f, "/"),
            LParen => write!(f, "("),
            RParen => write!(f, ")"),
        }
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.0, self.1)
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::LexErrorKind::*;
        let loc = &self.loc;
        match self.value {
            InvalidChar(c) => write!(f, "{}: invalid char '{}'", loc, c),
            Eof => write!(f, "End of file"),
        }
    }
}

// impl fmt::Display for ParseError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         use self::ParseError::*;
//         match self {
//             UnexpectedToken(tok) => write!(f, "{}: {} is not expected", tok.loc, tok.value),
//             NotExpression(tok) => write!(
//                 f,
//                 "{}: '{}' is not a start of expression",
//                 tok.loc, tok.value
//             ),
//             NotOperator(tok) => write!(f, "{}: '{}' is not an operator", tok.loc, tok.value),
//             UnclosedOpenParen(tok) => write!(f, "{}: '{}' is not closed", tok.loc, tok.value),
//             RedundantExpression(tok) => write!(
//                 f,
//                 "{}: expression after '{}' is redundant",
//                 tok.loc, tok.value
//             ),
//             Eof => write!(f, "End of file"),
//         }
//     }
// }

impl StdError for LexError {}

// impl StdError for ParseError {}

fn print_annotation(input: &str, loc: Loc) {
    eprintln!("{}", input);
    eprintln!("{}{}", " ".repeat(loc.0), "^".repeat(loc.1 - loc.0));
}

impl Error {
    /// 診断メッセージを表示する
    pub fn show_diagnostic(&self, input: &str) {
        use self::Error::*;
        use self::ParseError as P;
        let (e, loc): (&dyn StdError, Loc) = match self {
            Lexer(e) => (e, e.loc.clone()),
            Parser(e) => {
                let loc = match e {
                    P::UnexpectedToken(Token { loc, .. })
                    | P::NotExpression(Token { loc, .. })
                    | P::NotOperator(Token { loc, .. })
                    | P::UnclosedOpenParen(Token { loc, .. }) => loc.clone(),
                    P::RedundantExpression(Token { loc, .. }) => Loc(loc.0, input.len()),
                    P::Eof => Loc(input.len(), input.len() + 1),
                };
                (e, loc)
            }
        };
        eprintln!("{}", e);
        print_annotation(input, loc);
    }
}

/// 評価器を表すデータ型
pub struct Interpreter;

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum InterpreterErrorKind {
    DivisionByZero,
}

type InterpreterError = Annotation<InterpreterErrorKind>;

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::InterpreterErrorKind::*;
        match self.value {
            DivisionByZero => write!(f, "division by zero"),
        }
    }
}

impl StdError for InterpreterError {
    fn description(&self) -> &str {
        use self::InterpreterErrorKind::*;
        match self.value {
            DivisionByZero => "the right hand expression of the division evaluates to zero",
        }
    }
}

impl InterpreterError {
    pub fn show_diagnostic(&self, input: &str) {
        eprintln!("{}", self);
        print_annotation(input, self.loc.clone());
    }
}

impl Interpreter {
    pub fn eval(&mut self, expr: &Ast) -> Result<i64, InterpreterError> {
        use self::AstKind::*;
        match expr.value {
            Num(n) => Ok(n as i64),
            UniOp { ref op, ref e } => {
                let e = self.eval(e)?;
                Ok(self.eval_uni_op(op, e))
            }
            BinOp {
                ref op,
                ref l,
                ref r,
            } => {
                let l = self.eval(l)?;
                let r = self.eval(r)?;
                self.eval_bin_op(op, l, r)
                    .map_err(|e| InterpreterError::new(e, expr.loc.clone()))
            }
        }
    }

    fn eval_uni_op(&mut self, op: &UniOp, n: i64) -> i64 {
        use self::UniOpKind::*;
        match op.value {
            Plus => n,
            Minus => -n,
        }
    }

    fn eval_bin_op(&mut self, op: &BinOp, l: i64, r: i64) -> Result<i64, InterpreterErrorKind> {
        use self::BinOpKind::*;
        match op.value {
            Add => Ok(l + r),
            Sub => Ok(l - r),
            Multi => Ok(l * r),
            Div => {
                if r == 0 {
                    Err(InterpreterErrorKind::DivisionByZero)
                } else {
                    Ok(l / r)
                }
            }
        }
    }
}

pub struct RpnCompiler;

impl Default for RpnCompiler {
    fn default() -> Self {
        RpnCompiler
    }
}

impl RpnCompiler {
    pub fn compile(&mut self, expr: &Ast) -> String {
        let mut buf = String::new();
        self.compile_inner(expr, &mut buf);
        buf
    }

    fn compile_inner(&mut self, expr: &Ast, buf: &mut String) {
        use self::AstKind::*;
        match expr.value {
            Num(n) => buf.push_str(&n.to_string()),
            UniOp { ref op, ref e } => {
                self.compile_uni_op(op, buf);
                self.compile_inner(e, buf);
            }
            BinOp {
                ref op,
                ref l,
                ref r,
            } => {
                self.compile_inner(l, buf);
                buf.push(' ');
                self.compile_inner(r, buf);
                buf.push(' ');
                self.compile_bin_op(op, buf);
            }
        }
    }

    fn compile_uni_op(&mut self, op: &UniOp, buf: &mut String) {
        use self::UniOpKind::*;
        match op.value {
            Plus => buf.push('+'),
            Minus => buf.push('-'),
        }
    }

    fn compile_bin_op(&mut self, op: &BinOp, buf: &mut String) {
        use self::BinOpKind::*;
        match op.value {
            Add => buf.push('+'),
            Sub => buf.push('-'),
            Multi => buf.push('*'),
            Div => buf.push('/'),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_tokens() -> Vec<Token> {
        vec![
            Token::number(1, Loc(0, 1)),
            Token::plus(Loc(2, 3)),
            Token::number(2, Loc(4, 5)),
            Token::asterisk(Loc(6, 7)),
            Token::number(3, Loc(8, 9)),
            Token::minus(Loc(10, 11)),
            Token::minus(Loc(12, 13)),
            Token::number(10, Loc(14, 16)),
        ]
    }

    #[test]
    fn test_lexer() {
        assert_eq!(lex("1 + 2 * 3 - - 10"), Ok(create_tokens()));
    }

    #[test]
    fn test_parser() {
        let ast = parse(create_tokens());
        assert_eq!(
            ast,
            Ok(Ast::bin_op(
                BinOp::sub(Loc(10, 11)),
                Ast::bin_op(
                    BinOp::add(Loc(2, 3)),
                    Ast::num(1, Loc(0, 1)),
                    Ast::bin_op(
                        BinOp::new(BinOpKind::Multi, Loc(6, 7)),
                        Ast::num(2, Loc(4, 5)),
                        Ast::num(3, Loc(8, 9)),
                        Loc(4, 9)
                    ),
                    Loc(0, 9),
                ),
                Ast::uni_op(
                    UniOp::minus(Loc(12, 13)),
                    Ast::num(10, Loc(14, 16)),
                    Loc(12, 16)
                ),
                Loc(0, 16)
            ))
        );
    }

    #[test]
    fn test_parse_error_redundant() {
        assert_eq!(
            "(+ 1 3)".parse::<Ast>(),
            Err(Error::Parser(ParseError::RedundantExpression(
                Token::number(3, Loc(5, 6))
            )))
        );
    }

    #[test]
    fn test_parse_error_unclosed_open_paren() {
        assert_eq!(
            "1 + (2 - 3".parse::<Ast>(),
            Err(Error::Parser(ParseError::UnclosedOpenParen(Token::lparen(
                Loc(4, 5)
            ))))
        );
    }

    #[test]
    fn test_parse_error_not_expression() {
        assert_eq!(
            "1 + 2 - * 3".parse::<Ast>(),
            Err(Error::Parser(ParseError::NotExpression(Token::asterisk(
                Loc(8, 9)
            ))))
        );
    }

    #[test]
    fn test_parse_error_eof() {
        assert_eq!("1 +".parse::<Ast>(), Err(Error::Parser(ParseError::Eof)));
    }

    #[test]
    fn test_parse_error_invalid_char() {
        assert_eq!(
            "aiueo".parse::<Ast>(),
            Err(Error::Lexer(LexError::invalid_char('a', Loc(0, 1))))
        );
    }
}
