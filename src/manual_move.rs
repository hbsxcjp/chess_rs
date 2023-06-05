#![allow(dead_code)]

use crate::amove;
// use crate::bit_constant;
// use std::borrow::Borrow;
use crate::board;
// use std::cell::RefCell;
use std::rc::Rc;
// use std::rc::Weak;

#[derive(Debug)]
pub struct ManualMove {
    pub board: board::Board,

    pub root_move: Rc<amove::Move>,
    pub current_move: Rc<amove::Move>,
}

impl ManualMove {
    pub fn new(fen: &str) -> Self {
        ManualMove {
            board: board::Board::new(fen),

            root_move: amove::Move::root(),
            current_move: amove::Move::root(),
        }
    }

    pub fn from(fen: &str) -> Self {
        let manual_move = ManualMove {
            board: board::Board::new(fen),

            root_move: amove::Move::root(),
            current_move: amove::Move::root(),
        };

        manual_move
    }

    fn read_from_stream(&self) {}

    pub fn to_string(&self) -> String {
        // format!(
        //     "{:?}.{:?} {} {}",
        //     self.before.upgrade().unwrap(),
        //     self.id,
        //     self.coordpair.to_string(),
        //     self.remark.borrow()
        // )

        String::new()
    }
}
