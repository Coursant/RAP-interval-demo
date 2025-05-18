#![feature(box_patterns)]
#![feature(rustc_private)]
// tidy-alphabetical-start
#![feature(assert_matches)]
#![feature(const_type_name)]
#![feature(cow_is_borrowed)]
#![feature(decl_macro)]
#![feature(if_let_guard)]
#![feature(impl_trait_in_assoc_type)]
#![feature(is_sorted)]
#![feature(let_chains)]
#![feature(map_try_insert)]
#![feature(never_type)]
#![feature(option_get_or_insert_default)]
#![feature(round_char_boundary)]
#![feature(try_blocks)]
#![feature(yeet_expr)]
// tidy-alphabetical-end
#[macro_use]
extern crate  rustc_abi;
extern crate rustc_codegen_ssa;
extern crate intervals;
extern crate rustc_const_eval;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_mir_dataflow;
extern crate rustc_mir_transform;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
extern crate stable_mir;
extern crate tracing;
pub mod SSA;
pub mod domain;
