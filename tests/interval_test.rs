#[cfg(test)]
use intervals::{bounds, Interval};

#[derive(PartialEq, Debug)]
pub struct MyStruct<T>
where
    T: PartialOrd + Clone,
{
    interval: Interval<bounds::Closed<T>, bounds::Closed<T>>, // 使用 Closed 边界，示例中可以根据需求调整
}

impl<T> MyStruct<T>
where
    T: PartialOrd + Clone,
{
    /// 通过 unchecked 函数将 T 转换为 Interval
    pub fn new(value: T) -> Self {
        // 这里假设值是闭区间形式 [T, T]
        MyStruct {
            interval: Interval::new_unchecked(
                bounds::Closed(value.clone()),
                bounds::Closed(value.clone()),
            ),
        }
    }
}

mod tests {
    use intervals::*;
    use RAP_interval_demo::domain::{
        range::{Range, RangeType},
        ConstraintGraph,
    };

    use super::*;

    #[test]
    fn test_interval_creation() {
        let interval = Interval::closed_unchecked(1.0, 5.0);
        assert_eq!(interval.left.0, 1.0);
        assert_eq!(interval.right.0, 5.0);
    }

    #[test]
    fn test_interval_contains() {
        let interval = Interval::closed_unchecked(1.0, 5.0);
        assert!(interval.contains(3.0)); // 3 在区间 [1.0, 5.0] 内
        assert!(!interval.contains(0.0)); // 0 不在区间 [1.0, 5.0] 内
    }

    #[test]
    fn test_interval_intersect() {
        let interval1 = Interval::closed_unchecked(1.0, 5.0);
        let interval2 = Interval::closed_unchecked(4.0, 7.0);

        // 交集为 [4.0, 5.0]
        let intersection = interval1.intersect(interval2);
        assert!(intersection.is_some());
        let intersected = intersection.unwrap();
        assert_eq!(intersected.left.0, 4.0);
        assert_eq!(intersected.right.0, 5.0);
    }

    #[test]
    fn test_interval_union_closure() {
        let interval1 = Interval::closed_unchecked(1.0, 5.0);
        let interval2 = Interval::closed_unchecked(4.0, 7.0);

        // 并集为 [1.0, 7.0]
        let union = interval1.union_closure(interval2);
        assert_eq!(union.left.0, 1.0);
        assert_eq!(union.right.0, 7.0);
    }

    #[test]
    fn test_degenerate_interval() {
        let degenerate = Interval::closed_unchecked(3.0, 3.0);
        assert!(degenerate.is_degenerate()); // 区间 [3.0, 3.0] 是 degenerate
    }

    #[test]
    fn test_unit_interval() {
        // let unit = Interval::unit::<f64>();
        // assert!(unit.contains(0.5)); // 0.5 在区间 [0.0, 1.0] 内
        // assert!(!unit.contains(1.5)); // 1.5 不在区间 [0.0, 1.0] 内
        // let i32_test = Interval::<f64>::unit();

        let lower_bound: i32 = 0; // 左边界
        let upper_bound: i32 = 10; // 右边界

        let i32_test = Interval::closed_unchecked(lower_bound, upper_bound);
        print!("{:?}", i32_test);

        let MyStruct_test = MyStruct::new(lower_bound);
        print!("{:?}", MyStruct_test);
    }
    #[test]
    fn test_range_creation() {
        let range = Range::new(1, 10, RangeType::Regular);
        assert_eq!(range.get_lower(), 1);
        assert_eq!(range.get_upper(), 10);
        assert_eq!(range.rtype, RangeType::Regular);
    }

    #[test]
    fn test_set_lower() {
        let mut range = Range::new(1, 10, RangeType::Regular);
        range.set_lower(5);
        assert_eq!(range.get_lower(), 5);
    }

    #[test]
    fn test_set_upper() {
        let mut range = Range::new(1, 10, RangeType::Regular);
        range.set_upper(15);
        assert_eq!(range.get_upper(), 15);
    }

    #[test]
    fn test_is_unknown() {
        let range = Range::new(1, 10, RangeType::Unknown);
        assert!(range.is_unknown());
    }

    #[test]
    fn test_set_unknown() {
        let mut range = Range::new(1, 10, RangeType::Regular);
        range.set_unknown();
        assert!(range.is_unknown());
    }

    #[test]
    fn test_is_regular() {
        let range = Range::new(1, 10, RangeType::Regular);
        assert!(range.is_regular());
    }

    #[test]
    fn test_set_regular() {
        let mut range = Range::new(1, 10, RangeType::Empty);
        range.set_regular();
        assert!(range.is_regular());
    }

    #[test]
    fn test_is_empty() {
        let range = Range::new(1, 10, RangeType::Empty);
        assert!(range.is_empty());
    }

    #[test]
    fn test_set_empty() {
        let mut range = Range::new(1, 10, RangeType::Regular);
        range.set_empty();
        assert!(range.is_empty());
    }

    #[test]
    fn test_edge_case_max_range() {
        // Assuming there's a function to check if the range is the max range
        let range = Range::new(i64::MIN, i64::MAX, RangeType::Regular);
        assert_eq!(range.get_lower(), i64::MIN);
        assert_eq!(range.get_upper(), i64::MAX);
    }
}
