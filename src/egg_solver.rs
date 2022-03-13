use std::collections::{HashMap, HashSet};

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
    // output and min size
    type Data = (Vec<ValueT>, usize);

    fn make(egraph: &EGraph, enode: &Program) -> Self::Data {
        let inputs = &egraph.analysis.inputs;
        let o = |i: &Id| &egraph[*i].data.0; // output
        let s = |i: &Id| &egraph[*i].data.1; // minsize
        use Program::*;
        match enode {
            Bvnot(id) | Smol(id) | Ehad(id) | Arba(id) | Shesh(id) => {
                let outputs = o(id)
                    .iter()
                    .zip(inputs)
                    .map(|(&arg, inp)| enode.semantics()(&[arg], inp))
                    .collect();
                (outputs, s(id) + 1)
            }
            Bvand(ids) | Bvor(ids) | Bvxor(ids) | Bvadd(ids) => {
                let outputs = o(&ids[0])
                    .iter()
                    .zip(o(&ids[1]).iter())
                    .zip(inputs)
                    .map(|((&arg1, &arg2), inp)| enode.semantics()(&[arg1, arg2], inp))
                    .collect();
                (outputs, s(&ids[0]) + s(&ids[1]) + 1)
            }
            Im(ids) => {
                let outputs = o(&ids[0])
                    .iter()
                    .zip(o(&ids[1]).iter())
                    .zip(o(&ids[2]).iter())
                    .zip(inputs)
                    .map(|(((&arg1, &arg2), &arg3), inp)| {
                        enode.semantics()(&[arg1, arg2, arg3], inp)
                    })
                    .collect();
                (outputs, s(&ids[0]) + s(&ids[1]) + s(&ids[2]) + 1)
            }
            Lit(v) => (inputs.iter().map(|_| *v).collect(), 1),
            Var(_) => (inputs.clone(), 1),
        }
    }

    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> DidMerge {
        assert_eq!(a.0, b.0);
        if a.1 > b.1 {
            a.1 = b.1;
            DidMerge(true, false)
        } else if a.1 == b.1 {
            DidMerge(false, false)
        } else {
            DidMerge(false, true)
        }
    }
}

pub struct EggSynthesizer {
    pub bank: EGraph,
    sizes: Vec<HashSet<Id>>, // sizes[s] = set of eclasses whose minsize = s
    outputs: Vec<ValueT>,
}

impl EggSynthesizer {
    pub fn new(io_spec: IOMapT) -> Self {
        let (inputs, outputs): (Vec<_>, Vec<_>) = io_spec.into_iter().unzip();
        Self {
            bank: EGraph::new(ObservEquiv::new(inputs)),
            sizes: Vec::new(),
            outputs,
        }
    }

    fn is_goal(&self, prog: Id) -> bool {
        self.outputs == self.bank[prog].data.0
    }

    pub fn synthesize(&mut self, maxs: usize) -> Option<Id> {
        use Program::*;
        // map from outputs to eclass id
        let mut classmap: HashMap<Vec<ValueT>, Id> = HashMap::new();
        macro_rules! process {
            ($size:expr, $prog:expr) => {
                let prog = $prog;
                let nid = self.bank.add(prog);
                if let Some(&cid) = classmap.get(&self.bank[nid].data.0) {
                    self.bank.union(cid, nid);
                    // println!("nidsize {} cidsize {}", self.bank[nid].data.1, self.bank[cid].data.1);
                    // nid.size must >= cid.size, no need to insert nid to sizes
                } else {
                    classmap.insert(self.bank[nid].data.0.clone(), nid);
                    self.sizes[$size].insert(nid);
                }
                if self.is_goal(nid) {
                    self.bank.rebuild();
                    return Some(self.bank.find(nid));
                }
            };
        }
        self.sizes.push(HashSet::new());
        // nodes with arity 0
        self.sizes.push(HashSet::new());
        for &lit in LITS.iter() {
            process!(1, Lit(lit));
        }
        process!(1, Var(String::from("x")));
        for size in 2..=maxs {
            self.sizes.push(HashSet::new());
            // expand nodes with arity 1
            for args in self.gen_args(size - 1, 1) {
                for f in [Bvnot, Smol, Ehad, Arba, Shesh] {
                    let newnode = f(args[0]);
                    process!(size, newnode);
                }
            }
            // expand nodes with arity 2
            for args in self.gen_args(size - 1, 2) {
                for f in [Bvand, Bvor, Bvxor, Bvadd] {
                    let newnode = f([args[0], args[1]]);
                    process!(size, newnode);
                }
            }
            // expand nodes with arity 3
            for args in self.gen_args(size - 1, 3) {
                let newnode = Im([args[0], args[1], args[2]]);
                process!(size, newnode);
            }
            self.bank.rebuild();
        }
        println!("not found within size {}", maxs);
        None
    }

    /// Generates args for `arity` number of children whose sizes sum up to `total`.
    fn gen_args(&self, total: usize, arity: usize) -> Vec<Vec<Id>> {
        if total < arity {
            return vec![];
        }
        if arity == 1 {
            return self.sizes[total].iter().map(|&id| vec![id]).collect();
        }
        let upper = total - arity + 1;
        let mut res = vec![];
        for y in 1..=upper {
            for &u in self.sizes[y].iter() {
                for mut v in self.gen_args(total - y, arity - 1) {
                    v.push(u);
                    res.push(v);
                }
            }
        }
        res
    }

    pub fn print_equivalents(&mut self, id: Id, howmany: i32) {
        let mut removed = HashSet::<Program>::new();
        let mut exhausted = false;
        for i in 1..=howmany {
            let ext = Extractor::new(&self.bank, MyAstSize { removed: &removed });
            let (cost, ast) = ext.find_best(id);
            if cost >= INF_COST {
                exhausted = true;
                break;
            }
            println!(
                "[#{} cost={}] {}",
                i,
                cost,
                ast.pretty(80).replace("18446744073709551615", "-1")
            );
            if i < howmany && self.delete_best(id, &mut removed, &ext) == false {
                exhausted = true;
                break;
            }
        }
        if exhausted {
            println!("cannot find another non-trivial variant.");
        }
    }

    /// mark a tombstone on the best program
    fn delete_best(
        &self,
        id: Id,
        removed: *mut HashSet<Program>,
        ext: &Extractor<MyAstSize, Program, ObservEquiv>,
    ) -> bool {
        let eclass_minsize = self.bank[id].data.1;
        if eclass_minsize <= 2 {
            return false;
        }
        let enode = ext.find_best_node(id);
        // try to delete stuff in children first
        for child in enode.children() {
            if self.delete_best(*child, removed, ext) {
                return true;
            }
        }
        // if not, delete this enode when it is not the only one in the eclass
        let equivalents = &self.bank[id].nodes;
        if equivalents.len() > 1 {
            unsafe {
                // println!("\t\nremoved {:?} from {:?}\n", enode, equivalents);
                removed.as_mut().unwrap().insert(enode.clone());
            }
            true
        } else {
            false
        }
    }
}

static INF_COST: usize = 100000000;
#[derive(Debug)]
pub struct MyAstSize {
    removed: *const HashSet<Program>,
}
// impl<L: Language> CostFunction<L> for MyAstSize {
impl CostFunction<Program> for MyAstSize {
    type Cost = usize;
    // fn cost<C>(&mut self, enode: &L, mut costs: C) -> Self::Cost
    fn cost<C>(&mut self, enode: &Program, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        if unsafe { self.removed.as_ref().unwrap().contains(enode) } {
            INF_COST
        } else {
            enode.fold(1, |sum, id| sum + costs(id))
        }
    }
}
