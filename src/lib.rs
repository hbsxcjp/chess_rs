mod bit_board;
mod bit_constant;
mod bit_effect;
mod board;
mod manual;
mod manual_move;
mod piece;
mod amove;

// pub use crate::piece;

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_piece() {
//         let piece_chars = "_KABNRCPkabnrcp";
//         let mut result = String::new();

//         for ch in piece_chars.chars() {
//             let piece = piece::Piece::from(ch);
//             result.push(piece.ch());
//         }

//         assert_eq!(result, piece_chars);
//     }
// }
