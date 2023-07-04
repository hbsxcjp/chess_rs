#![allow(dead_code)]

// use serde_derive::Deserialize;
// use serde_derive::Serialize;

use crate::board;
// use crate::bit_board;
// use crate::bit_constant;
// use std::borrow::Borrow;
// use std::borrow::Borrow;
// use crate::common;
use crate::coord;
// use crate::coord::CoordPair;
// use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Debug)] //, Serialize, Deserialize
pub struct Move {
    before: Option<Weak<Move>>,
    after: RefCell<Option<Vec<Rc<Move>>>>,

    pub coordpair: coord::CoordPair,
    remark: RefCell<Option<String>>,
}

impl Move {
    pub fn root() -> Rc<Self> {
        Rc::new(Move {
            before: None,
            after: RefCell::new(None),

            coordpair: coord::CoordPair::new(),
            remark: RefCell::new(None),
        })
    }

    pub fn is_root(&self) -> bool {
        self.before.is_none()
    }

    pub fn before(&self) -> Option<Rc<Self>> {
        match &self.before {
            None => None,
            Some(before) => Some(before.upgrade().unwrap()),
        }
    }

    pub fn after(&self) -> Vec<Rc<Self>> {
        self.after.borrow().clone().unwrap_or(vec![])
    }

    pub fn from_to_index(&self) -> (usize, usize) {
        (
            self.coordpair.from_coord.index(),
            self.coordpair.to_coord.index(),
        )
    }

    pub fn remark(&self) -> String {
        self.remark.borrow().clone().unwrap_or(String::new())
    }

    pub fn set_remark(&self, remark: String) {
        if !remark.is_empty() {
            *self.remark.borrow_mut() = Some(remark);
        }
    }

    pub fn append(self: &Rc<Self>, coordpair: coord::CoordPair, remark: String) -> Rc<Self> {
        let amove = Rc::new(Self {
            before: Some(Rc::downgrade(self)),
            after: RefCell::new(None),

            coordpair,
            remark: RefCell::new(if remark.is_empty() {
                None
            } else {
                Some(remark)
            }),
        });

        self.after
            .borrow_mut()
            .get_or_insert(Vec::new())
            .push(amove.clone());

        amove
    }

    pub fn before_moves(self: &Rc<Self>) -> Vec<Rc<Self>> {
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

        let mut remark = self.remark();
        if !remark.is_empty() {
            remark = format!("{{{}}}", remark);
        }

        let num = self.after().len();
        let after_num = if num > 0 {
            format!("({})", num)
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
