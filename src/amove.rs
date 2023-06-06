#![allow(dead_code)]

// use crate::bit_board;
use crate::bit_constant;
// use std::borrow::Borrow;
// use crate::piece;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Debug)]
pub struct Move {
    pub before: Weak<Move>,
    pub after: RefCell<Vec<Rc<Move>>>,

    pub coordpair: bit_constant::CoordPair,
    pub remark: RefCell<String>,
}

impl Move {
    pub fn root() -> Rc<Self> {
        Rc::new(Move {
            before: Weak::new(),
            after: RefCell::new(vec![]),

            coordpair: bit_constant::CoordPair::new(),
            remark: RefCell::new(String::new()),
        })
    }

    pub fn add(self: &Rc<Self>, coordpair: bit_constant::CoordPair, remark: String) -> Rc<Self> {
        let amove = Rc::new(Move {
            before: Rc::downgrade(&self),
            after: RefCell::new(vec![]),

            coordpair,
            remark: RefCell::new(remark),
        });

        let step_clone = Rc::clone(&amove);
        self.after.borrow_mut().push(amove);

        step_clone
    }

    fn to_base_string(&self) -> String {
        format!("{} {}\n", self.coordpair.to_string(), self.remark.borrow())
    }

    pub fn to_string(&self) -> String {
        let mut result = self.to_base_string();
        for after_move in self.after.borrow().iter() {
            result.push_str(&after_move.to_string());
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::bit_constant::CoordPair;

    use super::*;

    #[test]
    fn test_amove() {
        let root_move = Move::root();

        let from_coord = bit_constant::Coord::from_rowcol(0, 0).unwrap();
        let to_coord = bit_constant::Coord::from_rowcol(0, 2).unwrap();
        let coordpair = CoordPair::from_coord(from_coord, to_coord);
        let remark = String::from("Hello, move.");
        root_move.add(coordpair, remark);

        assert_eq!(
            "[(0,0)->(0,0)] \n[(0,0)->(0,2)] Hello, move.\n",
            root_move.to_string()
        );
    }
}
