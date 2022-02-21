#![allow(dead_code)]
mod parse;
use std::rc::Rc;
// use std::fmt;
use std::collections::HashMap;

// hardcode SyGuS spec
// --------------------------------------------

type ValueT = u64;
type IOMapT = HashMap<ValueT, ValueT>;     // assume one input one output

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
enum OP {
    _lit(ValueT),
    _var,  // positional arguments

    bvnot,
    smol,
    ehad,
    arba,
    shesh,
    bvand,
    bvor,
    bvxor,
    bvadd,
    im
}
static LITS: &'static [ValueT] = &[0, 1];
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
    pub fn semantics(&self, a: &Vec<ValueT>, x: ValueT) -> ValueT {
        match self {
            OP::_lit(v) => *v,     // dereference -> default is move semantics, 
                                   // but u64 has copy semantics.
            OP::_var => x,
            OP::bvnot => !a[0],
            OP::smol => a[0] << 1,
            OP::ehad => a[0] >> 1,
            OP::arba => a[0] >> 4,
            OP::shesh => a[0] >> 16,
            OP::bvand => a[0] & a[1],
            OP::bvor => a[0] | a[1],
            OP::bvxor => a[0] ^ a[1],
            OP::bvadd => a[0] & a[1],
            OP::im => if a[0] == 1 as ValueT {a[1]} else {a[2]}
        }
    }
}

// --------------------------------------------


#[derive(Debug)]
struct GNode {
    operator: OP,
    children: Vec<Rc<GNode>>,
    io: IOMapT,
    size: i32
}

// impl GNode {
//     pub fn new(operator: OP, actuals: Vec<Rc<GNode>>) -> Rc<Self> {
//         let v = semantics(&operator, &actuals);
//         let s = actuals.iter().map(|u| u.size).sum::<i32>() + 1;
//         Rc::new(Self {
//             operator,
//             actuals,
//             io: v,
//             size: s
//         })
//     }
//     pub fn new_lit(lit: ValueT) -> Rc<Self> {
//         Rc::new(Self {
//             operator: OP::_lit(lit),
//             actuals: vec![],
//             io: lit,
//             size: 1
//         })
//     }
// }

// impl fmt::Display for GNode {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self.operator {
//             OP::Lit => write!(f, "{}", self.value),
//             OP::Not => write!(f, "!({})", self.actuals[0]),
//             OP::And => write!(f, "({})&({})", self.actuals[0], self.actuals[1])
//         }
//     }
// }

struct BottomUpSynthesizer {
    bank: Vec<Vec<Rc<GNode>>>,
    io_spec: IOMapT,
    inputs: Vec<ValueT>,
}

impl BottomUpSynthesizer {

    pub fn new_node(&mut self, operator: OP, children: Vec<Rc<GNode>>) -> Rc<GNode> {
        let get_actuals = |x| children.iter().cloned().map(|a| a.io[x]).collect::<Vec<ValueT>>();
        let io = self.inputs.iter().cloned()
            .map(|x| (x,  operator.semantics(&get_actuals(&x),x)  )  )
            .collect::<IOMapT>();
        let size = children.iter().map(|u| u.size).sum::<i32>() + 1;
        Rc::new(GNode {
            operator,
            children,
            io,
            size
        })
    }

    pub fn new_lit(&mut self, lit: ValueT) -> Rc<GNode> {
        Rc::new(GNode {
            operator: OP::_lit(lit),
            children: vec![],
            io: self.inputs.iter().cloned().map(|v| (v, lit)).collect(),
            size: 1
        })
    }

    pub fn synthesize(&mut self) -> Rc<GNode> {
        for s in 0..3 {
            let mut sbank = Vec::new();
            if s == 0 {

            } else if s == 1 {
                for lit in LITS {
                    sbank.push(self.new_lit(*lit)) // copy semantics of u64
                }
                sbank.push(self.new_node(OP::_var, vec![]))
            } else {
                for op in self.ops.iter().clone() {
                    for args in self.gen_args(s-1, op.arity()) {
                        sbank.push(GNode::new(op, args))
                    }
                }

            }
            self.bank.push(sbank);
        }

        // println!("{:?}", &bank[2][1]);
        // dbg!(bank);
        todo!()
    }

    // TODO types
    // TODO optimization. memoization
    fn gen_args(&mut self, total: i32, arity: i32) -> Vec<Vec<Rc<GNode>>> {
        if total < arity {
            return vec![]
        }
        let mut ret = vec![];
        if arity == 1 {
            for u in &self.bank[total as usize] {
                ret.push(vec![u.clone()])
            }
        } else {
            let upper = total - arity + 1;
            for y in 1..upper+1 { 
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

fn main() {
    // let mut map = HashMap::new();
    // map.insert("a", 1);
    // map.insert("b", 2);
    // map.insert("c", 3);
    
    let inputs : Vec<ValueT> = vec![1,2,3];
    dbg!( inputs.into_iter().map(|v| (v, 0 as u64)).collect::<IOMapT>());
    // io: self.io_spec.keys().map(|_| lit).collect(),

    // let _ = synthesize();
}
