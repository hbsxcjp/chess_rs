#![allow(dead_code)]

// use crate::bit_board;
use crate::bit_constant;
// use std::borrow::Borrow;
// use std::borrow::Borrow;
// use crate::piece;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Debug)]
pub struct Move {
    id: RefCell<usize>,
    pub before: Weak<Move>,
    pub after: RefCell<Vec<Rc<Move>>>,

    pub coordpair: bit_constant::CoordPair,
    pub remark: RefCell<String>,
}

impl Move {
    pub fn root() -> Rc<Self> {
        Rc::new(Move {
            id: RefCell::new(0),
            before: Weak::new(),
            after: RefCell::new(vec![]),

            coordpair: bit_constant::CoordPair::new(),
            remark: RefCell::new(String::new()),
        })
    }

    pub fn add(self: &Rc<Self>, coordpair: bit_constant::CoordPair, remark: String) -> Rc<Self> {
        let amove = Rc::new(Move {
            id: RefCell::new(self.get_max_id()),
            before: Rc::downgrade(&self),
            after: RefCell::new(vec![]),

            coordpair,
            remark: RefCell::new(remark),
        });

        let step_clone = Rc::clone(&amove);
        self.after.borrow_mut().push(amove);

        step_clone
    }

    pub fn is_root(&self) -> bool {
        self.before.upgrade().is_none()
    }

    // 将root的id字段作为递增储存器
    fn get_max_id(self: &Rc<Self>) -> usize {
        let mut root_move = self.clone();
        while !root_move.is_root() {
            root_move = root_move.before.upgrade().unwrap();
        }

        *root_move.id.borrow_mut() += 1;
        let max_id = *root_move.id.borrow();
        max_id
    }

    fn to_base_string(&self) -> String {
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
            let before = &self.before.upgrade().unwrap();

            format!(
                "{:-3}_{:-3}{}{}\n",
                if before.is_root() {
                    0
                } else {
                    *before.id.borrow()
                },
                *self.id.borrow(),
                self.coordpair.to_string(),
                remark
            )
        }
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
            "  0_  1[(0,0)->(0,2)]{Hello, move.}\n",
            root_move.to_string()
        );
    }
}
