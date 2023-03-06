use super::{LeftBrace, LeftBracket, LeftParen, RightBrace, RightBracket, RightParen, SingleToken};

pub trait Delimiter {
    type Left: SingleToken;
    type Right: SingleToken;
}

pub struct Paren;
pub struct Bracket;
pub struct Brace;

impl Delimiter for Paren {
    type Left = LeftParen;
    type Right = RightParen;
}

impl Delimiter for Bracket {
    type Left = LeftBracket;
    type Right = RightBracket;
}

impl Delimiter for Brace {
    type Left = LeftBrace;
    type Right = RightBrace;
}
