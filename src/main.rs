#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_parens)]
#![allow(non_snake_case)]
#![feature(box_patterns)]
#![feature(rustc_private)]
// tidy-alphabetical-start
#![feature(assert_matches)]
#![feature(const_type_name)]
#![feature(cow_is_borrowed)]
#![feature(decl_macro)]
#![feature(if_let_guard)]
#![feature(impl_trait_in_assoc_type)]
#![feature(let_chains)]
#![feature(map_try_insert)]
#![feature(never_type)]
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
use rustc_hir::def_id::DefId;
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
use std::env;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use RAP_interval_demo::domain::ConstraintGraph::{ConstraintGraph, Nuutila};
use RAP_interval_demo::SSA::{PassRunner::*, SSATransformer::*};

pub struct MyVisitor<'tcx> {
    body_test: HashMap<LocalDefId, Option<bool>>,
    body: &'tcx Body<'tcx>,
}

impl<'tcx> MyVisitor<'tcx> {
    pub fn new(body: &'tcx Body<'tcx>, def_id: LocalDefId) -> MyVisitor<'tcx> {
        let mut body_test = HashMap::new();
        body_test.insert(def_id, None); // 或 Some(true)/Some(false)
        MyVisitor { body_test, body }
    }
}

fn analyze_mir<'tcx>(tcx: TyCtxt<'tcx>, def_id: LocalDefId, ssa_def_id: DefId, essa_def_id: DefId) {
    let mut body = tcx.optimized_mir(def_id).clone();
    {
        let body_mut_ref: &mut Body<'tcx> = unsafe {
            &mut *(&mut body as *mut Body<'tcx>)
        };
        let passrunner = PassRunner::new(tcx);
        passrunner.run_pass(body_mut_ref, ssa_def_id, essa_def_id);
        passrunner.print_diff(body_mut_ref);

        let mut cg: ConstraintGraph<'tcx, i32> = ConstraintGraph::new(essa_def_id, ssa_def_id);
        cg.build_graph(body_mut_ref);
        cg.build_nuutila(false);
        cg.find_intervals();
        cg.print_conponent_vars();
        // let mut cg_usize: ConstraintGraph<'tcx, u32> = ConstraintGraph::new(essa_def_id, ssa_def_id);
        // cg_usize.build_graph(body_mut_ref);
    }
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
                if let Some(ssa_def_id) = tcx.hir().items().find(|id| {
                    let item = tcx.hir().item(*id);
                    item.ident.name.to_string() == "SSAstmt"
                }) {
                    let ssa_def_id = ssa_def_id.owner_id.to_def_id();
                    println!("Found SSAstmt def_id: {ssa_def_id:?}");

                    if let Some(essa_def_id) = tcx.hir().items().find(|id| {
                        let item = tcx.hir().item(*id);
                        item.ident.name.to_string() == "ESSAstmt"
                    }) {
                        let essa_def_id = essa_def_id.owner_id.to_def_id();
                        println!("Found ESSAstmt def_id: {essa_def_id:?}");
                        analyze_mir(tcx, def_id, ssa_def_id, essa_def_id);
                    }
                }
            }
        });
        Compilation::Continue
    }
}

fn main() {
    // 打开 backtrace
    env::set_var("RUST_BACKTRACE", "full");

    // 获取命令行参数，跳过第一个参数（程序名）
    let mut args: Vec<String> = env::args().collect();

    // 检查是否提供了输入文件
    if args.len() < 2 {
        eprintln!("用法: {} <测试文件路径>", args[0]);
        std::process::exit(1);
    }

    // 拿出测试文件路径
    let input_file = args.remove(1);

    // 构建传给 RunCompiler 的参数
    let mut rustc_args = vec![
        "rustc".to_string(),
        input_file,
        "--crate-type=bin".to_string(),
        "-Zalways-encode-mir".to_string(),
    ];

    // 运行编译器
    RunCompiler::new(&rustc_args, &mut MyDataflowCallbacks)
        .run()
        .unwrap();
}
