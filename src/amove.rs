#![allow(dead_code)]

// use serde_derive::Deserialize;
// use serde_derive::Serialize;

use crate::board;
// use crate::bit_board;
// use crate::bit_constant;
// use std::borrow::Borrow;
// use std::borrow::Borrow;
use crate::coord;
use crate::coord::CoordPair;
use crate::utility;
// use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Debug)] //, Serialize, Deserialize
pub struct Move {
    pub before: Option<Weak<Move>>,
    pub after: RefCell<Vec<Rc<Move>>>,

    pub coordpair: coord::CoordPair,
    pub remark: RefCell<String>,
}

impl Move {
    pub fn root() -> Rc<Self> {
        Rc::new(Move {
            before: None,
            after: RefCell::new(vec![]),

            coordpair: coord::CoordPair::new(),
            remark: RefCell::new(String::new()),
        })
    }

    pub fn is_root(&self) -> bool {
        match self.before {
            Some(_) => false,
            None => true,
        }
    }

    pub fn append(self: &Rc<Self>, coordpair: coord::CoordPair, remark: String) -> Rc<Self> {
        let amove = Rc::new(Move {
            before: Some(Rc::downgrade(self)),
            after: RefCell::new(vec![]),

            coordpair,
            remark: RefCell::new(remark),
        });

        self.after.borrow_mut().push(amove.clone());
        amove
    }

    pub fn before_moves(self: &Rc<Self>) -> Vec<Rc<Move>> {
        let mut result = Vec::new();
        let mut amove = self.clone();
        while !amove.is_root() {
            result.push(amove.clone());
            if let Some(before) = &amove.before {
                amove = before.upgrade().unwrap();
            }
        }
        result.reverse();

        result
    }

    pub fn get_from_input(input: &mut &[u8], is_root: bool) -> (CoordPair, String, usize) {
        let coordpair = if !is_root {
            utility::read_coordpair(input)
        } else {
            CoordPair::new()
        };

        let remark = utility::read_string(input);
        let after_num = utility::read_be_u32(input) as usize;

        (coordpair, remark, after_num)
    }

    pub fn to_output(&self, output: &mut Vec<u8>) {
        if !self.is_root() {
            utility::write_coordpair(output, &self.coordpair);
        }

        utility::write_string(output, self.remark.borrow().as_str());
        utility::write_be_u32(output, self.after.borrow().len() as u32);
    }

    pub fn to_string(&self, record_type: coord::RecordType, board: &board::Board) -> String {
        let coordpair_string = if self.is_root() {
            String::new()
        } else {
            if record_type == coord::RecordType::PgnZh {
                let before = self.before.as_ref().unwrap().upgrade().unwrap().clone();
                let board = board.to_move(&before);
                board.get_zhstr_from_coordpair(&self.coordpair)
            } else {
                self.coordpair.to_string(record_type)
            }
        };

        let remark = if self.remark.borrow().len() > 0 {
            format!("{{{}}}", self.remark.borrow().clone())
        } else {
            String::new()
        };

        let after_num = if self.after.borrow().len() > 0 {
            format!("({})", self.after.borrow().len())
        } else {
            String::new()
        };

        format!("{}{}{}\n", coordpair_string, remark, after_num)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amove() {
        let root_move = Move::root();

        let from_coord = coord::Coord::from(0, 0).unwrap();
        let to_coord = coord::Coord::from(0, 2).unwrap();
        let coordpair = coord::CoordPair::from(from_coord, to_coord);
        let remark = String::from("Hello, move.");
        let amove = root_move.append(coordpair, remark);
        let board = board::Board::new();

        assert_eq!(
            "(0,0)(0,2){Hello, move.}\n",
            amove.to_string(coord::RecordType::Txt, &board)
        );
    }
}
