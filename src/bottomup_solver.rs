use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

// hardcode SyGuS spec
// --------------------------------------------

pub type ValueT = u64;
pub type IOMapT = HashMap<ValueT, ValueT>; // assume one input one output

#[derive(Debug, Clone)]
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
    io: IOMapT,
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
    io_spec: IOMapT,
    inputs: Vec<ValueT>,
}

impl BottomUpSynthesizer {
    pub fn new(io_spec: IOMapT) -> Self {
        let inputs = io_spec.keys().cloned().collect();
        Self {
            bank: vec![],
            io_spec,
            inputs,
        }
    }

    fn new_node(&mut self, operator: OP, children: Vec<Rc<GNode>>) -> Rc<GNode> {
        let get_actuals = |x| {
            children
                .iter()
                .cloned()
                .map(|a| a.io[x])
                .collect::<Vec<ValueT>>()
        };
        let io = self
            .inputs
            .iter()
            .map(|x| (*x, operator.semantics(&get_actuals(x), x)))
            .collect::<IOMapT>();
        let size = children.iter().map(|u| u.size).sum::<i32>() + 1;
        Rc::new(GNode {
            operator,
            children,
            io,
            size,
        })
    }

    fn new_lit(&mut self, lit: ValueT) -> Rc<GNode> {
        Rc::new(GNode {
            operator: OP::_lit(lit),
            children: vec![],
            io: self.inputs.iter().cloned().map(|v| (v, lit)).collect(),
            size: 1,
        })
    }

    fn is_goal(&self, u: &Rc<GNode>) -> bool {
        u.io == self.io_spec
    }

    pub fn synthesize(&mut self, maxs: usize) -> Option<Rc<GNode>> {
        for s in 0..maxs + 1 {
            let mut sbank = Vec::new();
            macro_rules! check_or_push {
                ($ue:expr) => {
                    let u = $ue;
                    if self.is_goal(&u) {
                        return Some(u);
                    }
                    sbank.push(u);
                };
            }
            if s == 0 {
            } else if s == 1 {
                for lit in LITS {
                    check_or_push!(self.new_lit(*lit));
                }
                check_or_push!(self.new_node(OP::_var, vec![]));
            } else {
                for op in OPS.iter() {
                    for args in self.gen_args((s - 1) as i32, op.arity()) {
                        check_or_push!(self.new_node(op.clone(), args));
                    }
                }
            }
            self.bank.push(sbank);
        }

        println!("not found within size {}", maxs);
        // for s in 1..maxs + 1 {
        //     println!("programs of size {}", s);
        //     for u in &self.bank[s] {
        //         println!("{}", u);
        //     }
        // }
        None
    }

    // TODO types
    // TODO optimization. memoization
    fn gen_args(&mut self, total: i32, arity: i32) -> Vec<Vec<Rc<GNode>>> {
        if total < arity {
            return vec![];
        }
        let mut ret = vec![];
        if arity == 1 {
            for u in &self.bank[total as usize] {
                ret.push(vec![u.clone()])
            }
        } else {
            let upper = total - arity + 1;
            for y in 1..upper + 1 {
                for u in self.bank[y as usize].clone() {
                    for mut xs in self.gen_args(total - y, arity - 1) {
                        xs.push(u.clone());
                        ret.push(xs);
                    }
                }
            }
        }
        ret
    }

    // fn gen_size(total: i32, arity: i32) -> Vec<Vec<i32>> {
    //     if total < arity {
    //         return vec![]
    //     }
    //     if total == arity {
    //         return vec![vec![1; arity as usize]]
    //     }
    //     if arity == 1 {
    //         return vec![vec![total]]
    //     }
    //     let mut ret = vec![];
    //     let upper = total - arity + 1;
    //     for y in 1..upper+1 {
    //         for mut xs in BottomUpSynthesizer::gen_size(total - y, arity - 1) {
    //             xs.push(y);
    //             ret.push(xs);
    //         }
    //     }
    //     ret
    // }
}