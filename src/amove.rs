#![allow(dead_code)]

use crate::board;
use crate::coord;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::rc::Weak;
// use serde_derive::Deserialize;
// use serde_derive::Serialize;
// use crate::piece;
// use std::arch::x86_64::_CMP_FALSE_OQ;
// use std::borrow::BorrowMut;
// use std::sync::{Rc, RwLock, Weak};
// use crate::coord::CoordPair;
// use std::borrow::BorrowMut;
// use crate::piece;
// use crate::bit_constant;
// use std::borrow::Borrow;
// use std::borrow::Borrow;
// use crate::common;

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

    pub fn after_len(&self) -> usize {
        match &*self.after.borrow() {
            Some(after) => after.len(),
            None => 0,
        }
    }

    pub fn after(&self) -> Option<Vec<Rc<Self>>> {
        match &*self.after.borrow() {
            Some(after) => Some(after.clone()),
            None => None,
        }
    }

    pub fn remark(&self) -> String {
        match &*self.remark.borrow() {
            Some(remark) => remark.clone(),
            None => String::new(),
        }
    }

    fn converted_remark(remark: String) -> Option<String> {
        match remark.is_empty() {
            false => Some(remark),
            true => None,
        }
    }

    pub fn set_remark(&self, remark: String) {
        *self.remark.borrow_mut() = Self::converted_remark(remark);
    }

    pub fn append(self: &Rc<Move>, coordpair: coord::CoordPair, remark: String) -> Rc<Self> {
        let amove = Rc::new(Self {
            before: Some(Rc::downgrade(self)),
            after: RefCell::new(None),

            coordpair,
            remark: RefCell::new(Self::converted_remark(remark)),
        });

        self.after
            .borrow_mut()
            .get_or_insert(vec![])
            .push(amove.clone());

        amove
    }

    pub fn before(&self) -> Option<Rc<Self>> {
        match &self.before {
            Some(before) => Some(before.upgrade().unwrap()),
            None => None,
        }
    }

    pub fn before_moves(self: &Rc<Self>, contains_self: bool) -> Vec<Rc<Self>> {
        let mut before_moves = Vec::new();
        let mut amove = if contains_self {
            self.clone()
        } else {
            self.before().unwrap()
        };
        while !amove.is_root() {
            before_moves.insert(0, amove.clone());
            amove = amove.before().unwrap();
        }
        // before_moves.reverse();

        before_moves
    }

    pub fn get_all_after_moves(self: &Rc<Self>) -> Vec<Rc<Self>> {
        fn enqueue_after(move_deque: &mut VecDeque<Rc<Move>>, amove: &Rc<Move>) {
            if let Some(after) = amove.after() {
                for bmove in after {
                    move_deque.push_back(bmove);
                }
            }
        }

        let mut all_after_moves = Vec::new();
        let mut move_deque = VecDeque::new();
        enqueue_after(&mut move_deque, self);
        while let Some(amove) = move_deque.pop_front() {
            enqueue_after(&mut move_deque, &amove);
            all_after_moves.push(amove);
        }

        all_after_moves
    }

    pub fn to_string(
        self: &Rc<Self>,
        record_type: coord::RecordType,
        board: &board::Board,
    ) -> String {
        let coordpair_string = if self.is_root() {
            String::new()
        } else {
            if record_type == coord::RecordType::PgnZh {
                if self.is_root() {
                    String::new()
                } else {
                    board
                        .to_move(self, false)
                        .get_zhstr_from_coordpair(&self.coordpair)
                }
            } else {
                self.coordpair.to_string(record_type)
            }
        };

        let remark = match &*self.remark.borrow() {
            Some(remark) => format!("{{{}}}", remark),
            None => String::new(),
        };

        let num = self.after_len();
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
        let board = board::Board::new();
        let amove = root_move.append(coordpair, remark);

        assert_eq!(
            "(0,0)(0,2){Hello, move.}\n",
            amove.to_string(coord::RecordType::Txt, &board)
        );
    }
}
