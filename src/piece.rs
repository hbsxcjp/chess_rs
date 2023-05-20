pub enum Color {
    Red,
    Black,
}

pub enum Kind {
    King,
    Advisor,
    Bishop,
    Knight,
    Rook,
    Cannon,
    Pawn,
}

pub enum Piece {
    None,
    Some(Color, Kind),
}

impl Piece {
    const NULLCHAR: char = '_';

    const KINGCHAR: char = 'k';
    const ADVISORCHAR: char = 'a';
    const BISHOPCHAR: char = 'b';
    const KNIGHTCHAR: char = 'n';
    const ROOKCHAR: char = 'r';
    const CANNONCHAR: char = 'c';
    const PAWNCHAR: char = 'p';

    const NULLNAME: char = '空';

    const REDKINGNAME: char = '帅';
    const REDADVISORNAME: char = '仕';
    const REDBISHOPNAME: char = '相';
    const KNIGHTNAME: char = '马';
    const ROOKNAME: char = '车';
    const CANNONNAME: char = '炮';
    const REDPAWNNAME: char = '兵';

    const BLACKKINGNAME: char = '将';
    const BLACKADVISORNAME: char = '士';
    const BLACKBISHOPNAME: char = '象';
    const BLACKKNIGHTNAME: char = '馬';
    const BLACKROOKNAME: char = '車';
    const BLACKCANNONNAME: char = '砲';
    const BLACKPAWNNAME: char = '卒';

    pub fn is_line_move(kind: &Kind) -> bool {
        match kind {
            Kind::King => true,
            Kind::Rook => true,
            Kind::Cannon => true,
            Kind::Pawn => true,
            _ => false,
        }
    }

    pub fn other_color(color: &Color) -> Color {
        match color {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        }
    }

    pub fn from(ch: char) -> Self {
        match ch != Self::NULLCHAR {
            true => Self::Some(Self::color(ch), Self::kind(ch)),
            false => Self::None,
        }
    }

    fn color(ch: char) -> Color {
        match ch.is_ascii_uppercase() {
            true => Color::Red,
            false => Color::Black,
        }
    }

    fn kind(ch: char) -> Kind {
        match ch.to_ascii_lowercase() {
            Self::KINGCHAR => Kind::King,
            Self::ADVISORCHAR => Kind::Advisor,
            Self::BISHOPCHAR => Kind::Bishop,
            Self::KNIGHTCHAR => Kind::Knight,
            Self::ROOKCHAR => Kind::Rook,
            Self::CANNONCHAR => Kind::Cannon,
            _ => Kind::Pawn,
        }
    }

    pub fn ch(&self) -> char {
        match self {
            Self::None => Self::NULLCHAR,
            Self::Some(color, kind) => Self::color_ch(color, Self::kind_ch(kind)),
        }
    }

    fn color_ch(color: &Color, ch: char) -> char {
        match color {
            Color::Red => ch.to_ascii_uppercase(),
            Color::Black => ch,
        }
    }

    fn kind_ch(kind: &Kind) -> char {
        match kind {
            Kind::King => Self::KINGCHAR,
            Kind::Advisor => Self::ADVISORCHAR,
            Kind::Bishop => Self::BISHOPCHAR,
            Kind::Knight => Self::KNIGHTCHAR,
            Kind::Rook => Self::ROOKCHAR,
            Kind::Cannon => Self::CANNONCHAR,
            Kind::Pawn => Self::PAWNCHAR,
        }
    }

    pub fn name(&self) -> char {
        match self {
            Self::None => Self::NULLNAME,
            Self::Some(color, kind) => match kind {
                Kind::King => match color {
                    Color::Red => Self::REDKINGNAME,
                    Color::Black => Self::BLACKKINGNAME,
                },
                Kind::Advisor => match color {
                    Color::Red => Self::REDADVISORNAME,
                    Color::Black => Self::BLACKADVISORNAME,
                },
                Kind::Bishop => match color {
                    Color::Red => Self::REDBISHOPNAME,
                    Color::Black => Self::BLACKBISHOPNAME,
                },
                Kind::Knight => Self::KNIGHTNAME,
                Kind::Rook => Self::ROOKNAME,
                Kind::Cannon => Self::CANNONNAME,
                Kind::Pawn => match color {
                    Color::Red => Self::REDPAWNNAME,
                    Color::Black => Self::BLACKPAWNNAME,
                },
            },
        }
    }

    pub fn print_name(&self) -> char {
        match self {
            Self::None => Self::NULLNAME,
            Self::Some(color, kind) => match color {
                Color::Black => match kind {
                    Kind::Knight => Self::BLACKKNIGHTNAME,
                    Kind::Rook => Self::BLACKROOKNAME,
                    Kind::Cannon => Self::BLACKCANNONNAME,
                    _ => self.name(),
                },
                Color::Red => self.name(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ch_names() {
        let piece_chars = "_KABNRCPkabnrcp";
        let piece_names = "空帅仕相马车炮兵将士象马车炮卒";
        let piece_print_names = "空帅仕相马车炮兵将士象馬車砲卒";
        let mut chars_result = String::new();
        let mut names_result = String::new();
        let mut print_names_result = String::new();

        for ch in piece_chars.chars() {
            let piece = Piece::from(ch);
            chars_result.push(piece.ch());
            names_result.push(piece.name());
            print_names_result.push(piece.print_name());
        }

        assert_eq!(chars_result, piece_chars);
        assert_eq!(names_result, piece_names);
        assert_eq!(print_names_result, piece_print_names);
    }
}
