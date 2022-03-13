use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

// hardcode SyGuS spec
// --------------------------------------------

pub type ValueT = u64;
pub type IOMapT = HashMap<ValueT, ValueT>; // assume one input one output
pub type VecT = Vec<ValueT>;

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
enum OP {
    _lit(ValueT),
    _var, // positional arguments

    bvnot,
    smol,
    ehad,
    arba,
    shesh,
    bvand,
    bvor,
    bvxor,
    bvadd,
    im,
}

static OPS: &[OP] = &[
    // dunno how to loop enum
    OP::bvnot,
    OP::smol,
    OP::ehad,
    OP::arba,
    OP::shesh,
    OP::bvand,
    OP::bvor,
    OP::bvxor,
    OP::bvadd,
    OP::im,
];

pub const NEG1: ValueT = u64::MAX;
pub const LITS: &[ValueT] = &[0, 1, NEG1];

impl OP {
    pub fn arity(&self) -> i32 {
        match self {
            OP::_lit(_) => 0,
            OP::_var => 0,
            OP::bvnot => 1,
            OP::smol => 1,
            OP::ehad => 1,
            OP::arba => 1,
            OP::shesh => 1,
            OP::bvand => 2,
            OP::bvor => 2,
            OP::bvxor => 2,
            OP::bvadd => 2,
            OP::im => 3,
        }
    }
    // assume environment is just that one input
    pub fn semantics(&self, a: &[ValueT], x: &ValueT) -> ValueT {
        match self {
            OP::_lit(v) => *v,
            OP::_var => *x,
            OP::bvnot => !a[0],
            OP::smol => a[0] << 1,
            OP::ehad => a[0] >> 1,
            OP::arba => a[0] >> 4,
            OP::shesh => a[0] >> 16,
            OP::bvand => a[0] & a[1],
            OP::bvor => a[0] | a[1],
            OP::bvxor => a[0] ^ a[1],
            OP::bvadd => a[0].wrapping_add(a[1]),
            OP::im => {
                if a[0] == 1 {
                    a[1]
                } else {
                    a[2]
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct GNode {
    operator: OP,
    children: Vec<Rc<GNode>>,
    outvec: VecT,
    size: i32,
}

impl fmt::Display for GNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.operator {
            OP::_lit(NEG1) => write!(f, "-1"),
            OP::_lit(v) => write!(f, "{}", v),
            OP::_var => write!(f, "x"),
            OP::bvnot => write!(f, "~({})", self.children[0]),
            OP::smol => write!(f, "({}) << 1", self.children[0]),
            OP::ehad => write!(f, "({}) >> 1", self.children[0]),
            OP::arba => write!(f, "({}) >> 4", self.children[0]),
            OP::shesh => write!(f, "({}) >> 16", self.children[0]),
            OP::bvand => write!(f, "({}) & ({})", self.children[0], self.children[1]),
            OP::bvor => write!(f, "({}) | ({})", self.children[0], self.children[1]),
            OP::bvxor => write!(f, "({}) ^ ({})", self.children[0], self.children[1]),
            OP::bvadd => write!(f, "({}) + ({})", self.children[0], self.children[1]),
            OP::im => write!(
                f,
                "if {} == 1 then {} else {}",
                self.children[0], self.children[1], self.children[2]
            ),
        }
    }
}

pub struct BottomUpSynthesizer {
    bank: Vec<Vec<Rc<GNode>>>,
    inputs: VecT,
    outputs: VecT,
    enable_oe: bool,     // observational equivalence pruning
    enable_ft: bool,
}

impl BottomUpSynthesizer {
    pub fn new(io_spec: IOMapT, enable_oe: bool, enable_ft: bool) -> Self {
        let (inputs, outputs): (Vec<_>, Vec<_>) = io_spec.into_iter().unzip();
        Self {
            bank: vec![],
            inputs,
            outputs,
            enable_oe,
            enable_ft,
        }
    }

    fn new_node(&mut self, operator: OP, children: Vec<Rc<GNode>>) -> Rc<GNode> {
        let get_actuals = |i| {
            children
                .iter()
                .cloned()
                .map(|a| a.outvec[i])
                .collect::<VecT>()
        };
        let outvec = self
            .inputs
            .iter().enumerate()
            .map(|(i,x)| operator.semantics(&get_actuals(i), x))
            .collect::<VecT>();
        let size = children.iter().map(|u| u.size).sum::<i32>() + 1;
        Rc::new(GNode {
            operator,
            children,
            outvec,
            size,
        })
    }

    fn new_lit(&mut self, lit: ValueT) -> Rc<GNode> {
        Rc::new(GNode {
            operator: OP::_lit(lit),
            children: vec![],
            outvec: self.inputs.iter().map(|_| lit).collect(),
            size: 1,
        })
    }

    fn is_goal(&self, u: &Rc<GNode>) -> bool {
        u.outvec == self.outputs
    }

    pub fn synthesize(&mut self, maxs: usize) -> Option<Rc<GNode>> {
        let mut classmap = HashMap::<VecT, Rc<GNode>>::new();

        for s in 0..maxs + 1 {
            let mut sbank = Vec::new();
            // check for goal / redundancy. if not, add to bank
            macro_rules! check_or_push {
                ($ue:expr) => {
                    let u = $ue;
                    if self.is_goal(&u) {
                        println!("found at size = {}", s);
                        return Some(u);
                    }
                    if !self.enable_oe || classmap.get(&u.outvec).is_none() {
                        if self.enable_oe {
                            classmap.insert(u.outvec.clone(), u.clone());
                        }
                        sbank.push(u);
                    }
                }
            }
            if s == 0 {
            } else if s == 1 {
                for lit in LITS {
                    check_or_push!(self.new_lit(*lit));
                }
                check_or_push!(self.new_node(OP::_var, vec![]));
            } else {
                for op in OPS.iter() {
                    for args in self.gen_args((s - 1) as i32, op.arity(), *op) {
                        check_or_push!(self.new_node(*op, args));
                    }
                }
            }
            self.bank.push(sbank);
        }

        println!("not found within size {}", maxs);
        None
    }

    /// generate a list of argument lists, 
    /// given total size, and the number of args
    ///       context operator (for filtering)
    fn gen_args(&mut self, total: i32, arity: i32, op: OP) -> Vec<Vec<Rc<GNode>>> {
        if total < arity {
            return vec![];
        }

        let mut ret = vec![];
        if arity == 1 {
            for u in &self.bank[total as usize] {
                if !self.enable_ft || !self.filter(op, u) {
                    ret.push(vec![u.clone()]);
                }
            }
        } else {
            let upper = total - arity + 1;
            for y in 1..upper + 1 {
                for u in self.bank[y as usize].clone() {
                    for mut xs in self.gen_args(total - y, arity - 1, op) {
                        xs.push(u.clone());
                        ret.push(xs);
                    }
                }
            }
        }
        ret
    }

    /// returns tree iff can be filtered out
    /// for example (+ 0) can be filtered out
    fn filter(&self, op: OP, first_arg: &Rc<GNode>) -> bool {
        use OP::*;
        match (op, first_arg.operator) {
            (smol, ehad) => true,
            (ehad, smol) => true,
            (bvnot, _lit(0)) => true,
            (bvnot, _lit(NEG1)) => true,
            (bvnot, bvnot) => true,
            (bvand, _lit(NEG1)) => true,
            (bvand, _lit(0)) => true,
            (bvor, _lit(NEG1)) => true,
            (bvor, _lit(0)) => true,
            (bvxor, _lit(NEG1)) => true,
            (bvxor, _lit(0)) => true,
            (bvadd, _lit(0)) => true,
            _ => false
        }
    }
}
