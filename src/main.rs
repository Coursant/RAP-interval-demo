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
extern crate rustc_codegen_ssa;
extern crate RAP_interval_demo;
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
extern crate tracing;
use rustc_mir_transform::*;

use rustc_data_structures::graph::dominators::Dominators;
// use rustc_mir_transform::ssa::SsaLocals;
// use crate::ssa::SsaLocals;
use rustc_data_structures::fx::FxHashMap;
use rustc_data_structures::graph::{dominators, Predecessors};
use rustc_driver::Compilation;
use rustc_driver::{Callbacks, RunCompiler};
use rustc_hir::def_id::LocalDefId;
use rustc_index::IndexVec;
use rustc_interface::{interface::Compiler, Queries};
use rustc_middle::mir::pretty::*;
use rustc_middle::mir::*;
use rustc_middle::{
    mir::{visit::Visitor, Body, Local, Location},
    ty::TyCtxt,
};
use rustc_mir_transform::*;
use rustc_target::abi::FieldIdx;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use RAP_interval_demo::domain::ConstraintGraph::ConstraintGraph;
use RAP_interval_demo::SSA::{PassRunner::*, SSATransformer::*};

fn analyze_mir<'tcx>(tcx: TyCtxt<'tcx>, def_id: LocalDefId) {

    // let mir_built = tcx.mir_built(def_id);
    // let body = mir_built.borrow();

    let mut body = tcx.optimized_mir(def_id).clone();
    //不许存储body的可变引用
    let body_mut_ref = &mut body;
    let passrunner = PassRunner::new(tcx);
    passrunner.run_pass(body_mut_ref);
    passrunner.print_diff(body_mut_ref);


    let body_clone = tcx.optimized_mir(def_id).clone();


    // let mut ssa: SSATransformer<'tcx> = SSATransformer::new(tcx, def_id);
    // // ssa.insert_phi_statment();
    // ssa.analyze();
    // let mut cg: ConstraintGraph<'tcx, u32> = ConstraintGraph::new(tcx);
    // println!("{:?}", cg.vars);

    // let p =
    //     RAP_interval_demo::domain::ConstraintGraph::ConstraintGraph::<'tcx, u32>::create_random_place(
    //         tcx,
    //     );
    // println!("{:?}", p);

    // // cg.build_graph(&body);

    // println!("{:?}", cg.vars);
    // println!("{:?}", cg.values_branchmap);
}

struct MyDataflowCallbacks;

impl Callbacks for MyDataflowCallbacks {
    fn after_analysis<'tcx>(
        &mut self,
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        let mut tcx = queries.global_ctxt().unwrap();
        tcx.enter(|tcx| {
            // 获取 main 函数对应的LocalDefId，仅做示例
            if let Some(def_id) = tcx
                .hir()
                .body_owners()
                .find(|id| tcx.def_path_str(*id) == "main")
            {
                analyze_mir(tcx, def_id);
            }
        });
        Compilation::Continue
    }
}

// 在main函数中使用rustc_driver手动调用编译过程，并运行回调进行数据流分析
fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");

    let args = vec![
        String::from("rustc"),
        String::from("tests/test1.rs"),
        String::from("--crate-type=bin"),
        String::from("-Zalways-encode-mir"),
    ];

    RunCompiler::new(&args, &mut MyDataflowCallbacks)
        .run()
        .unwrap();
}

// ...existing code...
