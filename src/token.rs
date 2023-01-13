#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub str : String
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword, Name, Number(i64),
    LF,
    Comma, Colon, SemiColon, EqualSign, MathSymbol, Logic, To, Ampersand, Star,
    LCurlyBracket, RCurlyBracket, LBracket, RBracket, LParenthesis, RParenthesis,
    Char(char), Str(String),
    Unknown,
}