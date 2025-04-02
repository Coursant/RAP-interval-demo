use super::{domain::*, range::RangeType, range::*};

use num_traits::Bounded;
use rand::Rng;
use rustc_middle::{
    mir::*,
    ty::{self, Const, ScalarInt, TyCtxt},
};
use rustc_mir_transform::*;
use rustc_span::sym::var;

use std::collections::{HashMap, HashSet};
pub struct ConstraintGraph<'a, T: PartialOrd + Clone + Bounded> {
    // Protected fields
    pub vars: VarNodes<'a, T>, // The variables of the source program
    pub oprs: GenOprs<'a, T>,  // The operations of the source program

    // Private fields
    // func: Option<Function>,             // Save the last Function analyzed
    pub defmap: DefMap<'a, T>, // Map from variables to the operations that define them
    pub usemap: UseMap<'a, T>, // Map from variables to operations where variables are used
    pub symbmap: SymbMap<'a, T>, // Map from variables to operations where they appear as bounds
    pub values_branchmap: ValuesBranchMap<'a, T>, // Store intervals, basic blocks, and branches
    // values_switchmap: ValuesSwitchMap<'a, T>, // Store intervals for switch branches
    constant_vector: Vec<T>, // Vector for constants from an SCC
}

impl<'a, T> ConstraintGraph<'a, T>
where
    T: PartialOrd + Clone + Bounded + From<ScalarInt>,
{
    pub fn new() -> Self {
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
        }
    }
    //     fn create_random_place() -> Place<'a> {
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
    pub fn create_random_place() -> Place<'a> {
        // 随机生成一个新的 Local 值
        let mut rng = rand::rng();
        let random_local = Local::from_usize(rng.random_range(10000..1000000)); // 假设 Local 的范围是 0 到 99

        // 创建一个新的 Place 值，使用随机生成的 Local 和空的投影列表
        Place {
            local: random_local,
            projection: ty::List::empty(),
        }
    }
    pub fn add_varnode(&mut self, v: &'a Place<'a>) -> &VarNode<'a, T> {
        // 如果变量已存在，则直接返回

        // 插入新的 VarNode
        let node = VarNode::new(v);
        // 先进行不可变借用，确保没有冲突
        let node_ref = self.vars.entry(v).or_insert(node);

        // 确保 usemap 也更新
        self.usemap.entry(v).or_insert(HashSet::new());

        node_ref
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
    pub fn build_graph(&mut self, body: &'a Body<'a>) {
        self.build_value_maps(body);
        for block in body.basic_blocks.indices() {
            let block_data = &body[block];
            // Traverse statements
            for statement in block_data.statements.iter() {
                // self.build_operations(statement);
            }
        }
    }

    pub fn build_value_maps(&mut self, body: &'a Body<'a>) {
        for bb in body.basic_blocks.indices() {
            let block_data = &body[bb];
            if let Some(terminator) = &block_data.terminator {
                match &terminator.kind {
                    TerminatorKind::SwitchInt { discr, targets } => {
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
    }

    pub fn build_value_branch_map(
        &mut self,
        body: &Body<'a>,
        discr: &'a Operand<'a>,
        targets: &'a SwitchTargets,
        block: &'a BasicBlockData<'a>,
    ) {
        // let place1: &Place<'a>;
        // 确保分支条件是二元比较
        if let Operand::Copy(place) | Operand::Move(place) = discr {
            if let Some((op1, op2, cmp_op)) = self.extract_condition(place, block) {
                let const_op1 = op1.constant();
                let const_op2 = op2.constant();

                match (const_op1, const_op2) {
                    (Some(c1), Some(c2)) => {}
                    (Some(c), None) | (None, Some(c)) => {
                        let const_in_left: bool;
                        let variable: &Place<'a>;
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
                        self.add_varnode(variable);
                        print!("{:?}\n", variable);
                        let scalar = c.const_.try_to_scalar_int().unwrap();
                        let scalar_value: T = scalar.into();

                        let const_range = Range::new(
                            scalar_value.clone(),
                            scalar_value.clone(),
                            RangeType::Regular,
                        );

                        let true_range = self.apply_comparison(
                            scalar_value.clone(),
                            cmp_op,
                            true,
                            const_in_left,
                        );
                        let false_range = self.apply_comparison(
                            scalar_value.clone(),
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
                        self.values_branchmap.insert(place, vbm);
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
                        self.add_varnode(p1);
                        self.add_varnode(p2);
                        let STOp1 = IntervalType::Symb(SymbInterval::new(CR.clone(), &p2, true));
                        let SFOp1 = IntervalType::Symb(SymbInterval::new(CR.clone(), &p2, false));
                        let STOp2 = IntervalType::Symb(SymbInterval::new(CR.clone(), &p1, true));
                        let SFOp2 = IntervalType::Symb(SymbInterval::new(CR.clone(), &p1, false));
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
        place: &'a Place<'a>,
        block: &'a BasicBlockData<'a>,
    ) -> Option<(&'a Operand<'a>, &'a Operand<'a>, BinOp)> {
        for stmt in &block.statements {
            if let StatementKind::Assign(box (lhs, Rvalue::BinaryOp(bin_op, box (op1, op2)))) =
                &stmt.kind
            {
                if lhs == place {
                    print!("!!!!!!");
                    print!("{:?}\n", bin_op);
                    print!("{:?}\n", op1);
                    print!("{:?}\n", op2);
                    return Some((op1, op2, *bin_op));
                }
            }
        }
        None
    }
    // pub fn calculate_ranges(
    //     &self,
    //     op1: &Operand<'a>,
    //     op2: &Operand<'a>,
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
    pub fn build_operations(&mut self, inst: &'a Statement<'a>) {
        // Handle binary instructions
        if let StatementKind::Assign(box (place, rvalue)) = &inst.kind {
            match rvalue {
                Rvalue::BinaryOp(op, box (op1, op2)) => {
                    let p1 = match op1 {
                        Operand::Copy(p) | Operand::Move(p) => p,
                        _ => panic!("Expected a place"),
                    };

                    self.add_varnode(p1);
                    let p2 = match op2 {
                        Operand::Copy(p) | Operand::Move(p) => p,
                        _ => panic!("Expected a place"),
                    };
                    self.add_varnode(p1);
                    self.add_varnode(p2);
                    // self.add_varnode(place);
                    self.add_binary_op(inst);
                }
                Rvalue::UnaryOp(op, op1) => {
                    let p = match op1 {
                        Operand::Copy(p) | Operand::Move(p) => p,
                        _ => panic!("Expected a place"),
                    };

                    self.add_varnode(p);
                    // self.add_varnode(place);
                    self.add_unary_op(inst);
                }
                _ => {}
            }
        }
    }
    fn add_unary_op(&mut self, stmt: &'a Statement<'a>) {
        let rand_place: Place<'a> = Self::create_random_place();
        let stmt_varnode = self.add_varnode(Box::leak(Box::new(rand_place)));
    }

    fn add_binary_op(&mut self, inst: &'a Statement<'a>) {
        // Implementation for adding binary operation
        // ...
    }

    // fn add_phi_op(&mut self, phi: &'a PHINode<'a>) {
    //     // Implementation for adding phi operation
    //     // ...
    // }

    // fn add_sigma_op(&mut self, phi: &'a PHINode<'a>) {
    //     // Implementation for adding sigma operation
    //     // ...
    // }
    // pub fn find_intervals(&mut self) {
    //     // 构建符号交集映射
    //     self.build_symbolic_intersect_map();

    //     // 查找强连通分量（SCC）
    //     let scc_list = Nuutila::new(&self.vars, &self.usemap, &self.symbmap);
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
}

// pub struct Nuutila<'a> {
//     worklist: Vec<&'a Rc<VarNode>>,
//     components: Vec<HashSet<Rc<VarNode>>>,
// }

// impl<'a> Nuutila<'a> {
//     fn new(
//         vars: &'a VarNodes,
//         use_map: &'a HashMap<String, Vec<Rc<VarNode>>>,
//         symb_map: &'a HashMap<String, Vec<Rc<VarNode>>>,
//     ) -> Self {
//         Nuutila {
//             worklist: Vec::new(),
//             components: Vec::new(),
//         }
//     }

//     fn components(&self) -> &Vec<HashSet<Rc<VarNode>>> {
//         &self.components
//     }
// }
