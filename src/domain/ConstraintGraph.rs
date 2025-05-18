use super::{domain::*, range::RangeType, range::*};

use num_traits::Bounded;
use rand::Rng;
use rustc_hir::{def, def_id::DefId};
use rustc_index::IndexVec;
use rustc_middle::{
    mir::*,
    ty::{self, print, ScalarInt, TyCtxt},
};
use rustc_mir_transform::*;
use rustc_span::sym::var;
use rustc_target::abi::FieldIdx;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
};
pub struct ConstraintGraph<'tcx, T: PartialOrd + Clone + Bounded + Debug> {
    // Protected fields
    pub vars: VarNodes<'tcx, T>, // The variables of the source program
    pub oprs: GenOprs<'tcx, T>,  // The operations of the source program

    // func: Option<Function>,             // Save the last Function analyzed
    pub defmap: DefMap<'tcx>, // Map from variables to the operations that define them
    pub usemap: UseMap<'tcx>, // Map from variables to operations where variables are used
    pub symbmap: SymbMap<'tcx>, // Map from variables to operations where they appear as bounds
    pub values_branchmap: ValuesBranchMap<'tcx, T>, // Store intervals, basic blocks, and branches
    // values_switchmap: ValuesSwitchMap<'tcx, T>, // Store intervals for switch branches
    constant_vector: Vec<T>, // Vector for constants from an SCC

    pub inst_rand_place_set: Vec<Place<'tcx>>,
    pub essa: DefId,
    pub ssa: DefId,
}

impl<'tcx, T> ConstraintGraph<'tcx, T>
where
    T: PartialOrd + Clone + Bounded + ConstConvert + Debug,
    {
    pub fn convert_const(&self, c: &Const) -> Option<T> {
        T::from_const(c)
    }

    pub fn new(essa: DefId, ssa: DefId) -> Self {
        Self {
            vars: VarNodes::new(),
            oprs: GenOprs::new(),
            // func: None,
            defmap: DefMap::new(),
            usemap: UseMap::new(),
            symbmap: SymbMap::new(),
            values_branchmap: ValuesBranchMap::new(),
            // values_switchmap: ValuesSwitchMap::new(),
            constant_vector: Vec::new(),
            inst_rand_place_set: Vec::new(),
            essa,
            ssa,
        }
    }
    //     fn create_random_place() -> Place<'tcx> {
    //     // 随机生成一个新的 Local 值
    //     let mut rng = rand::rng();
    //     let random_local = Local::from_usize(rng.random_range(0..100));

    //     // 创建一个新的 Place 值，使用随机生成的 Local 和空的投影列表
    //     let place = Place {
    //         local: random_local,
    //         projection: ,
    //     };

    //     place
    // }
    pub fn create_random_place(&mut self) -> Place<'tcx> {
        // 随机生成一个新的 Local 值
        let mut rng = rand::rng();
        let random_local = Local::from_usize(rng.random_range(10000..100000)); // 假设 Local 的范围是 0 到 99

        // 创建一个新的 Place 值，使用随机生成的 Local 和空的投影列表
        let place = Place {
            local: random_local,
            projection: ty::List::empty(),
        };
        self.inst_rand_place_set.push(place);
        place
    }
    pub fn add_varnode(&mut self, v: Place<'tcx>) -> &VarNode<'tcx, T> {
        // 如果变量已存在，则直接返回

        // 插入新的 VarNode
        let node = VarNode::new(v);
        // 先进行不可变借用，确保没有冲突
        let node_ref: &mut VarNode<'tcx, T> = self.vars.entry(v).or_insert(node);

        // 确保 usemap 也更新
        self.usemap.entry(v).or_insert(HashSet::new());

        node_ref
    }
    pub fn add_varnode_inst(&mut self, inst: &'tcx Statement<'tcx>) -> Place<'tcx> {
        let inst_rand_place: Place<'tcx> = self.create_random_place();
        // let place_ref: &'tcx Place<'tcx> = self.inst_rand_place_set.last().unwrap();

        // let place_ref = unsafe {
        //     // 强制转换为更长的生命周期
        //     &*(self.inst_rand_place_set.last().unwrap() as *const Place<'tcx>)
        // };
        let node = VarNode::new(inst_rand_place);
        let node_ref = self.vars.entry(inst_rand_place).or_insert(node);

        // 确保 usemap 也更新
        self.usemap.entry(inst_rand_place).or_insert(HashSet::new());

        inst_rand_place
    }
    // pub fn get_oprs(&self) -> &GenOprs {
    //     &self.oprs
    // }

    // pub fn get_defmap(&self) -> &DefMap {
    //     &self.defmap
    // }

    // pub fn get_usemap(&self) -> &UseMap {
    //     &self.usemap
    // }

    // pub fn build_graph(&self, body: &Body) -> ConstraintGraph {
    //     let mut graph = ConstraintGraph::new();
    //     let basic_blocks = &body.basic_blocks;
    //     for basic_block_data in basic_blocks.iter() {
    //         for statement in basic_block_data.statements.iter() {
    //             graph.add_stat_to_graph(&statement.kind);
    //         }
    //         if let Some(terminator) = &basic_block_data.terminator {
    //             graph.add_terminator_to_graph(&terminator.kind);
    //         }
    //     }
    //     graph
    // }
    pub fn build_graph(&mut self, body: &'tcx Body<'tcx>) {
        print!("====Building graph====\n");
        self.build_value_maps(body);
        print!("varnodes{:?}\n", self.vars);
        print!("vbm{:?}\n", self.values_branchmap);

        print!("====build_operations====\n");

        for block in body.basic_blocks.indices() {
            let block_data = &body[block];
            // Traverse statements

            for statement in block_data.statements.iter() {
                self.build_operations(statement);
            }
        }

        print!("varnodes{:?}\n", self.vars);
        print!("oprs{:?}\n", self.oprs);
        print!("defmap{:?}\n", self.defmap);
        print!("usemap{:?}\n", self.usemap);
        print!("end\n");
    }

    pub fn build_value_maps(&mut self, body: &'tcx Body<'tcx>) {
        for bb in body.basic_blocks.indices() {
            let block_data = &body[bb];
            if let Some(terminator) = &block_data.terminator {
                match &terminator.kind {
                    TerminatorKind::SwitchInt { discr, targets } => {
                        print!("SwitchIntblock{:?}\n", bb);
                        self.build_value_branch_map(body, discr, targets, block_data);
                    }
                    TerminatorKind::Goto { target } => {
                        // self.build_value_goto_map(block_index, *target);
                    }
                    _ => {
                        // println!(
                        //     "BasicBlock {:?} has an unsupported terminator: {:?}",
                        //     block_index, terminator.kind
                        // );
                    }
                }
            }
        }
        // print!("value_branchmap{:?}\n", self.values_branchmap);
        // print!("varnodes{:?}\n,", self.vars);
    }

    pub fn build_value_branch_map(
        &mut self,
        body: &Body<'tcx>,
        discr: &'tcx Operand<'tcx>,
        targets: &'tcx SwitchTargets,
        block: &'tcx BasicBlockData<'tcx>,
    ) {
        // let place1: &Place<'tcx>;

        if let Operand::Copy(place) | Operand::Move(place) = discr {
            if let Some((op1, op2, cmp_op)) = self.extract_condition(place, block) {
                let const_op1 = op1.constant();
                let const_op2 = op2.constant();

                match (const_op1, const_op2) {
                    (Some(c1), Some(c2)) => {}
                    (Some(c), None) | (None, Some(c)) => {
                        let const_in_left: bool;
                        let variable: Place<'tcx>;
                        if const_op1.is_some() {
                            const_in_left = true;
                            variable = match op2 {
                                Operand::Copy(p) | Operand::Move(p) => p,
                                _ => panic!("Expected a place"),
                            };
                        } else {
                            const_in_left = false;
                            variable = match op1 {
                                Operand::Copy(p) | Operand::Move(p) => p,
                                _ => panic!("Expected a place"),
                            };
                        }
                        // 此处应根据T进行选取，设定为scalarInt
                        self.add_varnode(variable.clone());
                        print!("add_vbm_varnode{:?}\n", variable.clone());

                        // let value = c.const_.try_to_scalar_int().unwrap();
                        let value  = self.convert_const(&c.const_).unwrap();
                        let const_range = Range::new(
                            value.clone(),
                            value.clone(),
                            RangeType::Regular,
                        );

                        let true_range = self.apply_comparison(
                            value.clone(),
                            cmp_op,
                            true,
                            const_in_left,
                        );
                        let false_range = self.apply_comparison(
                            value.clone(),
                            cmp_op,
                            false,
                            const_in_left,
                        );
                        let target_vec = targets.all_targets();
                        let vbm = ValueBranchMap::new(
                            variable,
                            &target_vec[0],
                            &target_vec[1],
                            IntervalType::Basic(BasicInterval::new(true_range)),
                            IntervalType::Basic(BasicInterval::new(false_range)),
                        );
                        self.values_branchmap.insert(place.clone(), vbm);
                    }
                    (None, None) => {
                        // 两个变量之间的比较

                        let CR = Range::new(T::min_value(), T::max_value(), RangeType::Regular);

                        let p1 = match op1 {
                            Operand::Copy(p) | Operand::Move(p) => p,
                            _ => panic!("Expected a place"),
                        };
                        let p2 = match op2 {
                            Operand::Copy(p) | Operand::Move(p) => p,
                            _ => panic!("Expected a place"),
                        };
                        let target_vec = targets.all_targets();
                        self.add_varnode(p1.clone());
                        print!("add_vbm_varnode{:?}\n", p1.clone());

                        self.add_varnode(p2.clone());
                        print!("add_vbm_varnode{:?}\n", p2.clone());

                        let STOp1 = IntervalType::Symb(SymbInterval::new(CR.clone(), p2, true));
                        let SFOp1 = IntervalType::Symb(SymbInterval::new(CR.clone(), p2, false));
                        let STOp2 = IntervalType::Symb(SymbInterval::new(CR.clone(), p1, true));
                        let SFOp2 = IntervalType::Symb(SymbInterval::new(CR.clone(), p1, false));
                        let vbm_1 =
                            ValueBranchMap::new(p1, &target_vec[0], &target_vec[1], STOp1, SFOp1);
                        let vbm_2 =
                            ValueBranchMap::new(p2, &target_vec[0], &target_vec[1], STOp2, SFOp2);
                        self.values_branchmap.insert(p1, vbm_1);
                        self.values_branchmap.insert(p2, vbm_2);
                    }
                }
            };
        }
    }

    fn extract_condition(
        &self,
        place: &Place<'tcx>,
        switch_block: &BasicBlockData<'tcx>,
    ) -> Option<(Operand<'tcx>, Operand<'tcx>, BinOp)> {
        for stmt in &switch_block.statements {
            if let StatementKind::Assign(box (lhs, Rvalue::BinaryOp(bin_op, box (op1, op2)))) =
                &stmt.kind
            {
                if lhs == place {
                    let mut return_op1: &Operand<'tcx> = &op1;
                    let mut return_op2: &Operand<'tcx> = &op2;
                    for stmt_original in &switch_block.statements {
                        if let StatementKind::Assign(box (lhs, Rvalue::Use(OP1))) =
                            &stmt_original.kind
                        {
                            if lhs.clone() == op1.place().unwrap() {
                                return_op1 = OP1;
                            }
                        }
                    }
                    if op2.constant().is_none() {
                        for stmt_original in &switch_block.statements {
                            if let StatementKind::Assign(box (lhs, Rvalue::Use(OP2))) =
                                &stmt_original.kind
                            {
                                if lhs.clone() == op2.place().unwrap() {
                                    return_op2 = OP2;
                                }
                            }
                        }
                    }
                    return Some((return_op1.clone(), return_op2.clone(), *bin_op));
                }
            }
        }
        None
    }
    // pub fn calculate_ranges(
    //     &self,
    //     op1: &Operand<'tcx>,
    //     op2: &Operand<'tcx>,
    //     cmp_op: BinOp,
    // ) -> (Option<(i128, i128)>, Option<(i128, i128)>) {
    //     // 检查操作数是否为常量
    //     let const_op1 = op1.constant();
    //     let const_op2 = op2.constant();

    //     match (const_op1, const_op2) {
    //         (Some(c1), Some(c2)) => {}
    //         (Some(c), None) | (None, Some(c)) => {
    //             let const_in_left: bool;
    //             if const_op1.is_some() {
    //                 const_in_left = true;
    //             } else {
    //                 const_in_left = false;
    //             }
    //             // 此处应根据T进行选取，设定为scalarInt
    //             let const_range = Range::new(c.const_.try_to_scalar().unwrap());

    //             let true_range = self.apply_comparison(const_range, cmp_op, true, const_in_left);
    //             let false_range = self.apply_comparison(const_range, cmp_op, false, const_in_left);

    //             (true_range, false_range)
    //         }
    //         (None, None) => {
    //             // 两个变量之间的比较
    //             let variable_range1 = Range::new(UserType::new());
    //             let variable_range2 = Range::new(UserType::new());
    //             let true_range =
    //                 self.apply_comparison(variable_range1, variable_range2, cmp_op, true);
    //             let false_range =
    //                 self.apply_comparison(variable_range1, variable_range2, cmp_op, false);
    //             (true_range, false_range)
    //         }
    //     }
    // }

    fn apply_comparison<U: PartialOrd + Clone + Bounded>(
        &self,
        constant: U,
        cmp_op: BinOp,
        is_true_branch: bool,
        const_in_left: bool,
    ) -> Range<U> {
        match cmp_op {
            BinOp::Lt => {
                if is_true_branch ^ const_in_left {
                    Range::new(U::min_value(), constant, RangeType::Regular)
                } else {
                    Range::new(constant, U::max_value(), RangeType::Regular)
                }
            }

            BinOp::Le => {
                if is_true_branch ^ const_in_left {
                    Range::new(U::min_value(), constant, RangeType::Regular)
                } else {
                    Range::new(constant, U::max_value(), RangeType::Regular)
                }
            }

            BinOp::Gt => {
                if is_true_branch ^ const_in_left {
                    Range::new(U::min_value(), constant, RangeType::Regular)
                } else {
                    Range::new(constant, U::max_value(), RangeType::Regular)
                }
            }

            BinOp::Ge => {
                if is_true_branch ^ const_in_left {
                    Range::new(U::min_value(), constant, RangeType::Regular)
                } else {
                    Range::new(constant, U::max_value(), RangeType::Regular)
                }
            }

            BinOp::Eq => {
                if is_true_branch ^ const_in_left {
                    Range::new(U::min_value(), constant, RangeType::Regular)
                } else {
                    Range::new(constant, U::max_value(), RangeType::Regular)
                }
            }

            _ => Range::new(constant.clone(), constant.clone(), RangeType::Empty),
        }
    }

    fn build_value_goto_map(&self, block_index: BasicBlock, target: BasicBlock) {
        println!(
            "Building value map for Goto in block {:?} targeting block {:?}",
            block_index, target
        );
        // 在这里实现具体的 Goto 处理逻辑
    }
    pub fn build_varnodes(&mut self) {
        // Builds VarNodes
        for (name, node) in self.vars.iter_mut() {
            let is_undefined = !self.defmap.contains_key(name);
            node.init(is_undefined);
        }
    }
    pub fn build_operations(&mut self, inst: &'tcx Statement<'tcx>) {
        // Handle binary instructions
        if let StatementKind::Assign(box (place, rvalue)) = &inst.kind {
            match rvalue {
                Rvalue::BinaryOp(op, box (op1, op2)) => {
                    match op {
                        // 加减乘除和取余（含 unchecked 和 overflow 版本）
                        BinOp::Add
                        | BinOp::Sub
                        | BinOp::Mul
                        | BinOp::Div
                        | BinOp::Rem
                        | BinOp::AddUnchecked => {
                            self.add_binary_op(inst, op1, op2);
                        }
                        BinOp::AddWithOverflow => {
                            self.add_binary_op(inst, op1, op2);
                        }
                        BinOp::SubUnchecked => {
                            self.add_binary_op(inst, op1, op2);
                        }
                        BinOp::SubWithOverflow => {
                            self.add_binary_op(inst, op1, op2);
                        }
                        BinOp::MulUnchecked => {
                            self.add_binary_op(inst, op1, op2);
                        }
                        BinOp::MulWithOverflow => {
                            self.add_binary_op(inst, op1, op2);
                        }

                        // 其他运算
                        _ => {}
                    }
                }
                Rvalue::UnaryOp(UnOp, op) => {
                    self.add_unary_op(inst, op);
                }
                Rvalue::Aggregate(kind, operends) => {
                    // 处理聚合类型的 Rvalue
                    match **kind {
                        AggregateKind::Adt(def_id, _, _, _, _) => {
                            if def_id == self.essa {
                                self.add_essa_op(inst, operends);
                                // println!("Adt{:?}\n", operends);
                            }
                            if def_id == self.ssa {
                                self.add_ssa_op(inst, operends);
                                // println!("Adt{:?}\n", operends);
                            }
                        }
                        _ => {}
                    }
                }
                Rvalue::Use(operend) => {
                    // 处理使用操作数的 Rvalue
                    match operend {
                        Operand::Copy(place) | Operand::Move(place) => {

                            self.add_use_op(inst, operend);

                        }
                        Operand::Constant(_) => {
                            // 处理常量操作数
                            // println!("Constant{:?}\n", operend);
                        }
                    }
                }
                _ => {
                    // 处理其他类型的 Rvalue
                    // println!("Unsupported Rvalue: {:?}", rvalue);
                }
            }
        }
    }
    fn add_ssa_op(
        &mut self,
        inst: &'tcx Statement<'tcx>,
        operands: &IndexVec<FieldIdx, Operand<'tcx>>,
    ) {
        print!("ssa_op{:?}\n", inst);
        self.add_varnode_inst(inst);

        let sink = self.add_varnode_inst(inst);
        let BI: BasicInterval<T> = BasicInterval::new(Range::default(T::min_value()));
        let mut phiop = PhiOp::new(BI, sink, inst, 0);
                let bop_index = self.oprs.len();

        for i in 0..operands.len() {
            let source = match operands[i.into()] {
                Operand::Copy(place) | Operand::Move(place) => Some(place.clone()),
                _ => None, // 不处理 Constant
            };
            if let Some(source) = source {
                self.add_varnode(source.clone());
                phiop.add_source(source.clone());
                println!("addvar_in_ssa_op{:?}\n", source);
                            self.usemap
                .entry(source)
                .or_default()
                .insert(bop_index);
            }
        }
        // Insert the operation in the graph.

        self.oprs.push(BasicOpKind::Phi(phiop));

        // Insert this definition in defmap

        self.defmap.insert(sink, bop_index);
    }
    fn add_use_op(
        &mut self,
        inst: &'tcx Statement<'tcx>, op: &'tcx Operand<'tcx>
    ) {
        print!("use_op{:?}\n", inst);
        self.add_varnode_inst(inst);

        let sink = self.add_varnode_inst(inst);

        let BI: BasicInterval<T> = BasicInterval::new(Range::default(T::min_value()));
        let source = match op {
            Operand::Copy(place) | Operand::Move(place) => Some(place.clone()),
            _ => None, // 不处理 Constant
        };
        println!("addvar_in_use_op{:?}\n", source.unwrap());

        let essaop = EssaOp::new(BI, sink, inst, source.unwrap(), 0);
        // Insert the operation in the graph.
        let bop_index = self.oprs.len();

        self.oprs.push(BasicOpKind::Essa(essaop));
        // Insert this definition in defmap

        self.defmap.insert(sink, bop_index);
    }
    fn add_essa_op(
        &mut self,
        inst: &'tcx Statement<'tcx>,
        operands: &IndexVec<FieldIdx, Operand<'tcx>>,
    ) {
        print!("essa_op{:?}\n", inst);
        self.add_varnode_inst(inst);

        let sink = self.add_varnode_inst(inst);

        let BI: BasicInterval<T> = BasicInterval::new(Range::default(T::min_value()));
        let loc_1: usize = 0;
        let source = match operands[loc_1.into()] {
            Operand::Copy(place) | Operand::Move(place) => Some(place.clone()),
            _ => None, // 不处理 Constant
        };
        println!("addvar_in_essa_op{:?}\n", source.unwrap());

        let essaop = EssaOp::new(BI, sink, inst, source.unwrap(), 0);
        // Insert the operation in the graph.
        let bop_index = self.oprs.len();

        self.oprs.push(BasicOpKind::Essa(essaop));
        // Insert this definition in defmap

        self.defmap.insert(sink, bop_index);
    }
    fn add_unary_op(&mut self, inst: &'tcx Statement<'tcx>, op: &'tcx Operand<'tcx>) {
        print!("unary_op{:?}\n", inst);
        self.add_varnode_inst(inst);

        let sink = self.add_varnode_inst(inst);

        let BI: BasicInterval<T> = BasicInterval::new(Range::default(T::min_value()));
        let loc_1: usize = 0;
        let source = match op {
            Operand::Copy(place) | Operand::Move(place) => Some(place.clone()),
            _ => None, // 不处理 Constant
        };
        println!("addvar_in_unary_op{:?}\n", source.unwrap());

        let essaop = EssaOp::new(BI, sink, inst, source.unwrap(), 0);
        // Insert the operation in the graph.
        let bop_index = self.oprs.len();

        self.oprs.push(BasicOpKind::Essa(essaop));
        // Insert this definition in defmap

        self.defmap.insert(sink, bop_index);
    }

    fn add_binary_op(
        &mut self,
        inst: &'tcx Statement<'tcx>,
        op1: &'tcx Operand<'tcx>,
        op2: &'tcx Operand<'tcx>,
    ) {
        print!("binary_op{:?}\n", inst);
        self.add_varnode_inst(inst);

        let sink = self.add_varnode_inst(inst);

        let source1_place = match op1 {
            Operand::Copy(place) | Operand::Move(place) => {
                self.add_varnode(place.clone()); // 构建 VarNode 图
                        println!("addvar_in_binary_op{:?}\n", place);

                Some(place.clone())
            }
            Operand::Constant(_) => None, // 先忽略
        };

        let source2_place = match op2 {
            Operand::Copy(place) | Operand::Move(place) => {
                self.add_varnode(place.clone());
                                        println!("addvar_in_binary_op{:?}\n", place);

                Some(place.clone())
            }
            Operand::Constant(_) => None,
        };
        let BI: BasicInterval<T> = BasicInterval::new(Range::default(T::min_value()));
        let BOP = BinaryOp::new(BI, sink, inst, source1_place, source2_place, 0);
        let bop_index = self.oprs.len();

        self.oprs.push(BasicOpKind::Binary(BOP));
        // let bop_ref = unsafe { &*(self.oprs.last().unwrap() as *const BasicOp<'tcx, T>) };
        self.defmap.insert(sink, bop_index);
        if let Some(place) = source1_place {
            self.usemap
                .entry(place.clone())
                .or_default()
                .insert(bop_index);
        }

        if let Some(place) = source2_place {
            self.usemap
                .entry(place.clone())
                .or_default()
                .insert(bop_index);
        }

        // print!("varnodes{:?}\n", self.vars);
        // print!("defmap{:?}\n", self.defmap);
        // print!("usemap{:?}\n", self.usemap);
        // print!("{:?}add_binary_op{:?}\n", inst,sink);
        // ...
    }

    // pub fn find_intervals(&mut self) {
    //     // 构建符号交集映射
    //     self.build_symbolic_intersect_map();

    //     // 查找强连通分量（SCC）
    //     let scc_list = Nuutila::new(&self.vars, &self.usemap, &self.symbmap,false);
    //     self.num_sccs += scc_list.worklist.len();

    //     // 遍历每个 SCC
    //     for component in scc_list.components() {
    //         if component.len() == 1 {
    //             // 处理单节点的 SCC
    //             self.num_alone_sccs += 1;
    //             self.fix_intersects(&component);

    //             let var = component.iter().next().unwrap();
    //             if var.get_range().is_unknown() {
    //                 var.set_range(allue {
    //                     min: i32::MIN,
    //                     max: i32::MAX,
    //                 });
    //             }
    //         } else {
    //             // 更新最大 SCC 大小
    //             if component.len() > self.size_max_scc {
    //                 self.size_max_scc = component.len();
    //             }

    //             // 为该 SCC 构建使用映射
    //             let comp_use_map = self.build_use_map(&component);

    //             // 获取 SCC 的入口点
    //             let mut entry_points = HashSet::new();
    //             self.generate_entry_points(&component, &mut entry_points);

    //             // 固定点迭代，更新范围
    //             self.pre_update(&comp_use_map, &entry_points);
    //             self.fix_intersects(&component);

    //             // 为未知范围的变量设置默认范围
    //             for var in &component {
    //                 if var.get_range().is_unknown() {
    //                     var.set_range(Range {
    //                         min: i32::MIN,
    //                         max: i32::MAX,
    //                     });
    //                 }
    //             }

    //             // 二次迭代，更新活动变量
    //             let mut active_vars = HashSet::new();
    //             self.generate_active_vars(&component, &mut active_vars);
    //             self.pos_update(&comp_use_map, &active_vars, &component);
    //         }

    //         // 将结果传播到下一个 SCC
    //         self.propagate_to_next_scc(&component);
    //     }
    // }

    // 假设的辅助方法定义
    fn build_symbolic_intersect_map(&self) {
        // 构建符号交集映射
    }
    // pub fn find_intervals(&mut self) {
    //     // 构建符号交集映射（类似 buildSymbolicIntersectMap）
    //     self.build_symbolic_intersect_map();
    //     let num_sccs = 0;
    //     let num_alone_sccs = 0;
    //     let size_max_scc =1023;
    //     // 构造 SCC 列表
    //     let mut scc_list = Nuutila::new(&self.vars, &self.usemap, &self.symbmap,true);

    //     // 统计 SCC 数量
    //     num_sccs += scc_list.worklist.len();

    //     // 遍历每个 SCC
    //     for scc_id in scc_list.iter() {
    //         let component = scc_list.components[scc_id].clone(); // SmallPtrSet<VarNode*, 32>

    //         if component.len() == 1 {
    //             num_alone_sccs += 1;
    //             self.fix_intersects(&component);

    //             // let var = component.iter().next().unwrap();
    //             // if var.get_range().is_unknown() {
    //             //     var.set_range(Range::new(MIN, MAX));
    //             // }
    //         } else {
    //             // 记录最大 SCC 尺寸
    //             if component.len() > size_max_scc {
    //                 size_max_scc = component.len();
    //             }

    //             // 构建 UseMap
    //             let comp_use_map = Self::build_usemap(&component);

    //             // 找出入口点
    //             let mut entry_points = SmallPtrSet::<&Value, 6>::default();

    //             // 第一次固定点迭代前处理
    //             self.generate_entry_points(&component, &mut entry_points);
    //             self.pre_update(&comp_use_map, &entry_points);
    //             self.fix_intersects(&component);

    //             // 修正尚未设定范围的 VarNode
    //             for var in &component {
    //                 if var.get_range().is_unknown() {
    //                     var.set_range(Range::new(MIN, MAX));
    //                 }
    //             }

    //             // 第二次固定点迭代
    //             let mut active_vars = SmallPtrSet::<&Value, 6>::default();
    //             self.generate_active_vars(&component, &mut active_vars);
    //             self.pos_update(&comp_use_map, &active_vars, &component);
    //         }

    //         // 将信息传播到下一组 SCC
    //         self.propagate_to_next_scc(&component);
    //     }
    // }
}

pub struct Nuutila<'tcx, T: PartialOrd + Clone + Bounded + Debug> {
    pub variables: &'tcx VarNodes<'tcx, T>,
    pub index: i32,
    pub dfs: HashMap<Place<'tcx>, i32>,
    pub root: HashMap<Place<'tcx>, Place<'tcx>>,
    pub in_component: HashSet<Place<'tcx>>,
    pub components: HashMap<Place<'tcx>, HashSet<&'tcx VarNode<'tcx, T>>>,
    pub worklist: VecDeque<Place<'tcx>>,
}

impl<'tcx, T> Nuutila<'tcx, T>
where
    T: PartialOrd + Clone + Bounded + From<ScalarInt> + Debug,
{

    pub fn new(
        varNodes: &'tcx VarNodes<'tcx, T>,
        use_map: &mut UseMap<'tcx>,
        symb_map: &SymbMap<'tcx>,
        single: bool,
    ) -> Self {
        let mut n = Nuutila {
            variables:varNodes,
            index: 0,
            dfs: HashMap::new(),
            root: HashMap::new(),
            in_component: HashSet::new(),
            components: HashMap::new(),
            worklist: std::collections::VecDeque::new(),
            // 如果你用到了 control_deps 字段
            // control_deps: HashSet::new(),
        };

        if single {
            // 所有变量归入一个 SCC
            // let mut scc = HashSet::new();
            // for var_node in variables.values() {
            //     scc.insert(var_node.clone());
            // }

            // for (place, _) in variables.iter() {
            //     n.components.insert(place.clone(), scc.clone());
            // }

            // // 任选一个 place 放入 worklist
            // if let Some((first_place, _)) = variables.iter().next() {
            //     n.worklist.push_back(first_place.clone());
            // }
        } else {
            // 初始化 dfs 值
            for place in n.variables.keys() {
                n.dfs.insert(place.clone(), -1);
            }

            n.add_control_dependence_edges(symb_map, use_map, varNodes);

            for place in n.variables.keys() {
                if n.dfs[place] < 0 {
                    let mut stack = Vec::new();
                    n.visit(place, &mut stack, use_map);
                }
            }

            n.del_control_dependence_edges(use_map);
        }

        n
    }




    pub fn visit(
        &mut self,
        place: &Place<'tcx>,
        stack: &mut Vec<Place<'tcx>>,
        use_map: &UseMap<'tcx>,
    ) {
        self.dfs.insert(place.clone(), self.index);
        self.index += 1;
        self.root.insert(place.clone(), place.clone());

        // 访问所有使用该变量的 BasicOp（或指令）
        // if let Some(uses) = use_map.get(place) {
        //     for op in uses {
        //         let name = op.sink().value();

        //         // 初始化 dfs 值（如果尚未访问）
        //         if self.dfs.get(name).copied().unwrap_or(-1) < 0 {
        //             self.visit(name, stack, use_map);
        //         }

        //         // 如果 name 不在任何组件中，并且 root 更新
        //         if !self.in_component.contains(name) {
        //             let root_place = self.root.get(place).unwrap().clone();
        //             let root_name = self.root.get(name).unwrap().clone();

        //             let dfs_root_place = self.dfs.get(&root_place).copied().unwrap();
        //             let dfs_root_name = self.dfs.get(&root_name).copied().unwrap();

        //             if dfs_root_place >= dfs_root_name {
        //                 self.root.insert(place.clone(), root_name);
        //             }
        //         }
        //     }
        // }

        // // 第二阶段：确定组件并构建 SCC
        // if self.root.get(place) == Some(place) {
        //     self.worklist.push_back(place.clone());

        //     let mut scc = HashSet::new();
        //     if let Some(var_node) = self.variables.get(place) {
        //         scc.insert(var_node.clone());
        //     }

        //     self.in_component.insert(place.clone());

        //     while let Some(top) = stack.last() {
        //         if self.dfs.get(top).copied().unwrap_or(-1) > self.dfs.get(place).copied().unwrap() {
        //             let node = stack.pop().unwrap();
        //             self.in_component.insert(node.clone());

        //             if let Some(var_node) = self.variables.get(&node) {
        //                 scc.insert(var_node.clone());
        //             }
        //         } else {
        //             break;
        //         }
        //     }

        //     self.components.insert(place.clone(), scc);
        // } else {
        //     stack.push(place.clone());
        // }
    }



    pub fn add_control_dependence_edges(
        &mut self,
        _symb_map: &SymbMap<'tcx>,
        _use_map: &UseMap<'tcx>,
        _vars: &'tcx VarNodes<'tcx, T>,
    ) {
        todo!()
    }

    pub fn del_control_dependence_edges(&mut self, _use_map: &mut UseMap<'tcx>) {
        todo!()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Place<'tcx>> {
        self.worklist.iter().rev()
    }
}
