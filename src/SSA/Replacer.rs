use rustc_index::IndexSlice;
use rustc_index::{bit_set::BitSet, IndexVec};
use rustc_middle::mir::visit::*;
use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, instrument};

use crate::SSA::SSATransformer::SSATransformer;

pub fn fully_moved_locals(ssa: &super::ssa::SsaLocals, body: &Body<'_>) -> BitSet<Local> {
    let mut fully_moved = BitSet::new_filled(body.local_decls.len());

    for (_, rvalue, _) in ssa.assignments(body) {
        let (Rvalue::Use(Operand::Copy(place) | Operand::Move(place))
        | Rvalue::CopyForDeref(place)) = rvalue
        else {
            continue;
        };

        let Some(rhs) = place.as_local() else {
            continue;
        };
        if !ssa.is_ssa(rhs) {
            continue;
        }

        if let Rvalue::Use(Operand::Copy(_)) | Rvalue::CopyForDeref(_) = rvalue {
            fully_moved.remove(rhs);
        }
    }

    ssa.meet_copy_equivalence(&mut fully_moved);

    fully_moved
}

pub struct Replacer<'a, 'tcx> {
    pub(crate) tcx: TyCtxt<'tcx>,
    pub(crate) fully_moved: BitSet<Local>,
    pub(crate) storage_to_remove: BitSet<Local>,
    pub(crate) borrowed_locals: &'a BitSet<Local>,
    pub(crate) copy_classes: &'a IndexSlice<Local, Local>,
    pub(crate) ssatransformer: super::SSATransformer::SSATransformer<'tcx>,
    pub(crate) new_local_collection: HashSet<Local>,
}
impl<'tcx> Replacer<'_, 'tcx> {
    // pub fn print_mir(&self, body: &mut Body<'tcx>) {
    //     let dir_path = "passrunner_mir";
    //     // PassRunner::new(self.tcx);
    //     // 动态生成文件路径
    //     let phi_mir_file_path = format!("{}/after_copy_prop_mir.txt", dir_path);

    //     let mut file2 = File::create(&phi_mir_file_path).unwrap();
    //     let mut w2 = io::BufWriter::new(&mut file2);
    //     let options = PrettyPrintMirOptions::from_cli(self.tcx);
    //     write_mir_fn(self.tcx, body, &mut |_, _| Ok(()), &mut w2, options).unwrap();
    // }
    pub fn insert_phi_statment(&mut self, body: &mut Body<'tcx>) {
        // 初始化所有基本块的 phi 函数集合
        for (block_index, blockdata) in body.basic_blocks.iter_enumerated() {}
        let mut phi_functions: HashMap<BasicBlock, HashSet<Local>> = HashMap::new();
        for bb in body.basic_blocks.indices() {
            phi_functions.insert(bb, HashSet::new());
        }
        let variables: Vec<Local> = self
            .ssatransformer
            .local_assign_blocks
            .iter()
            .filter(|(_, blocks)| blocks.len() >= 2) // 只保留基本块数量大于等于 2 的条目
            .map(|(&local, _)| local) // 提取 Local
            .collect();
        print!("{:?}", variables);
        for var in &variables {
            // 获取变量的定义位置
            if let Some(def_blocks) = self.ssatransformer.local_assign_blocks.get(var) {
                let mut worklist: VecDeque<BasicBlock> = def_blocks.iter().cloned().collect();
                let mut processed: HashSet<BasicBlock> = HashSet::new();

                while let Some(block) = worklist.pop_front() {
                    if let Some(df_blocks) = self.ssatransformer.df.get(&block) {
                        for &df_block in df_blocks {
                            if !processed.contains(&df_block) {
                                phi_functions.get_mut(&df_block).unwrap().insert(*var);
                                processed.insert(df_block);
                                if self.ssatransformer.local_assign_blocks[var].contains(&df_block)
                                {
                                    worklist.push_back(df_block);
                                }
                            }
                        }
                    }
                }
            }
        }

        for (block, vars) in phi_functions {
            for var in vars {
                let decl = body.local_decls[var].clone();
                // let new_var = body.local_decls.push(decl);

                // print!("body.local_decls.len():{:?}\n", body.local_decls.len());
                let predecessors = body.basic_blocks.predecessors()[block].clone();

                // 构造元组元素，使用占位变量
                let mut operands = IndexVec::with_capacity(predecessors.len());
                for _ in 0..predecessors.len() {
                    operands.push(Operand::Copy(Place::from(var)));
                } // 创建 phi 语句
                let phi_stmt = Statement {
                    source_info: SourceInfo::outermost(body.span),
                    kind: StatementKind::Assign(Box::new((
                        Place::from(var), // 左值是变量
                        Rvalue::Aggregate(
                            Box::new(AggregateKind::Tuple), // 元组类型
                            operands,
                        ),
                    ))),
                };
                // let phi = std::ptr::addr_of!(phi_stmt);

                // self.ssatransformer.phi_index.insert(phi, predecessors.len());

                body.basic_blocks_mut()[block]
                    .statements
                    .insert(0, phi_stmt);
            }
        }
    }
    pub fn rename_variables(&mut self, body: &mut Body<'tcx>) {
        // 初始化每个变量的 reachingDef
        for local in body.local_decls.indices() {
            self.ssatransformer.reaching_def.insert(local, None);
        }
        // self.ssatransformer.local_defination_block = Self::map_locals_to_definition_block(&self.body.borrow());
        print!(
            "self.ssatransformer.dom_tree {:?}\n ",
            self.ssatransformer.dom_tree
        );
        print!(
            "local_defination_block after phi {:?}\n ",
            self.ssatransformer.local_defination_block
        );
        print!("dom_tree {:?}\n ", self.ssatransformer.dom_tree);
        let order = SSATransformer::depth_first_search_preorder(
            &self.ssatransformer.dom_tree,
            body.basic_blocks.indices().next().unwrap().clone(),
        );
        // 深度优先先序遍历支配树
        print!("order {:?}\n ", order);
        for bb in order {
            self.process_basic_block(bb, body);
        }
    }

    /// 处理单个基本块
    fn process_basic_block(&mut self, bb: BasicBlock, body: &mut Body<'tcx>) {
        // 获取基本块的可变引用

        self.rename_statement(bb, body);
        self.rename_terminator(bb, body.basic_blocks.as_mut()[bb].terminator_mut());

        // if let Some(terminator) = &mut block.terminator {
        //     self.rename_terminator(terminator);

        let successors: Vec<_> = body.basic_blocks[bb].terminator().successors().collect();
        for succ_bb in successors {
            print!("&&&& process successors {:?}  \n", succ_bb);

            self.process_phi_functions(succ_bb, body, bb);
        }
    }
    fn process_phi_functions(
        &mut self,
        succ_bb: BasicBlock,
        body: &mut Body<'tcx>,
        do_bb: BasicBlock,
    ) {
        for statement in body.basic_blocks.as_mut()[succ_bb].statements.iter_mut() {
            let phi_stmt = statement as *const _; // 获取 statement 的指针作为 key

            if SSATransformer::is_phi_statement(statement) {
                print!(
                    "*******phi statement {:?} in {:?}*******\n",
                    statement, succ_bb
                );

                if let StatementKind::Assign(box (_, rvalue)) = &mut statement.kind {
                    if let Rvalue::Aggregate(_, operands) = rvalue {
                        // 获取或初始化 `phi_index`
                        let operand_count = operands.len();
                        let index = self
                            .ssatransformer
                            .phi_index
                            .entry(phi_stmt)
                            .or_insert(0)
                            .clone();

                        // 仅当 index 在合法范围内时才修改
                        if index < operand_count {
                            print!(
                                "*******phi statement operand  {:?} index {:?}*******\n",
                                operands, index
                            );

                            // self.replace_operand(&mut operands[(index).into()], &succ_bb);s
                            match &mut operands[(index).into()] {
                                Operand::Copy(place) | Operand::Move(place) => {
                                    self.replace_place(place, &do_bb);
                                }
                                _ => {}
                            }
                            *self.ssatransformer.phi_index.entry(phi_stmt).or_insert(0) += 1;
                            // 如果所有 `operand` 都已修改完，移除 `phi_index`
                            // if *index >= operand_count {
                            //     self.ssatransformer.phi_index.remove(&phi_stmt);
                            // }
                        }
                    }
                }

                print!(
                    "*******phi statement after {:?} in {:?}*******\n",
                    statement, succ_bb
                );
            }
        }
    }

    /// 处理后继块中的 φ 函数
    // fn process_phi_functions(&mut self, succ_bb: BasicBlock, body: &mut Body<'tcx>,do_bb: BasicBlock) {
    //     for statement in body.basic_blocks.as_mut()[succ_bb].statements.iter_mut() {
    //         // let rc_stat = Rc::new(RefCell::new(statement));
    //         // self.update_reachinf_def(local, &bb);
    //         let phi = std::ptr::addr_of!(statement);

    //         let is_phi = SSATransformer::is_phi_statement(statement);
    //         if is_phi {
    //             print!("*******phi statment {:?} in {:?}*******\n", statement, succ_bb);
    //             match &mut statement.kind {
    //                 // 1. 赋值语句: 变量使用（右值），变量定义（左值）
    //                 StatementKind::Assign(box (place, rvalue)) => {
    //                     match rvalue {
    //                         Rvalue::Aggregate(_, operands) => {
    //                             for operand in operands.iter_mut() {
    //                                 self.replace_operand(operand, &succ_bb);
    //                                 break;
    //                             }
    //                         }
    //                         _ => {}

    //                     }
    //                     print!(
    //                         "*******phi statment after {:?} in {:?}*******\n",
    //                         statement, succ_bb
    //                     );
    //                 }
    //                 _ => {}
    //             }
    //         }
    //     }

    // }

    pub fn rename_statement(&mut self, bb: BasicBlock, body: &mut Body<'tcx>) {
        for statement in body.basic_blocks.as_mut()[bb].statements.iter_mut() {
            print!("*******statment {:?} in {:?}*******\n", statement, bb);

            // let rc_stat = Rc::new(RefCell::new(statement));
            let is_phi = SSATransformer::is_phi_statement(statement);
            match &mut statement.kind {
                // 1. 赋值语句: 变量使用（右值），变量定义（左值）
                StatementKind::Assign(box (place, rvalue)) => {
                    {
                        if !is_phi {
                            self.replace_rvalue(rvalue, &bb);
                            self.rename_local_def(place, &bb, true);
                        } else {
                            //每个定义生成的变量
                            self.rename_local_def(place, &bb, false);
                        }
                    }
                }
                // 2. FakeRead: 变量使用
                // StatementKind::FakeRead(_, place)
                StatementKind::Deinit(place) | StatementKind::SetDiscriminant { place, .. } => {
                    // let place_mut = unsafe { &mut *(place as *const _ as *mut _) };

                    // self.replace_place(place.as_mut());
                }
                // 3. StorageLive: 变量定义
                StatementKind::StorageLive(local) => {
                    // self.rename_local_def(*local);
                }
                // 4. StorageDead: 变量使用
                StatementKind::StorageDead(local) => {
                    // self.replace_local(local);
                }
                _ => {}
            }
            print!("*******statment after {:?} in {:?}*******\n", statement, bb);
        }
    }

    fn rename_terminator(&mut self, bb: BasicBlock, terminator: &mut Terminator<'tcx>) {
        match &mut terminator.kind {
            // 1. 函数调用: 参数使用，返回值定义
            TerminatorKind::Call {
                args, destination, ..
            } => {
                // for operand in args {
                //     self.replace_operand(operand);
                // }
                // if let Some((place, _)) = destination {
                //     self.rename_def(place);
                // }
            }
            // 2. 断言: 变量使用
            TerminatorKind::Assert { cond, .. } => {
                self.replace_operand(cond, &bb);
            }
            // 3. Drop: 变量使用
            TerminatorKind::Drop { place, .. } => {
                self.replace_place(place, &bb);
            }
            // 4. SwitchInt: 变量使用
            TerminatorKind::SwitchInt { discr, .. } => {
                self.replace_operand(discr, &bb);
            }
            _ => {}
        }
    }

    fn replace_rvalue(&mut self, rvalue: &mut Rvalue<'tcx>, bb: &BasicBlock) {
        match rvalue {
            Rvalue::Use(operand)
            | Rvalue::Repeat(operand, _)
            | Rvalue::UnaryOp(_, operand)
            | Rvalue::Cast(_, operand, _)
            | Rvalue::ShallowInitBox(operand, _) => {
                self.replace_operand(operand, &bb);
            }
            Rvalue::BinaryOp(_, box (lhs, rhs)) | Rvalue::BinaryOp(_, box (lhs, rhs)) => {
                self.replace_operand(lhs, &bb);
                self.replace_operand(rhs, &bb);
            }
            Rvalue::Aggregate(_, operands) => {
                for operand in operands {
                    self.replace_operand(operand, &bb);
                }
            }
            _ => {}
        }
    }

    fn replace_operand(&mut self, operand: &mut Operand<'tcx>, bb: &BasicBlock) {
        // print!("old Operand: {:?} \n", operand);
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                self.replace_place(place, bb);
                // print!("replace_operand: {:?} -> {:?}\n", place.local, place.local);
            }
            _ => {}
        }
        // print!("New Operand: {:?} \n", operand);
    }

    fn replace_place(&mut self, place: &mut Place<'tcx>, bb: &BasicBlock) {
        let old_local = place.local;
        self.update_reachinf_def(&place.local, &bb);
        // print!("old place: {:?} \n", place);

        if let Some(Some(reaching_local)) = self.ssatransformer.reaching_def.get(&place.local) {
            let local = reaching_local.clone();
            *place = Place::from(local);
            print!("replace_place: {:?} -> {:?}\n", old_local, place.local,);
            print!("replaced place: {:?} \n", place);
        } else {
            eprintln!(
                "Warning: place.local {:?} not found in reaching_def",
                place.local
            );
        }
    }

    fn rename_local_def(&mut self, place: &mut Place<'tcx>, bb: &BasicBlock, not_phi: bool) {
        // let old_local = place.as_local().as_mut().unwrap();
        self.update_reachinf_def(&place.local, &bb);
        let Place {
            local: old_local,
            projection: _,
        } = place;
        if self.ssatransformer.skipped.contains(&old_local.as_u32()) && not_phi {
            self.ssatransformer.skipped.remove(&old_local.as_u32());
            self.ssatransformer
                .reaching_def
                .insert(*old_local, Some(*old_local));
            print!(
                "%%%%reaching_def{:?} keep {:?}%%%%\n",
                self.ssatransformer.reaching_def, old_local
            );
            return;
        }
        let new_local = Local::from_u32(self.ssatransformer.local_index);
        self.ssatransformer.local_index += 1;
        print!(
            "fuck!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!{:?}\n",
            new_local
        );
        let _old_local = old_local.clone();
        *place = Place::from(new_local);
        self.ssatransformer
            .local_defination_block
            .insert(new_local.clone(), bb.clone());
        let old_local_reaching = self
            .ssatransformer
            .reaching_def
            .get(&_old_local.clone())
            .unwrap();

        self.ssatransformer
            .reaching_def
            .insert(new_local.clone(), *old_local_reaching);
        self.ssatransformer
            .reaching_def
            .insert(_old_local.clone(), Some(new_local.clone()));

        print!(
            "local_defination_block after  {:?}\n ",
            self.ssatransformer.local_defination_block
        );
        print!(
            "%%%%reaching_def{:?} insert {:?}%%%%\n",
            self.ssatransformer.reaching_def, new_local
        );

        // self.reaching_def
        //     .entry(old_local)
        //     .or_default()
        //     .replace(Some(old_local));
    }

    pub fn dominates_(&self, def_bb: &BasicBlock, bb: &BasicBlock) -> bool {
        // 使用一个集合来追踪所有被 def_bb 支配的基本块
        let mut visited = HashSet::new();

        // 从 def_bb 出发，遍历其子树
        let mut stack = self.ssatransformer.dom_tree.get(def_bb).unwrap().clone();
        while let Some(block) = stack.pop() {
            if !visited.insert(block) {
                continue;
            }

            // 如果当前块是 bb，说明 def_bb 支配了 bb
            if block == *bb {
                return true;
            }

            // 将所有子节点加入栈中，继续遍历
            if let Some(children) = self.ssatransformer.dom_tree.get(&block) {
                stack.extend(children);
            }
        }

        false
    }
    fn update_reachinf_def(&mut self, local: &Local, bb: &BasicBlock) {
        // if self.ssatransformer.reaching_def[local]!= None {
        //     return;
        // }
        let mut r = self.ssatransformer.reaching_def[local];
        let mut dominate_bool = true;
        if r != None {
            let def_bb = self.ssatransformer.local_defination_block[&r.unwrap()];
        }

        while !(r == None || dominate_bool) {
            r = self.ssatransformer.reaching_def[&r.unwrap()];
            if r != None {
                let def_bb = self.ssatransformer.local_defination_block[&r.unwrap()];

                dominate_bool = self.dominates_(&def_bb, bb);
            }
        }

        // if dominate_bool {
        //     print!("%%%% {:?} dominates_  {:?}%%%%\n", def_bb,*bb );

        // }

        if let Some(entry) = self.ssatransformer.reaching_def.get_mut(local) {
            *entry = r.clone();
        }
        print!(
            "%%%%reaching_def{:?} use {:?}%%%%\n",
            self.ssatransformer.reaching_def, local
        );
    }
}

impl<'tcx> MutVisitor<'tcx> for Replacer<'_, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn visit_local(&mut self, local: &mut Local, ctxt: PlaceContext, _: Location) {
        let new_local = self.copy_classes[*local];
        // We must not unify two locals that are borrowed. But this is fine if one is borrowed and
        // the other is not. We chose to check the original local, and not the target. That way, if
        // the original local is borrowed and the target is not, we do not pessimize the whole class.
        if self.borrowed_locals.contains(*local) {
            return;
        }
        match ctxt {
            // Do not modify the local in storage statements.
            PlaceContext::NonUse(NonUseContext::StorageLive | NonUseContext::StorageDead) => {}
            // The local should have been marked as non-SSA.
            PlaceContext::MutatingUse(_) => assert_eq!(*local, new_local),
            // We access the value.
            _ => *local = new_local,
            // _ => *local = new_local,
        }
    }

    fn visit_place(&mut self, place: &mut Place<'tcx>, _: PlaceContext, loc: Location) {
        if let Some(new_projection) = self.process_projection(place.projection, loc) {
            place.projection = self.tcx().mk_place_elems(&new_projection);
        }
        // Any non-mutating use context is ok.
        let ctxt = PlaceContext::NonMutatingUse(NonMutatingUseContext::Copy);
        self.visit_local(&mut place.local, ctxt, loc);
        print!("{:?}", place);
    }

    fn visit_operand(&mut self, operand: &mut Operand<'tcx>, loc: Location) {
        if let Operand::Move(place) = *operand
            // A move out of a projection of a copy is equivalent to a copy of the original
            // projection.
            && !place.is_indirect_first_projection()
            && !self.fully_moved.contains(place.local)
        {
            *operand = Operand::Copy(place);
        }
        self.super_operand(operand, loc);
    }

    fn visit_statement(&mut self, stmt: &mut Statement<'tcx>, loc: Location) {
        // When removing storage statements, we need to remove both (#107511).
        if let StatementKind::StorageLive(l) | StatementKind::StorageDead(l) = stmt.kind
            && self.storage_to_remove.contains(l)
        {
            stmt.make_nop();
            return;
        }

        self.super_statement(stmt, loc);

        // Do not leave tautological assignments around.
        if let StatementKind::Assign(box (lhs, ref rhs)) = stmt.kind
            && let Rvalue::Use(Operand::Copy(rhs) | Operand::Move(rhs)) | Rvalue::CopyForDeref(rhs) =
                *rhs
            && lhs == rhs
        {
            stmt.make_nop();
        }
    }
    fn visit_body_preserves_cfg(&mut self, body: &mut Body<'tcx>) {}
    fn visit_basic_block_data(&mut self, block: BasicBlock, data: &mut BasicBlockData<'tcx>) {
        let BasicBlockData {
            statements,
            terminator,
            is_cleanup: _,
        } = data;
    }
}
