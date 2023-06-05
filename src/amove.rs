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
    pub id: RefCell<usize>,

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
            id: RefCell::new(0),

            before: Rc::downgrade(&self),
            after: RefCell::new(vec![]),

            coordpair,
            remark: RefCell::new(remark),
        });

        let step_clone = Rc::clone(&amove);
        self.after.borrow_mut().push(amove);

        step_clone
    }

    pub fn to_string(&self) -> String {
        format!(
            "{:?}.{:?} {} {}",
            self.before.upgrade().unwrap(),
            self.id,
            self.coordpair.to_string(),
            self.remark.borrow()
        )
    }
}
