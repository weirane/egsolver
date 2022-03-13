use std::collections::HashMap;
use std::process::Output;
use std::rc::Rc;
use std::{fmt, option};

// hardcode SyGuS spec
// --------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
enum OP {
    _int(i32),
    _str(String),
    _var, // positional arguments, in PBE String condition, there is only one var with type string.

    //ntString op
    str_concat,
    str_replace,
    str_at,
    int_to_str,
    str_substr,

    //ntInt op
    plus,
    minus,
    str_len,
    str_to_int,
    str_indexof,

    //ntBoolean op
    str_prefixof,
    str_suffixof,
    str_contains,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ValueT {
    StringV(String),
    Int(i32),
    Bool(bool),
}
impl ValueT {
    pub fn getIntVal(&self) -> i32 {
        match self {
            ValueT::Int(v) => v.clone(),
            _ => panic!(),
        }
    }

    pub fn getStrVal(&self) -> String {
      //!(&self);
        match self {
            ValueT::StringV(v) => v.clone(),
            _ => panic!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ReturnT {
    String,
    Int,
    Bool,
}
pub type IOMapT = HashMap<ValueT, ValueT>; // assume one input one output
pub type VecT = Vec<ValueT>;
//pub const Str_const: &[ValueT] = &[ValueT::StringV(String::from(" "))];
//pub const Int_const: &[ValueT] = &[ValueT::Int(0),ValueT::Int(1),ValueT::Int(2),ValueT::Int(3),ValueT::Int(4), ValueT::Int(5)];
pub const Bool_const: &[bool] = &[true, false];

static OPS: &[OP] = &[
    // dunno how to loop enum

    //ntString op
    OP::str_concat,
    OP::str_replace,
    OP::str_at,
    OP::int_to_str,
    OP::str_substr,
    //ntInt op
    OP::plus,
    OP::minus,
    OP::str_len,
    OP::str_to_int,
    OP::str_indexof,
    //ntBoolean op
    // OP::str_prefixof,
    // OP::str_suffixof,
    // OP::str_contains,
];

// pub const NEG1: ValueT = u64::MAX;

impl OP {
    pub fn arity(&self) -> i32 {
        match self {
            OP::_int(v) => 0,
            OP::_str(v) => 0,
            OP::_var => 0, // positional arguments

            //ntString op
            OP::str_concat => 2,
            OP::str_replace => 3,
            OP::str_at => 2,
            OP::int_to_str => 1,
            OP::str_substr => 3,

            //ntInt op
            OP::plus => 2,
            OP::minus => 2,
            OP::str_len => 1,
            OP::str_to_int => 1,
            OP::str_indexof => 3,

            //ntBoolean op
            str_prefixof => 2,
            str_suffixof => 2,
            str_contains => 2,
        }
    }

    pub fn returnType(&self) -> ReturnT {
        match self {
            OP::_int(i32) => ReturnT::Int,
            OP::_str(str) => ReturnT::String,
            OP::_var =>  ReturnT::String, // positional arguments

            //ntString op
            OP::str_concat => ReturnT::String,
            OP::str_replace => ReturnT::String,
            OP::str_at => ReturnT::String,
            OP::int_to_str => ReturnT::String,
            OP::str_substr => ReturnT::String,

            //ntInt op
            OP::plus => ReturnT::Int,
            OP::minus => ReturnT::Int,
            OP::str_len => ReturnT::Int,
            OP::str_to_int => ReturnT::Int,
            OP::str_indexof => ReturnT::Int,

            //ntBoolean op
            str_prefixof => ReturnT::Bool,
            str_suffixof => ReturnT::Bool,
            str_contains => ReturnT::Bool,
        }
    }

    pub fn inputType(&self) -> &[ReturnT] {
        match self {
            // OP::_int(i32) => &[ReturnT::Int],
            // OP::_str(str) => &[ReturnT::String],
            // OP::_var => &[ReturnT::String], // positional arguments

            //ntString op
            OP::str_concat => &[ReturnT::String, ReturnT::String],
            OP::str_replace => &[ReturnT::String, ReturnT::String, ReturnT::String],
            OP::str_at => &[ReturnT::String, ReturnT::Int],
            OP::int_to_str => &[ReturnT::Int],
            OP::str_substr => &[ReturnT::String, ReturnT::Int, ReturnT::Int],

            //ntInt op
            OP::plus => &[ReturnT::Int, ReturnT::Int],
            OP::minus => &[ReturnT::Int, ReturnT::Int],
            OP::str_len => &[ReturnT::String],
            OP::str_to_int => &[ReturnT::String],
            OP::str_indexof => &[ReturnT::String, ReturnT::String, ReturnT::Int],

            //ntBoolean op
            OP::str_prefixof => &[ReturnT::String, ReturnT::String],
            OP::str_suffixof => &[ReturnT::String, ReturnT::String],
            OP::str_contains => &[ReturnT::String, ReturnT::String],
            _ => panic!(),
        }
    }

    // assume environment is just that one input
    pub fn semantics(&self, a: &[ValueT], x: &ValueT) -> Option<ValueT> {
        // dbg!(&self);
        // dbg!(&a);
        let ret = match self {
            
            OP::_int(v) => ValueT::Int(a[0].getIntVal()),
            OP::_str(v) => ValueT::StringV(a[0].getStrVal()),
            OP::_var => x.clone(), // positional arguments


            OP::str_concat => ValueT::StringV(a[0].getStrVal() + &a[1].getStrVal()),
            OP::str_replace => ValueT::StringV(
                a[0].getStrVal()
                    .replace(&a[1].getStrVal(), &a[2].getStrVal()),
            ),
            OP::str_at => ValueT::StringV(
                a[0].getStrVal()
                    .chars()
                    .nth(a[1].getIntVal() as usize)?
                    .to_string(),
            ),
            OP::int_to_str => ValueT::StringV(a[0].getIntVal().to_string()),
            OP::str_substr => {
                if a[0].getStrVal().len() as i32 >= a[2].getIntVal()
                    && a[1].getIntVal() <= a[2].getIntVal()
                    && a[1].getIntVal() >= 0
                {
                    ValueT::StringV(
                        a[0].getStrVal()[a[1].getIntVal() as usize..a[2].getIntVal() as usize]
                            .to_string()
                    )
                } else {
                    // dbg!("error");
                    // dbg!( a[0].getStrVal());
                    // dbg!(a[1].getIntVal());
                    // dbg!(a[2].getIntVal());
                    
                    return None;
                }
            }
            //ntInt op
            OP::plus => ValueT::Int(a[0].getIntVal() + a[1].getIntVal()),
            OP::minus => ValueT::Int(a[0].getIntVal() - a[1].getIntVal()),
            OP::str_len => ValueT::Int(a[0].getStrVal().len() as i32),
            OP::str_to_int => {
              let ret = a[0].getStrVal().parse::<i32>();
              match ret{
                Ok(ret) => ValueT::Int(ret),
                Err(e) => return None
              }
            },
            OP::str_indexof => {if a[0].getStrVal().len() as i32 > a[2].getIntVal() && a[2].getIntVal() >= 0  {ValueT::Int(
                a[0].getStrVal()[a[2].getIntVal() as usize..].find(&a[1].getStrVal())? as i32,
            )}
            else{
              // if a[1].getStrVal() == " "{
              //   dbg!("error");
              //   //dbg!( a[0].getStrVal());
              //   //dbg!(a[1].getStrVal());
              //   dbg!(a[2].getIntVal());
              // }
              
              return None;
            }
          },

            //ntBoolean op
            //TODO
            OP::str_prefixof => ValueT::Bool(true),
            OP::str_suffixof => ValueT::Bool(true),
            OP::str_contains => ValueT::Bool(true),
            //_ => panic!(),
        };
        Some(ret)
    }
}

#[derive(Debug)]
pub struct GNode {
    operator: OP,
    children: Vec<Rc<GNode>>,
    outvec: VecT,
    size: i32,
    outType: ReturnT,
}

impl fmt::Display for GNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match &self.operator{
        OP::_int(v) => write!(f, "({})", v),
        OP::_str(v) => write!(f, "(\"{}\")", v),
        OP::_var => write!(f, "({})", "name"), // positional arguments

        OP::str_concat => write!(f, "concat({} , {})", self.children[0], self.children[1]),
        OP::str_replace => write!(f, "str.replace({} , {}, {})", self.children[0], self.children[1],self.children[2]),
        OP::str_at => write!(f, "str.at({} , {})", self.children[0], self.children[1]),
        OP::int_to_str => write!(f, "str.to.int({})", self.children[0]), 
        OP::str_substr => write!(f, "str.substr({} , {}, {})", self.children[0], self.children[1], self.children[2]),

        //ntInt op
        OP::plus => write!(f, "{} + {}", self.children[0], self.children[1]),
        OP::minus => write!(f, "{} - {}", self.children[0], self.children[1]),
        OP::str_len => write!(f, "str.len({})", self.children[0]),
        OP::str_to_int => write!(f, "str.to.int({})", self.children[0]),
        OP::str_indexof => write!(f, "str.indexof({} , {}, {})", self.children[0], self.children[1], self.children[2]),

        OP::str_prefixof => write!(f, "str.prefixof( {} , {})", self.children[0], self.children[1]),
        OP::str_suffixof => write!(f, "str.suffixof( {} , {})", self.children[0], self.children[1]),
        OP::str_contains => write!(f, "str.contains( {} , {})", self.children[0], self.children[1]),
      }

    }
}

pub struct BottomUpSynthesizer {
    new_bank: HashMap<(ReturnT, i32), Vec<Rc<GNode>>>,
    //bank: Vec<Vec<Rc<GNode>>>,
    inputs: VecT,
    outputs: VecT,
    children_comb: HashMap<(String, i32), Vec<Vec<Rc<GNode>>>>,
    enable_oe: bool, // observational equivalence pruning
    enable_mc: bool, // memorize children-enumeration
    str_const: Vec<String>,
    int_const: Vec<i32>
}

impl BottomUpSynthesizer {
    pub fn new(io_spec: IOMapT, enable_oe: bool, enable_mc: bool, str_const: Vec<String>, int_const: Vec<i32>) -> Self {
        let (inputs, outputs): (Vec<_>, Vec<_>) = io_spec.into_iter().unzip();
        Self {
            new_bank: HashMap::new(),
            inputs,
            outputs,
            children_comb: HashMap::new(),
            enable_oe,
            enable_mc,
            str_const,
            int_const
        }
    }

    fn new_node(&mut self, operator: OP, children: Vec<Rc<GNode>>) -> Option<Rc<GNode>> {
      //dbg!(&operator);
      //dbg!(&children[0].operator);
        let get_actuals = |i| {
            children
                .iter()
                .cloned()
                .map(|a| (&a.outvec[i] as &ValueT).clone())
                .collect::<VecT>()
        };

        // if operator == OP::str_indexof{
        //   dbg!(&children[1]);
        // }
        let outvec = self
            .inputs
            .iter()
            .enumerate()
            .map(|(i , x)| { operator.semantics(&get_actuals(i), x)})
            .collect::<Option<VecT>>()?;
        // if operator == OP::str_substr{
        //   dbg!(&outvec);
        // }

        //concat("Dr." , substr (name, 0, idxof(name, " ", 0))), 
        let size = children.iter().map(|u| u.size).sum::<i32>() + 1;
        let outType = operator.returnType();
        Some(Rc::new(GNode {
            operator,
            children,
            outvec,
            size,
            outType,
        }))
    }

    // fn new_const_node(&mut self, val: ValueT) -> Rc<GNode> {
    //     match val {
    //         ValueT::String(str) => self.new_str(str),
    //         ValueT::Int(ival) => self.new_int(ival),
    //         ValueT::Bool(bv) => self.new_boolean(bv),
    //         _ => panic!(),
    //     }
    // }


    fn new_str(&mut self, str: ValueT) -> Rc<GNode> {
        Rc::new(GNode {
            operator: OP::_str(str.getStrVal()),
            children: vec![],
            outvec: self.inputs.iter().map(|_| str.clone()).collect(),
            size: 1,
            outType: ReturnT::String,
        })
    }

    fn new_int(&mut self, intV: ValueT) -> Rc<GNode> {
        Rc::new(GNode {
            operator: OP::_int(intV.getIntVal()),
            children: vec![],
            outvec: self.inputs.iter().map(|_| intV.clone()).collect(),
            size: 1,
            outType: ReturnT::Int,
        })
    }

    fn new_boolean(&mut self, bv: bool) -> Rc<GNode> {
        todo!()
    }

    fn is_goal(&self, u: &Rc<GNode>) -> bool {
        if u.outType == ReturnT::String && u.outvec == self.outputs {
            return true;
        }
        // dbg!(&u.outvec);
        // dbg!(&self.inputs);
      
        return false;
    }

    pub fn synthesize(&mut self, maxs: usize) -> Option<Rc<GNode>> {
        let mut classmap = HashMap::<VecT, Rc<GNode>>::new();
        // let  Str_const = &[ValueT::StringV(String::from(" "))];
        // let Int_const = &[ValueT::Int(0),ValueT::Int(1),ValueT::Int(2),ValueT::Int(3),ValueT::Int(4), ValueT::Int(5)];
        for s in 0..maxs + 1 {
            //dbg!(&self.new_bank);
            let mut sbank = Vec::new();
            let mut ibank = Vec::new();
            // check for goal / redundancy. if not, add to bank
            macro_rules! check_or_push {
                ($ue:expr) => {
                    let u = $ue;
                    if self.is_goal(&u) {
                        return Some(u);
                    }
                    // if !self.enable_oe || classmap.get(&u.outvec).is_none() {
                    //     if self.enable_oe {
                    //         classmap.insert(u.outvec.clone(), u.clone());
                    //     }
                    //     //sbank.push(u);
                    // }
                    if u.outType == ReturnT::String {
                        //println!("{}", "1");
                        if !self.enable_oe || classmap.get(&u.outvec).is_none() {
                          if self.enable_oe {
                              classmap.insert(u.outvec.clone(), u.clone());
                        }
                          sbank.push(u);
                      }
                        //sbank.push(u);
                    } else if u.outType == ReturnT::Int {
                      //println!("{}", "2");
                      if !self.enable_oe || classmap.get(&u.outvec).is_none() {
                        if self.enable_oe {
                            classmap.insert(u.outvec.clone(), u.clone());
                        }
                        ibank.push(u);
                    }
                        //ibank.push(u);
                    }
                    //println!("{}", "3");
                };
            }
            if s == 0 {
            } else if s == 1 {
                
                //dbg!();
                for sval in self.str_const.clone() {
                    check_or_push!(self.new_str(ValueT::StringV(sval)));
                }
                for ival in self.int_const.clone() {
                    check_or_push!(self.new_int(ValueT::Int(ival)));
                }
                let new_node = self.new_node(OP::_var, vec![])?;
                check_or_push!(new_node);
                
            } else {
                for op in OPS.iter() {
                    for args in
                        self.gen_args((s - 1) as i32, op.arity(), op.inputType(), op.returnType())
                    {
                      let new_node = self.new_node(op.clone(), args);
                      if let Some(new_node) = new_node{
                        check_or_push!(new_node);
                      }
                      else{
                        continue;
                      }
                      //check_or_push!(self.new_node(op.clone(), args));
                        
                    }
                }
            }
            self.new_bank.insert((ReturnT::Int, s as i32), ibank);
            self.new_bank.insert((ReturnT::String, s as i32), sbank);
            //dbg!(&self.new_bank);
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
    fn gen_args(
        &mut self,
        total: i32,
        arity: i32,
        in_type: &[ReturnT],
        out_type: ReturnT,
    ) -> Vec<Vec<Rc<GNode>>> {
        if total < arity {
            return vec![];
        }
        // if self.enable_mc {
        //     if let Some(v) = self.children_comb.get(&(total, arity)) {
        //         return v.clone();
        //     }
        // }

        let mut ret = vec![];
        if arity == 1 {
            let emptyVec = vec![];
            let banklist = self.new_bank.get(&(in_type[0].clone(), total)).unwrap_or(&emptyVec);

            for u in banklist {
                ret.push(vec![u.clone()])
            }
        } else {
            let upper = total - arity + 1;
            for y in 1..upper + 1 {
                let emptyVec = vec![];
                let banklist = self.new_bank.get(&(in_type[0].clone(), y)).unwrap_or(&emptyVec);
                for u in banklist.clone() {
                    for mut xs in self.gen_args(total - y, arity - 1, &in_type[1..], in_type[0].clone()) {
                        xs.insert(0,u.clone());
                        ret.push(xs);
                    }
                }
            }
        }
        // if self.enable_mc {
        //     self.children_comb.insert((total, arity), ret.clone());
        // }
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
