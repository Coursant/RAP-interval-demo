
pub mod BodyVisitor;
pub mod ConstraintGraph;
pub mod domain;
pub mod range;
pub mod interval_test;

use rustc_hir::def::DefKind;
use rustc_hir::def_id::DefId;
use rustc_middle::bug;
use rustc_middle::mir::interpret::{InterpResult, Scalar};
use rustc_middle::mir::visit::{MutVisitor, PlaceContext, Visitor};
use rustc_middle::mir::LocalDecls;
use rustc_middle::mir::*;
use rustc_middle::ty::layout::{HasParamEnv, LayoutOf};
use rustc_middle::ty::{self, Ty, TyCtxt};
pub struct IntervalAnalysis<'tcx> {
    map: Map,
    tcx: TyCtxt<'tcx>,
    body: &'tcx Body<'tcx>,
    local_decls: &'tcx LocalDecls<'tcx>,
    def_id: DefId,
    graph: ConstraintGraph,
    // ecx: InterpCx<'tcx, DummyMachine>,
    // param_env: ty::ParamEnv<'tcx>,
}
impl<'tcx> IntervalAnalysis<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, def_id: DefId) -> Self {
        let body = tcx.optimized_mir(def_id);
        Self {
            map,
            tcx,
            body: body,
            local_decls: &body.local_decls,
            def_id,
            graph: ConstraintGraph::new(def_id, body.arg_count, body.local_decls.len()),
        }
    }
}
