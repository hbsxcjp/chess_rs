#![allow(dead_code)]

// use crate::bit_board;
// use crate::bit_constant;
// use std::borrow::Borrow;
// use std::borrow::Borrow;
use crate::coord;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Debug)]
pub struct Move {
    pub id: RefCell<usize>,
    pub before: Weak<Move>,
    pub after: RefCell<Vec<Rc<Move>>>,

    pub coordpair: coord::CoordPair,
    pub remark: RefCell<String>,
}

impl Move {
    pub fn root() -> Rc<Self> {
        Rc::new(Move {
            id: RefCell::new(0),
            before: Weak::new(),
            after: RefCell::new(vec![]),

            coordpair: coord::CoordPair::new(),
            remark: RefCell::new(String::new()),
        })
    }

    pub fn is_root(&self) -> bool {
        self.before.upgrade().is_none()
    }

    pub fn add(self: &Rc<Self>, coordpair: coord::CoordPair, remark: String) -> Rc<Self> {
        let amove = Rc::new(Move {
            id: RefCell::new(0),
            before: Rc::downgrade(self),
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
            result.insert(0, amove.clone());
            amove = amove.before.upgrade().unwrap();
        }

        result
    }

    pub fn to_string(&self, record_type: coord::RecordType) -> String {
        let mut remark = self.remark.borrow().clone();
        if remark.len() > 0 {
            remark = format!("{{{}}}", remark);
        }

        if self.is_root() {
            if remark.len() > 0 {
                remark + "\n"
            } else {
                remark
            }
        } else {
            format!(
                "{:-3}_{:-3}{}{}\n",
                self.before.upgrade().unwrap().id.borrow(),
                self.id.borrow(),
                self.coordpair.to_string(record_type),
                remark
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amove() {
        let root_move = Move::root();

        let from_coord = coord::Coord::from_rowcol(0, 0).unwrap();
        let to_coord = coord::Coord::from_rowcol(0, 2).unwrap();
        let coordpair = coord::CoordPair::from_coord(from_coord, to_coord);
        let remark = String::from("Hello, move.");
        let amove = root_move.add(coordpair, remark);

        assert_eq!("  0_  0[(0,0)->(0,2)]{Hello, move.}\n", amove.to_string(coord::RecordType::Txt));
    }
}
