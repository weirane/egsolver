#![allow(dead_code)]
mod parse;
use std::rc::Rc;
use std::fmt;

#[derive(Debug, Copy, Clone)]
enum OP {
    And,
    Not,
    Lit,
}

impl OP {
    pub fn arity(self) -> i32 {
        match self {
            OP::And => 2,
            OP::Not => 1,
            OP::Lit => 0
        }
    }
}

#[derive(Debug)]
struct GNode {
    operator: OP,
    actuals: Vec<Rc<GNode>>,
    value: i32,
    size: i32
}

impl GNode {
    pub fn new(operator: OP, actuals: Vec<Rc<GNode>>) -> Rc<Self> {
        let v = semantics(&operator, &actuals);
        let s = actuals.iter().map(|u| u.size).sum::<i32>() + 1;
        Rc::new(Self {
            operator,
            actuals,
            value: v,
            size: s
        })
    }
    pub fn new_lit(lit: i32) -> Rc<Self> {
        Rc::new(Self {
            operator: OP::Lit,
            actuals: vec![],
            value: lit,
            size: 1
        })
    }
}

impl fmt::Display for GNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.operator {
            OP::Lit => write!(f, "{}", self.value),
            OP::Not => write!(f, "!({})", self.actuals[0]),
            OP::And => write!(f, "({})&({})", self.actuals[0], self.actuals[1])
        }
    }
}

fn semantics(operator: &OP, a: &Vec<Rc<GNode>>) -> i32 {
    match operator {
        OP::And => a[0].value & a[1].value,
        OP::Not => if a[0].value == 0 {1} else {0},
        OP::Lit => 0     // <-- shouldn't reach here
    }
}


struct BottomUpSynthesizer {
    bank: Vec<Vec<Rc<GNode>>>,
    ops: Vec<OP>, // allowed operations
}

impl BottomUpSynthesizer {

    pub fn new(ops: Vec<OP>) -> Self {
        Self {
            bank: vec![],
            ops
        }
    }

    pub fn synthesize(&mut self) -> Rc<GNode> {
        for s in 0..3 {
            let mut sbank = Vec::new();
            if s == 0 {

            } else if s == 1 {
                for lit in 0..2 {
                    sbank.push(GNode::new_lit(lit))
                }
            } else {
                for op in self.ops.clone() {
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
    // let _ = synthesize();
}
