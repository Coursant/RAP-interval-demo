use super::range::Range;
use num_traits::{Bounded, ToPrimitive};
use rustc_abi::Size;
use rustc_middle::mir::{BasicBlock, Const, Local, LocalDecl, Place, Statement};
use rustc_middle::ty::ScalarInt;
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
pub trait ConstConvert: Sized {
    fn from_const(c: &Const) -> Option<Self>;
}


impl ConstConvert for u32 {
    fn from_const(c: &Const) -> Option<Self> {
        Some(c.try_to_scalar_int().unwrap().to_u32())
    }
}
impl ConstConvert for usize {
    fn from_const(c: &Const) -> Option<Self> {
                let size = Size::from_bits(32);
                // let size = Size::from_bits(std::mem::size_of::<usize>() as u64 * 8);

        c.try_to_bits(size).unwrap().to_usize()

    }
}

#[derive(Debug)]
pub enum IntervalType<'tcx, T: PartialOrd + Clone + Bounded> {
    Basic(BasicInterval<T>),
    Symb(SymbInterval<'tcx, T>), // Using 'static for simplicity, adjust lifetime as needed
}

trait BasicIntervalTrait<T: PartialOrd + Clone + Bounded> {
    // fn get_value_id(&self) -> IntervalId;
    fn get_range(&self) -> &Range<T>;
    fn set_range(&mut self, new_range: Range<T>);
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicInterval<T: PartialOrd + Clone + Bounded> {
    range: Range<T>,
}

impl<T: PartialOrd + Clone + Bounded> BasicInterval<T> {
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
pub struct SymbInterval<'tcx, T: PartialOrd + Clone + Bounded> {
    range: Range<T>,
    symbound: Place<'tcx>,
    predicate: bool,
}

impl<'tcx, T: PartialOrd + Clone + Bounded> SymbInterval<'tcx, T> {
    pub fn new(range: Range<T>, symbound: Place<'tcx>, predicate: bool) -> Self {
        Self {
            range: range,
            symbound,
            predicate,
        }
    }

    pub fn get_operation(&self) -> &bool {
        &self.predicate
    }

    pub fn get_bound(&self) -> &Place<'tcx> {
        &self.symbound
    }

    pub fn fix_intersects(&self, symbound: &Place, sink: &Place) {
        println!(
            "Fixing intersects with bound {:?} and sink {:?}",
            symbound, sink
        );
    }
}

impl<'tcx, T: PartialOrd + Clone + Bounded> BasicIntervalTrait<T> for SymbInterval<'tcx, T> {
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

// #[derive(Debug, Clone)]
// pub struct BasicOp<'tcx, T: PartialOrd + Clone + Bounded> {
//     pub intersect: BasicInterval<T>,
//     pub sink: Place<'tcx>,
//     pub inst: &'tcx Statement<'tcx>,
// }

// impl<'tcx, T: PartialOrd + Clone + Bounded> BasicOp<'tcx, T> {
//     // Constructor for creating a new BasicOp
//     pub fn new(
//         intersect: BasicInterval<T>,
//         sink: Place<'tcx>,
//         inst: &'tcx Statement<'tcx>,
//     ) -> Self {
//         BasicOp {
//             intersect,
//             sink,
//             inst,
//         }
//     }

//     pub fn get_instruction(&self) -> Option<&Statement<'tcx>> {
//         Some(self.inst)
//     }

//     pub fn fix_intersects(&mut self, _v: &VarNode<T>) {}

//     pub fn set_intersect(&mut self, new_intersect: Range<T>) {
//         self.intersect.set_range(new_intersect);
//     }

//     pub fn get_sink(&self) -> Place<'tcx> {
//         self.sink
//     }
//     // Returns the instruction that originated this operation

//     // Returns the target of the operation (sink), mutable version
// }

// // Implement the Operation trait for BasicOp
// impl<'tcx, T: PartialOrd + Clone + Bounded> Operation<T> for BasicOp<'tcx, T> {
//     fn get_value_id(&self) -> u32 {
//         0 // Placeholder implementation
//     }

//     fn eval(&self) -> Range<T> {
//         // Placeholder for evaluating the range
//         Range::default(T::min_value()) // Assuming Range<T> implements Default
//     }

//     fn print(&self, os: &mut dyn fmt::Write) {}
// }
#[derive(Debug)]
pub enum BasicOpKind<'tcx, T: PartialOrd + Clone + Bounded> {
    Unary(UnaryOp<'tcx, T>),
    Binary(BinaryOp<'tcx, T>),
    Essa(EssaOp<'tcx, T>),
    ControlDep(ControlDep<'tcx, T>),
    Phi(PhiOp<'tcx, T>),
    Use(UseOp<'tcx, T>),
}
#[derive(Debug)]
pub struct UseOp<'tcx, T: PartialOrd + Clone + Bounded> {
    pub intersect: BasicInterval<T>,
    pub sink: Place<'tcx>,
    pub inst: &'tcx Statement<'tcx>,
    pub source: Place<'tcx>,
    pub opcode: u32,
}

impl<'tcx, T: PartialOrd + Clone + Bounded> UseOp<'tcx, T> {
    pub fn new(
        intersect: BasicInterval<T>,
        sink: Place<'tcx>,
        inst: &'tcx Statement<'tcx>,
        source: Place<'tcx>,
        opcode: u32,
    ) -> Self {
        Self {
            intersect,
            sink,
            inst,
            source,
            opcode,
        }
    }

    pub fn eval(&self) -> Range<T> {
        // 示例逻辑（用户可自定义）
        self.intersect.range.clone()
    }
}
#[derive(Debug)]
pub struct UnaryOp<'tcx, T: PartialOrd + Clone + Bounded> {
    pub intersect: BasicInterval<T>,
    pub sink: Place<'tcx>,
    pub inst: &'tcx Statement<'tcx>,
    pub source: Place<'tcx>,
    pub opcode: u32,
}

impl<'tcx, T: PartialOrd + Clone + Bounded> UnaryOp<'tcx, T> {
    pub fn new(
        intersect: BasicInterval<T>,
        sink: Place<'tcx>,
        inst: &'tcx Statement<'tcx>,
        source: Place<'tcx>,
        opcode: u32,
    ) -> Self {
        Self {
            intersect,
            sink,
            inst,
            source,
            opcode,
        }
    }

    pub fn eval(&self) -> Range<T> {
        // 示例逻辑（用户可自定义）
        self.intersect.range.clone()
    }
}
#[derive(Debug)]

pub struct EssaOp<'tcx, T: PartialOrd + Clone + Bounded> {
    pub intersect: BasicInterval<T>,
    pub sink: Place<'tcx>,
    pub inst: &'tcx Statement<'tcx>,
    pub source: Place<'tcx>,
    pub opcode: u32,
    pub unresolved: bool,
}

impl<'tcx, T: PartialOrd + Clone + Bounded> EssaOp<'tcx, T> {
    pub fn new(
        intersect: BasicInterval<T>,
        sink: Place<'tcx>,
        inst: &'tcx Statement<'tcx>,
        source: Place<'tcx>,
        opcode: u32,
    ) -> Self {
        Self {
            intersect,
            sink,
            inst,
            source,
            opcode,
            unresolved: true,
        }
    }

    pub fn eval(&self) -> Range<T> {
        self.intersect.range.clone()
    }

    pub fn is_unresolved(&self) -> bool {
        self.unresolved
    }

    pub fn mark_resolved(&mut self) {
        self.unresolved = false;
    }

    pub fn mark_unresolved(&mut self) {
        self.unresolved = true;
    }
}
#[derive(Debug)]

pub struct BinaryOp<'tcx, T: PartialOrd + Clone + Bounded> {
    pub intersect: BasicInterval<T>,
    pub sink: Place<'tcx>,
    pub inst: &'tcx Statement<'tcx>,
    pub source1: Option<Place<'tcx>>,
    pub source2: Option<Place<'tcx>>,
    pub opcode: u32,
}

impl<'tcx, T: PartialOrd + Clone + Bounded> BinaryOp<'tcx, T> {
    pub fn new(
        intersect: BasicInterval<T>,
        sink: Place<'tcx>,
        inst: &'tcx Statement<'tcx>,
        source1: Option<Place<'tcx>>,
        source2: Option<Place<'tcx>>,
        opcode: u32,
    ) -> Self {
        Self {
            intersect,
            sink,
            inst,
            source1,
            source2,
            opcode,
        }
    }

    pub fn eval(&self) -> Range<T> {
        self.intersect.range.clone()
    }
}
#[derive(Debug)]

pub struct PhiOp<'tcx, T: PartialOrd + Clone + Bounded> {
    pub intersect: BasicInterval<T>,
    pub sink: Place<'tcx>,
    pub inst: &'tcx Statement<'tcx>,
    pub sources: Vec<Place<'tcx>>,
    pub opcode: u32,
}

impl<'tcx, T: PartialOrd + Clone + Bounded> PhiOp<'tcx, T> {
    pub fn new(
        intersect: BasicInterval<T>,
        sink: Place<'tcx>,
        inst: &'tcx Statement<'tcx>,
        opcode: u32,
    ) -> Self {
        Self {
            intersect,
            sink,
            inst,
            sources: vec![],
            opcode,
        }
    }

    pub fn add_source(&mut self, src: Place<'tcx>) {
        self.sources.push(src);
    }

    pub fn eval(&self) -> Range<T> {
        self.intersect.range.clone()
    }
}
#[derive(Debug)]

pub struct ControlDep<'tcx, T: PartialOrd + Clone + Bounded> {
    pub intersect: BasicInterval<T>,
    pub sink: Place<'tcx>,
    pub inst: &'tcx Statement<'tcx>,
    pub source: Place<'tcx>,
}

impl<'tcx, T: PartialOrd + Clone + Bounded> ControlDep<'tcx, T> {
    pub fn new(
        intersect: BasicInterval<T>,
        sink: Place<'tcx>,
        inst: &'tcx Statement<'tcx>,
        source: Place<'tcx>,
    ) -> Self {
        Self {
            intersect,
            sink,
            inst,
            source,
        }
    }

    pub fn eval(&self) -> Range<T> {
        self.intersect.range.clone()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarNode<'tcx, T: PartialOrd + Clone + Bounded> {
    // The program variable which is represented.
    v: Place<'tcx>,
    // A Range associated to the variable.
    interval: Range<T>,
    // Used by the crop meet operator.
    abstract_state: char,
}
impl<'tcx, T: PartialOrd + Clone + Bounded> VarNode<'tcx, T> {
    pub fn new(v: Place<'tcx>) -> Self {
        Self {
            v,
            interval: Range::default(T::min_value()),
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
    pub fn get_value(&self) -> Place<'tcx> {
        self.v.clone()
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
pub struct ValueBranchMap<'tcx, T: PartialOrd + Clone + Bounded> {
    v: Place<'tcx>,               // The value associated with the branch
    bb_true: &'tcx BasicBlock,    // True side of the branch
    bb_false: &'tcx BasicBlock,   // False side of the branch
    itv_t: IntervalType<'tcx, T>, // Interval for the true side
    itv_f: IntervalType<'tcx, T>,
}
impl<'tcx, T: PartialOrd + Clone + Bounded> ValueBranchMap<'tcx, T> {
    pub fn new(
        v: Place<'tcx>,
        bb_true: &'tcx BasicBlock,
        bb_false: &'tcx BasicBlock,
        itv_t: IntervalType<'tcx, T>,
        itv_f: IntervalType<'tcx, T>,
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
    pub fn get_itv_t(&self) -> &IntervalType<'tcx, T> {
        &self.itv_t
    }

    /// Get the interval associated with the false side of the branch
    pub fn get_itv_f(&self) -> &IntervalType<'tcx, T> {
        &self.itv_f
    }

    /// Get the value associated with the branch
    pub fn get_v(&self) -> Place<'tcx> {
        self.v
    }

    // pub fn set_itv_t(&mut self, itv: &IntervalType<'tcx, T>) {
    //     self.itv_t = itv;
    // }

    // /// Change the interval associated with the false side of the branch
    // pub fn set_itv_f(&mut self, itv: &IntervalType<'tcx, T>) {
    //     self.itv_f = itv;
    // }

    // pub fn clear(&mut self) {
    //     self.itv_t = Box::new(EmptyInterval::new());
    //     self.itv_f = Box::new(EmptyInterval::new());
    // }
}
// #[derive(Debug, Clone, )]
// pub enum PorSKey<'tcx> {
//     Statement( Statement<'tcx>),
//     Place(Place<'tcx>),
// }
pub type VarNodes<'tcx, T> = HashMap<Place<'tcx>, VarNode<'tcx, T>>;
pub type GenOprs<'tcx, T> = Vec<BasicOpKind<'tcx, T>>;
pub type UseMap<'tcx> = HashMap<Place<'tcx>, HashSet<usize>>;
pub type SymbMap<'tcx> = HashMap<Place<'tcx>, HashSet<usize>>;
pub type DefMap<'tcx> = HashMap<Place<'tcx>, usize>;
pub type ValuesBranchMap<'tcx, T> = HashMap<Place<'tcx>, ValueBranchMap<'tcx, T>>;
// pub type VarNodes<'tcx, T> = HashMap<&'tcx Place<'tcx>, VarNode<'tcx, T>>;
// pub type GenOprs<'tcx, T> = Vec<BasicOp<'tcx, T>>;
// pub type UseMap<'tcx, T> = HashMap<&'tcx Place<'tcx>, Vec<&'tcx BasicOp<'tcx, T>>>;
// pub type SymbMap<'tcx, T> = HashMap<&'tcx Place<'tcx>, Vec<&'tcx BasicOp<'tcx, T>>>;
// pub type DefMap<'tcx, T> = HashMap<&'tcx Place<'tcx>, &'tcx BasicOp<'tcx, T>>;
// pub type ValuesBranchMap<'tcx, T> = HashMap<&'tcx Place<'tcx>, ValueBranchMap<'tcx, T>>;
// pub type ValuesSwitchMap<'tcx, T> = HashMap<&'tcx Place<'tcx>, ValueSwitchMap<'tcx, T>>;
// impl<'tcx, T: fmt::Debug + PartialOrd + Clone + Bounded> fmt::Debug for ValueBranchMap<'tcx, T> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("ValueBranchMap")
//             .field("v", &self.v)
//             .field("bb_false", &self.bb_false)
//             .field("bb_true", &self.bb_true)
//             .finish()
//     }
// }
