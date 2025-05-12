use std::default;

use bounds::Bound;
use intervals::*;
use num_traits::{Bounded, Num, Zero};
use z3::ast::Int;
// use std::ops::Range;

const MIN: i64 = i64::MIN;
const MAX: i64 = i64::MAX;

// #[derive(PartialEq, Debug)]
// pub struct MyStruct<T>
// where T: PartialOrd+Clone
// {
//     interval: Interval<bounds::Closed<T>, bounds::Closed<T>>, // 使用 Closed 边界，示例中可以根据需求调整
// }

// impl<T> MyStruct<T>
// where T: PartialOrd + Clone {
//     /// 通过 unchecked 函数将 T 转换为 Interval
//     pub fn new(value: T) -> Self {
//         // 这里假设值是闭区间形式 [T, T]
//         MyStruct {
//             interval: Interval::new_unchecked(bounds::Closed(value.clone()), bounds::Closed(value.clone())),
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum RangeType {
    Unknown,
    Regular,
    Empty,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Range<T>
where
    T: PartialOrd + Clone,
{
    pub rtype: RangeType,
    pub range: Closed<T>,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
enum UserType {
    Unknown,
    I32(i32),
    I64(i64),
    Usize(usize),
    Empty,
}

impl<T> Range<T>
where
    T: PartialOrd + Clone + Bounded,
{
    // Parameterized constructor
    pub fn new(lb: T, ub: T, rtype: RangeType) -> Self {
        Self {
            rtype,
            range: Interval::new_unchecked(bounds::Closed(lb), bounds::Closed(ub)),
        }
    }
    pub fn default(default: T) -> Self {
        Self {
            rtype: RangeType::Regular,

            range: Interval::new_unchecked(
                bounds::Closed(T::min_value()),
                bounds::Closed(T::max_value()),
            ),
        }
    }
    // Getter for lower bound
    pub fn get_lower(&self) -> T {
        self.range.left.0.clone()
    }

    // Getter for upper bound
    pub fn get_upper(&self) -> T {
        self.range.right.0.clone()
    }

    // Setter for lower bound
    pub fn set_lower(&mut self, newl: T) {
        self.range.left.0 = newl;
    }

    // Setter for upper bound
    pub fn set_upper(&mut self, newu: T) {
        self.range.right.0 = newu;
    }

    // Check if the range type is unknown
    pub fn is_unknown(&self) -> bool {
        self.rtype == RangeType::Unknown
    }

    // Set the range type to unknown
    pub fn set_unknown(&mut self) {
        self.rtype = RangeType::Unknown;
    }

    // Check if the range type is regular
    pub fn is_regular(&self) -> bool {
        self.rtype == RangeType::Regular
    }

    // Set the range type to regular
    pub fn set_regular(&mut self) {
        self.rtype = RangeType::Regular;
    }

    // Check if the range type is empty
    pub fn is_empty(&self) -> bool {
        self.rtype == RangeType::Empty
    }

    // Set the range type to empty
    pub fn set_empty(&mut self) {
        self.rtype = RangeType::Empty;
    }

    // Check if the range is the maximum range
    // pub fn is_max_range(&self) -> bool {
    //     self.range.lower() == T::min_value() && self.range.upper() == T::max_value()
    // }

    // // Print the range
    // pub fn print(&self) {
    //     println!("Range: [{} - {}]", self.get_lower(), self.get_upper());
    // }

    // // Arithmetic and bitwise operations (example for addition)
    // pub fn add(&self, other: &Range<T>) -> Range<T> {
    //     let lower = self.get_lower() + other.get_lower();
    //     let upper = self.get_upper() + other.get_upper();
    //     Range::with_bounds(lower, upper, RangeType::Regular)
    // }
}

// Implement the comparison operators
