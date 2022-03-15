```python
def synthesize(maxsize: int) -> Optional[Eclass]:
    let classmap: Map<OutputVec, Eclass>   # classmap[v] = the e-class whose output vector = v
    let sizes: Map<int, Set<Eclass>>       # sizes[s] = set of eclasses whose minsize = s
    initialize egraph with programs whose size <= 1
    for size from 2 to maxsize:
        for prod in productions:
            # generate arguments given children arity and total size
            for args in gen_args(size - 1, prod.arity, sizes):
                new_eclass = egraph.add(prod(args))
                # check if the new eclass can be merged with existing ones
                outvec = egraph[new_eclass].outputvec
                if outvec in classmap:
                    egraph.union(classmap[outvec], new_eclass)
                else:
                    classmap[outvec] = new_eclass
                    sizes[size] = sizes[size] ∪ {new_eclass}
                if is_goal(new_eclass):    # compares the outputvec to the io specification
                    return egraph.find(new_eclass)
    print("not found within size {maxsize}")
```

```python
def delete_best(eclass: Eclass, removed: Set<Enode>) -> bool:
    if eclass.minsize <= 2:
        return False
    best_node = find_best_node(eclass)
    # try to delete stuff in children first
    for child in best_node.children():
        if delete_best(child, removed):
            return True
    # if not, delete enodes with the same op
    to_delete = [n for n in eclass.nodes if same_op(n, best_node)]
    # only when this op is not the only op in the eclass
    if to_delete != eclass.nodes:
        removed = removed ∪ to_delete
        return True
    else:
        return False
```
