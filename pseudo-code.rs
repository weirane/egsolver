pub fn synthesize_1(maxs: usize) -> Option<Eclass> {
    let sizes: Vec<HashSet<Eclass>>               // sizes[s] = set of eclasses whose minsize = s
    initialize egraph with programs whose size <= 1
    for size in 2..=maxs {
        sizes.push(HashSet::new());
        for f in productions {
            for args in gen_args(size - 1, f.arity) {  // generate arguments given children arity and total size
                let new_enode = f(args);
                let new_eclass = egraph.add(new_enode);
                // check if the new eclass can be merged with existing ones
                let same_eclass = none;
                for eclass in egraph.all_eclasses {
                    if eclass.analysis == new_eclass.analysis {
                        same_eclass = eclass;
                    }
                }
                if same_eclass != none {
                    egraph.union(same_eclass, new_eclass);
                } else {
                    sizes[size].insert(new_eclass);
                }
                if is_goal(new_eclass) {      // compares the outputvec to the io specification
                    egraph.rebuild();
                    return Some(egraph.find(new_eclass));
                }
            }
        }
        egraph.rebuild();
    }
    println!("not found within size {}", maxs); 
}

pub fn synthesize_2(maxs: usize) -> Option<Eclass> {
    let classmap: HashMap<OutputVec, Eclass>      // given one outputvec, there should be only one (merged) eclass.
    let sizes: Vec<HashSet<Eclass>>               // sizes[s] = set of eclasses whose minsize = s
    initialize egraph with programs whose size <= 1
    for size in 2..=maxs {
        sizes.push(HashSet::new());
        for f in productions {
            for args in gen_args(size - 1, f.arity) {  // generate arguments given children arity and total size
                let new_enode = f(args);
                let new_eclass = egraph.add(new_enode);
                // check if the new eclass can be merged with existing ones
                if let Some(&same_eclass) = classmap.get(&egraph[new_eclass].analysis) {
                    egraph.union(same_eclass, new_eclass);
                } else {
                    classmap.insert(egraph[new_eclass].analysis, new_eclass);
                    sizes[size].insert(new_eclass);
                }
                if is_goal(new_eclass) {      // compares the outputvec to the io specification
                    egraph.rebuild();
                    return Some(egraph.find(new_eclass));
                }
            }
        }
        egraph.rebuild();
    }
    println!("not found within size {}", maxs); 
}

/// takes a generic cost function, a top-level e-class, and howmany programs to extract
pub fn print_equivalents<C: CF>(eclass: Eclass, howmany: i32)
{
    let mut removed = HashSet::<Program>::new();
    for i in 1..=howmany {
        let ext = Extractor::new(&self.egraph, C::new(&removed));
        let (cost, ast) = ext.find_best(eclass);
        if cost >= INF_COST {
            break;
        }
        print(ast);
        if delete_best(eclass, &mut removed, &ext) == false {
            exhausted = true;
            break;
        }
    }
}

/// mark a tombstone on the best program
fn delete_best<C: CF>(id: Eclass, removed: *mut HashSet<Program>, ext: &Extractor<...>) -> bool {
    if eclass.minsize <= 2 { return false; }
    let enode = ext.find_best_node(id);
    // try to delete stuff in children first
    for child in enode.children() {
        if self.delete_best(*child, removed, ext) {
            return true;
        }
    }
    // if not, delete enodes with the same op, when this op is not the only op in the eclass
    let equivalents = &self.egraph[id].nodes;
    let to_delete = equivalents
        .iter()
        .filter(|u| u.matches(&enode))
        .collect::<Vec<_>>();
    if equivalents.len() > to_delete.len() {
        for u in to_delete {
            removed.as_mut().unwrap().insert(u.clone());
        }
        true
    } else {
        false
    }
}

/// our cost function trait requires "tombstone" implemented using hashtable
pub trait CF {
    fn new(removed: *const HashSet<Program>) -> Self;
}

static INF_COST: usize = 100000000;

/// ast size (each production has size 1)
#[derive(Debug)]
pub struct MyAstSize {
    removed: *const HashSet<Program>,
}
impl CostFunction<Program> for MyAstSize {
    type Cost = usize;
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
impl CF for MyAstSize {
    fn new(removed: *const HashSet<Program>) -> Self {
        Self { removed }
    }
}

/// ast size (each production has some specified size)
#[derive(Debug, Clone)]
pub struct VariedWeight {
    removed: *const HashSet<Program>,
}
impl CostFunction<Program> for VariedWeight {
    type Cost = usize;
    fn cost<C>(&mut self, enode: &Program, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        if unsafe { self.removed.as_ref().unwrap().contains(enode) } {
            INF_COST
        } else {
            let opcost = match enode {
                Program::Bvand(_) | Program::Bvor(_) | Program::Bvxor(_) => 2,
                Program::Bvadd(_) => 10,
                Program::Im(_) => 100,
                _ => 1,
            };
            enode.fold(opcost, |acc, arg| acc + costs(arg))
        }
    }
}
impl CF for VariedWeight {
    fn new(removed: *const HashSet<Program>) -> Self {
        Self { removed }
    }
}
