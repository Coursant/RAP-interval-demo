use rustc_hir::def_id::DefId;
use rustc_middle::{mir::Body, ty::TyCtxt};

use super::ConstraintGraph::ConstraintGraph;

pub struct BodyVisitor<'tcx, T> {
    pub tcx: TyCtxt<'tcx>,
    pub def_id: DefId,
    pub CGT: ConstraintGraph<'tcx, T>,
    pub body: &'tcx Body<'tcx>,
}
impl<'tcx, T> BodyVisitor<'tcx, T> {
    pub fn new(tcx: TyCtxt<'tcx>, def_id: DefId) -> Self {
        let body = tcx.optimized_mir(def_id);
        Self {
            tcx,
            def_id,
            CGT: ConstraintGraph::new(),
            body: body,
        }
    }
    pub fn analysis(&self) {
        self.CGT.build_graph(self.body);
        self.CGT.build_varnodes();
        self.CGT.find_intervals();
    }
}
