use egg::*;

use crate::bottomup_solver::{IOMapT, ValueT, LITS};

pub type EGraph = egg::EGraph<Program, ObservEquiv>;

define_language! {
    pub enum Program {
        Lit(ValueT),
        Var(String),
        "~" = Bvnot(Id),
        "<<1" = Smol(Id),
        ">>1" = Ehad(Id),
        ">>4" = Arba(Id),
        ">>16" = Shesh(Id),
        "&" = Bvand([Id; 2]),
        "|" = Bvor([Id; 2]),
        "^" = Bvxor([Id; 2]),
        "+" = Bvadd([Id; 2]),
        "ite" = Im([Id; 3]),
    }
}

impl Program {
    pub fn semantics(&self) -> impl Fn(&[ValueT], &ValueT) -> ValueT + '_ {
        use Program::*;
        move |args, env| match self {
            Lit(v) => *v,
            Var(_) => *env,
            Bvnot(_) => !args[0],
            Smol(_) => args[0] << 1,
            Ehad(_) => args[0] >> 1,
            Arba(_) => args[0] >> 4,
            Shesh(_) => args[0] >> 16,
            Bvand(_) => args[0] & args[1],
            Bvor(_) => args[0] | args[1],
            Bvxor(_) => args[0] ^ args[1],
            Bvadd(_) => args[0].wrapping_add(args[1]),
            Im(_) => {
                if args[0] == 1 {
                    args[1]
                } else {
                    args[2]
                }
            }
        }
    }
}

#[derive(Default)]
pub struct ObservEquiv {
    inputs: Vec<ValueT>,
}

impl ObservEquiv {
    pub fn new(inputs: Vec<ValueT>) -> Self {
        Self { inputs }
    }
}

impl Analysis<Program> for ObservEquiv {
    type Data = Vec<ValueT>;

    fn make(egraph: &EGraph, enode: &Program) -> Self::Data {
        let inputs = &egraph.analysis.inputs;
        let x = |i: &Id| &egraph[*i].data;
        use Program::*;
        match enode {
            Bvnot(id) | Smol(id) | Ehad(id) | Arba(id) | Shesh(id) => x(id)
                .iter()
                .zip(inputs)
                .map(|(&arg, inp)| enode.semantics()(&[arg], inp))
                .collect(),
            Bvand(ids) | Bvor(ids) | Bvxor(ids) | Bvadd(ids) => x(&ids[0])
                .iter()
                .zip(x(&ids[1]).iter())
                .zip(inputs)
                .map(|((&arg1, &arg2), inp)| enode.semantics()(&[arg1, arg2], inp))
                .collect(),
            Im(ids) => x(&ids[0])
                .iter()
                .zip(x(&ids[1]).iter())
                .zip(x(&ids[2]).iter())
                .zip(inputs)
                .map(|(((&arg1, &arg2), &arg3), inp)| enode.semantics()(&[arg1, arg2, arg3], inp))
                .collect(),
            Lit(v) => inputs.iter().map(|_| *v).collect(),
            Var(_) => inputs.clone(),
        }
    }

    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> DidMerge {
        assert_eq!(a, &b);
        // TODO: this may be true
        DidMerge(false, false)
    }
}

pub struct EggSynthesizer {
    pub bank: EGraph,
    outputs: Vec<ValueT>,
}

impl EggSynthesizer {
    pub fn new(io_spec: IOMapT) -> Self {
        let (inputs, outputs): (Vec<_>, Vec<_>) = io_spec.into_iter().unzip();
        Self {
            bank: EGraph::new(ObservEquiv::new(inputs)),
            outputs,
        }
    }

    fn is_goal(&self, prog: Id) -> bool {
        self.outputs == self.bank[prog].data
    }

    pub fn synthesize(&mut self, maxs: usize) -> Option<Id> {
        for &lit in LITS.iter() {
            self.bank.add(Program::Lit(lit));
        }
        self.bank.add(Program::Var(String::from("x")));
        for _ in 0..maxs {
            let ids: Vec<_> = self.bank.classes().map(|c| c.id).collect();
            for &c in ids.iter() {
                // expand nodes with arity 1
                for f in [
                    Program::Bvnot,
                    Program::Smol,
                    Program::Ehad,
                    Program::Arba,
                    Program::Shesh,
                ] {
                    let newnode = f(c);
                    let nid = self.bank.add(newnode);
                    // TODO: use hash table Vec<ValueT> -> Id
                    for &c in ids.iter() {
                        if self.bank[c].data == self.bank[nid].data {
                            self.bank.union(c, nid);
                            self.bank.rebuild();
                            break;
                        }
                    }
                    if self.is_goal(nid) {
                        return Some(nid);
                    }
                }
                // expand nodes with arity 2
                for f in [
                    Program::Bvand,
                    Program::Bvor,
                    Program::Bvxor,
                    Program::Bvadd,
                ] {
                    for &c in ids.iter() {
                        for &d in ids.iter() {
                            let newnode = f([c, d]);
                            let nid = self.bank.add(newnode);
                            for &c in ids.iter() {
                                if self.bank[c].data == self.bank[nid].data {
                                    self.bank.union(c, nid);
                                    self.bank.rebuild();
                                    break;
                                }
                            }
                            if self.is_goal(nid) {
                                return Some(nid);
                            }
                        }
                    }
                }
                // expand nodes with arity 3
                for &c in ids.iter() {
                    for &d in ids.iter() {
                        for &e in ids.iter() {
                            let newnode = Program::Im([c, d, e]);
                            let nid = self.bank.add(newnode);
                            for &c in ids.iter() {
                                if self.bank[c].data == self.bank[nid].data {
                                    self.bank.union(c, nid);
                                    self.bank.rebuild();
                                    break;
                                }
                            }
                            if self.is_goal(nid) {
                                return Some(nid);
                            }
                        }
                    }
                }
            }
        }
        println!("not found within iteration {}", maxs);
        None
    }
}
