use std::fs::File;
use std::io::{self, Write};

use rustc_index::bit_set::BitSet;
use rustc_index::IndexSlice;
use rustc_middle::mir::pretty::{write_mir_fn, PrettyPrintMirOptions};
use rustc_middle::mir::visit::Visitor;
use rustc_middle::mir::visit::*;
use rustc_middle::mir::visit::*;
use rustc_middle::mir::*;
use rustc_middle::mir::{visit::MutVisitor, Body};
use rustc_middle::ty::TyCtxt;
use tracing::{debug, instrument};

use super::Replacer::*;
use crate::SSA::ssa::SsaLocals;
use crate::SSA::SSATransformer::SSATransformer;
pub struct PassRunner<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> PassRunner<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self { tcx }
    }
    pub fn print_diff(&self, body: &mut Body<'tcx>) {
        let dir_path = "passrunner_mir";
        // PassRunner::new(self.tcx);
        // 动态生成文件路径
        let mir_file_path = format!("{}/before_copy_prop_mir.txt", dir_path);
        let phi_mir_file_path = format!("{}/after_copy_prop_mir.txt", dir_path);
        let mut file = File::create(&mir_file_path).unwrap();
        let mut w = io::BufWriter::new(&mut file);
        write_mir_pretty(self.tcx, None, &mut w).unwrap();
        let mut file2 = File::create(&phi_mir_file_path).unwrap();
        let mut w2 = io::BufWriter::new(&mut file2);
        let options = PrettyPrintMirOptions::from_cli(self.tcx);
        write_mir_fn(self.tcx, body, &mut |_, _| Ok(()), &mut w2, options).unwrap();
    }
    pub fn run_pass(&self, body: &mut Body<'tcx>) {
        debug!(def_id = ?body.source.def_id());

        let param_env = self
            .tcx
            .param_env_reveal_all_normalized(body.source.def_id());
        let ssa = SsaLocals::new(self.tcx, body, param_env);

        let fully_moved = fully_moved_locals(&ssa, body);
        debug!(?fully_moved);

        let mut storage_to_remove = BitSet::new_empty(fully_moved.domain_size());
        for (local, &head) in ssa.copy_classes().iter_enumerated() {
            if local != head {
                storage_to_remove.insert(head);
            }
        }

        let any_replacement = ssa.copy_classes().iter_enumerated().any(|(l, &h)| l != h);

        let ssatransformer =
            SSATransformer::new(self.tcx, body, body.source.def_id().expect_local());
        let mut replacer = Replacer {
            tcx: self.tcx,
            copy_classes: ssa.copy_classes(),
            fully_moved,
            borrowed_locals: ssa.borrowed_locals(),
            storage_to_remove,
            ssatransformer,
        };
        replacer.insert_phi_statment(body);

        let param_env = self
            .tcx
            .param_env_reveal_all_normalized(body.source.def_id());
        let ssa = SsaLocals::new(self.tcx, body, param_env);

        let fully_moved = fully_moved_locals(&ssa, body);
        debug!(?fully_moved);

        let mut storage_to_remove = BitSet::new_empty(fully_moved.domain_size());
        for (local, &head) in ssa.copy_classes().iter_enumerated() {
            if local != head {
                storage_to_remove.insert(head);
            }
        }

        let any_replacement = ssa.copy_classes().iter_enumerated().any(|(l, &h)| l != h);
        replacer.copy_classes = ssa.copy_classes();
        replacer.fully_moved = fully_moved;
        replacer.borrowed_locals = ssa.borrowed_locals();
        replacer.storage_to_remove = storage_to_remove;

        replacer.visit_body_preserves_cfg(body);
        replacer.rename_variables(body);
    }
}
