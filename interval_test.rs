// A comprehensive overview of Rust generics usage.

use intervals::bounds::ValidateBounds;

// 1. Generic Functions
fn generic_function<T>(value: T) {
    println!("Value: {:?}", value);
}

// 2. Generic Structs
struct GenericStruct<T> {
    field: T,
}

impl<T> GenericStruct<T> {
    fn new(value: T) -> Self {
        GenericStruct { field: value }
    }
}
impl GenericStruct<i32> {
    fn new(value: i32) -> Self {
        GenericStruct { field: value }
    }
    
}
fn generics_i32_or_t<T>(z: GenericStruct<T>) {
    if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i32>() {
        println!("Value: {:?}", z.field);
    } else {
        println!("Value: {:?}", z.field);
    }
}
// 3. Generic Enums
enum GenericEnum<T> {
    Some(T),
    None,
}

// 4. Generic Traits
trait GenericTrait<T> {
    fn do_something(&self, value: T);
    fn do_not_recommend(&self,value: T) ;
}

struct Implementor;

impl GenericTrait<i32> for Implementor {
    fn do_something(&self, value: i32) {
        println!("Processing: {}", value);
    }
    fn do_not_recommend(&self,value: i64) {
        println!("Processing: {}", value);
    }
}

// 5. Generic Methods
impl GenericStruct<i32> {
    fn specific_method(&self) {
        println!("Field value: {}", self.field);
    }
}

impl<T> GenericStruct<T> {
    fn generic_method<U>(&self, other: U) {
        println!("Generic method called with: {:?}", other);
    }
}

// 6. Type Constraints
fn constrained_function<T: Clone + PartialEq>(item: T) {
    if item.clone() == item {
        println!("Item is equal to its clone.");
    }
}

// 7. Associated Types in Traits
trait AssociatedTypes {
    type Item;
    fn get_item(&self) -> Self::Item;
}

struct AssociatedTypesImpl;

impl AssociatedTypes for AssociatedTypesImpl {
    type Item = String;

    fn get_item(&self) -> Self::Item {
        "Hello".to_string()
    }
}

// 8. Lifetime Parameters
struct LifetimeStruct<'a, T> {
    reference: &'a T,
}

// 9. Const Generics
struct ConstGenericStruct<T, const N: usize> {
    array: [T; N],
}

impl<T, const N: usize> ConstGenericStruct<T, N> {
    fn len(&self) -> usize {
        N
    }
}

// 10. Default Generic Type Parameters
struct DefaultGeneric<T = i32> {
    field: T,
}

// 11. Higher-Kinded Generics (Simulated using traits)
trait HigherKindedTrait<F> {
    fn operate(&self, value: F);
}

// 12. Box with Enums
enum BoxedEnum {
    Variant1(Box<i32>),
    Variant2(Box<String>),
}

impl BoxedEnum {
    fn describe(&self) {
        match self {
            BoxedEnum::Variant1(value) => println!("Variant1 with value: {}", value),
            BoxedEnum::Variant2(value) => println!("Variant2 with value: {}", value),
        }
    }
}

// Usage examples
fn main() {
    generics_i32_or_t(GenericStruct { field: 10 });
    generics_i32_or_t( GenericStruct { field: "10" });
    // 1. Generic Functions
    generic_function(42);

    // 2. Generic Structs
    let gs = GenericStruct::new("Hello");
    println!("GenericStruct field: {:?}", gs.field);

    // 3. Generic Enums
    let ge: GenericEnum<i32> = GenericEnum::Some(10);
    if let GenericEnum::Some(value) = ge {
        println!("Enum value: {}", value);
    }

    // 4. Generic Traits
    // let impl = Implementor;
    // impl.do_something(100);

    // 5. Generic Methods
    let gs_int = GenericStruct { field: 10 };
    gs_int.specific_method();
    gs_int.generic_method("Extra data");

    // 6. Type Constraints
    constrained_function("Test");

    // 7. Associated Types in Traits
    let at_impl = AssociatedTypesImpl;
    println!("Associated type value: {}", at_impl.get_item());

    // 8. Lifetime Parameters
    let data = 42;
    let ls = LifetimeStruct { reference: &data };
    println!("LifetimeStruct reference: {}", ls.reference);

    // 9. Const Generics
    let cg = ConstGenericStruct { array: [1, 2, 3, 4] };
    println!("Array length: {}", cg.len());

    // 10. Default Generic Type Parameters
    let dg: DefaultGeneric = DefaultGeneric { field: 5 };
    println!("DefaultGeneric field: {}", dg.field);

    // 12. Box with Enums
    let boxed1 = BoxedEnum::Variant1(Box::new(123));
    let boxed2 = BoxedEnum::Variant2(Box::new("Hello".to_string()));

    boxed1.describe();
    boxed2.describe();
}
