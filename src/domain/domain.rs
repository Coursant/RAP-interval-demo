use num_traits::Bounded;
use rustc_middle::mir::{BasicBlock, Local, LocalDecl, Place, Statement};
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::fmt;

use super::range::Range;

#[derive(Debug)]
pub enum IntervalType<'a, T: PartialOrd + Clone + Bounded> {
    Basic(BasicInterval<T>),
    Symb(SymbInterval<'a, T>), // Using 'static for simplicity, adjust lifetime as needed
}

trait BasicIntervalTrait<T: PartialOrd + Clone + Bounded> {
    // fn get_value_id(&self) -> IntervalId;
    fn get_range(&self) -> &Range<T>;
    fn set_range(&mut self, new_range: Range<T>);
}

#[derive(Debug, Clone)]
pub struct BasicInterval<T: PartialOrd + Clone> {
    range: Range<T>,
}

impl<T: PartialOrd + Clone> BasicInterval<T> {
    pub fn new(range: Range<T>) -> Self {
        Self { range }
    }
}

impl<T: PartialOrd + Clone + Bounded> BasicIntervalTrait<T> for BasicInterval<T> {
    // fn get_value_id(&self) -> IntervalId {
    //     IntervalId::BasicIntervalId
    // }

    fn get_range(&self) -> &Range<T> {
        &self.range
    }

    fn set_range(&mut self, new_range: Range<T>) {
        self.range = new_range;
        if self.range.get_lower() > self.range.get_upper() {
            self.range.set_empty();
        }
    }
}

#[derive(Debug)]
pub struct SymbInterval<'a, T: PartialOrd + Clone + Bounded> {
    range: Range<T>,
    symbound: &'a Place<'a>,
    predicate: bool,
}

impl<'a, T: PartialOrd + Clone + Bounded> SymbInterval<'a, T> {
    pub fn new(range: Range<T>, symbound: &'a Place<'a>, predicate: bool) -> Self {
        Self {
            range: range,
            symbound,
            predicate,
        }
    }

    pub fn get_operation(&self) -> &bool {
        &self.predicate
    }

    pub fn get_bound(&self) -> &Place<'a> {
        &self.symbound
    }

    pub fn fix_intersects(&self, symbound: &Place, sink: &Place) {
        println!(
            "Fixing intersects with bound {:?} and sink {:?}",
            symbound, sink
        );
    }
}

impl<'a, T: PartialOrd + Clone + Bounded> BasicIntervalTrait<T> for SymbInterval<'a, T> {
    // fn get_value_id(&self) -> IntervalId {
    //     IntervalId::SymbIntervalId
    // }

    fn get_range(&self) -> &Range<T> {
        &self.range
    }

    fn set_range(&mut self, new_range: Range<T>) {
        self.range = new_range;
    }
}

// Define the basic operation trait
pub trait Operation<T: PartialOrd + Clone + Bounded> {
    fn get_value_id(&self) -> u32; // Placeholder for an operation identifier
    fn eval(&self) -> Range<T>; // Method to evaluate the result of the operation
    fn print(&self, os: &mut dyn fmt::Write);
}

// Define the BasicOp struct
pub struct BasicOp<'a, T: PartialOrd + Clone + Bounded> {
    pub intersect: &'a mut BasicInterval<T>, // The range associated with the operation
    pub sink: &'a mut VarNode<'a, T>,        // The target node storing the result
    pub inst: &'a Statement<'a>,             // The instruction that originated this operation
}

impl<'a, T: PartialOrd + Clone + Bounded> BasicOp<'a, T> {
    // Constructor for creating a new BasicOp
    pub fn new(
        intersect: &'a mut BasicInterval<T>,
        sink: &'a mut VarNode<'a, T>,
        inst: &'a Statement<'a>,
    ) -> Self {
        BasicOp {
            intersect,
            sink,
            inst,
        }
    }

    pub fn get_instruction(&self) -> Option<&Statement<'a>> {
        Some(self.inst)
    }

    pub fn fix_intersects(&mut self, _v: &VarNode<T>) {}

    pub fn set_intersect(&mut self, new_intersect: Range<T>) {
        self.intersect.set_range(new_intersect);
    }

    pub fn get_sink(&self) -> &VarNode<'a, T> {
        self.sink
    }
    // Returns the instruction that originated this operation

    // Returns the target of the operation (sink), mutable version
    pub fn get_sink_mut(&mut self) -> &mut VarNode<'a, T> {
        &mut self.sink
    }
}

// Implement the Operation trait for BasicOp
impl<'a, T: PartialOrd + Clone + Bounded> Operation<T> for BasicOp<'a, T> {
    fn get_value_id(&self) -> u32 {
        0 // Placeholder implementation
    }

    fn eval(&self) -> Range<T> {
        // Placeholder for evaluating the range
        Range::default() // Assuming Range<T> implements Default
    }

    fn print(&self, os: &mut dyn fmt::Write) {}
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarNode<'a, T: PartialOrd + Clone + Bounded> {
    // The program variable which is represented.
    v: &'a Place<'a>,
    // A Range associated to the variable.
    interval: Range<T>,
    // Used by the crop meet operator.
    abstract_state: char,
}
impl<'a, T: PartialOrd + Clone + Bounded> VarNode<'a, T> {
    pub fn new(v: &'a Place<'a>) -> Self {
        Self {
            v,
            interval: Range::default(),
            abstract_state: '?',
        }
    }

    /// Initializes the value of the node.
    pub fn init(&mut self, outside: bool) {
        let value = self.get_value();

        // if let Some(ci) = value.as_constant_int() {
        //     let tmp = ci.get_value();
        //     if tmp.bits() < MAX_BIT_INT {
        //         self.set_range(Range::new(
        //             tmp.extend_bits(MAX_BIT_INT),
        //             tmp.extend_bits(MAX_BIT_INT),
        //         ));
        //     } else {
        //         self.set_range(Range::new(tmp, tmp));
        //     }
        // } else {
        //     if !outside {
        //         self.set_range(Range::new(MIN, MAX));
        //     } else {
        //         self.set_range(Range::new(MIN, MAX));
        //     }
        // }
    }

    /// Returns the range of the variable represented by this node.
    pub fn get_range(&self) -> &Range<T> {
        &self.interval
    }

    /// Returns the variable represented by this node.
    pub fn get_value(&self) -> &Place<'a> {
        &self.v
    }

    /// Changes the status of the variable represented by this node.
    pub fn set_range(&mut self, new_interval: Range<T>) {
        self.interval = new_interval;

        // Check if lower bound is greater than upper bound. If it is,
        // set range to empty.
        // if self.interval.get_lower().sgt(self.interval.get_upper()) {
        //     self.interval.set_empty();
        // }
    }

    /// Pretty print.
    pub fn print(&self, os: &mut dyn std::io::Write) {
        // Implementation of pretty printing using the `os` writer.
    }

    pub fn get_abstract_state(&self) -> char {
        self.abstract_state
    }

    /// The possible states are '0', '+', '-', and '?'.
    pub fn store_abstract_state(&mut self) {
        // Implementation of storing the abstract state.
    }
}
#[derive(Debug)]
pub struct ValueBranchMap<'a, T: PartialOrd + Clone + Bounded> {
    v: &'a Place<'a>,           // The value associated with the branch
    bb_true: &'a BasicBlock,    // True side of the branch
    bb_false: &'a BasicBlock,   // False side of the branch
    itv_t: IntervalType<'a, T>, // Interval for the true side
    itv_f: IntervalType<'a, T>,
}
impl<'a, T: PartialOrd + Clone + Bounded> ValueBranchMap<'a, T> {
    pub fn new(
        v: &'a Place<'a>,
        bb_true: &'a BasicBlock,
        bb_false: &'a BasicBlock,
        itv_t: IntervalType<'a, T>,
        itv_f: IntervalType<'a, T>,
    ) -> Self {
        Self {
            v,
            bb_true,
            bb_false,
            itv_t,
            itv_f,
        }
    }

    /// Get the "false side" of the branch
    pub fn get_bb_false(&self) -> &BasicBlock {
        self.bb_false
    }

    /// Get the "true side" of the branch
    pub fn get_bb_true(&self) -> &BasicBlock {
        self.bb_true
    }

    /// Get the interval associated with the true side of the branch
    pub fn get_itv_t(&self) -> &IntervalType<'a, T> {
        &self.itv_t
    }

    /// Get the interval associated with the false side of the branch
    pub fn get_itv_f(&self) -> &IntervalType<'a, T> {
        &self.itv_f
    }

    /// Get the value associated with the branch
    pub fn get_v(&self) -> &Place<'a> {
        self.v
    }

    // pub fn set_itv_t(&mut self, itv: &IntervalType<'a, T>) {
    //     self.itv_t = itv;
    // }

    // /// Change the interval associated with the false side of the branch
    // pub fn set_itv_f(&mut self, itv: &IntervalType<'a, T>) {
    //     self.itv_f = itv;
    // }

    // pub fn clear(&mut self) {
    //     self.itv_t = Box::new(EmptyInterval::new());
    //     self.itv_f = Box::new(EmptyInterval::new());
    // }
}
// #[derive(Debug, Clone, )]
// pub enum PorSKey<'a> {
//     Statement( Statement<'a>),
//     Place(Place<'a>),
// }

pub type VarNodes<'a, T> = HashMap<&'a Place<'a>, VarNode<'a, T>>;
// pub type VarNodes<'a, T> = HashMap<&'a  Place<'a>, VarNode<'a,  T>>;

pub type GenOprs<'a, T> = HashSet<&'a BasicOp<'a, T>>;
pub type UseMap<'a, T> = HashMap<&'a Place<'a>, HashSet<&'a BasicOp<'a, T>>>;
pub type SymbMap<'a, T> = HashMap<&'a Place<'a>, HashSet<&'a BasicOp<'a, T>>>;
pub type DefMap<'a, T> = HashMap<&'a Place<'a>, &'a BasicOp<'a, T>>;
pub type ValuesBranchMap<'a, T> = HashMap<&'a Place<'a>, ValueBranchMap<'a, T>>;
// pub type ValuesSwitchMap<'a, T> = HashMap<&'a Place<'a>, ValueSwitchMap<'a, T>>;
// impl<'a, T: fmt::Debug + PartialOrd + Clone + Bounded> fmt::Debug for ValueBranchMap<'a, T> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("ValueBranchMap")
//             .field("v", &self.v)
//             .field("bb_false", &self.bb_false)
//             .field("bb_true", &self.bb_true)
//             .finish()
//     }
// }
