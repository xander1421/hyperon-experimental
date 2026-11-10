use hyperon_atom::*;
use hyperon_space::*;
use crate::metta::*;
use crate::metta::text::Tokenizer;
use crate::metta::types::{AtomType, get_atom_types, get_meta_type};
use hyperon_atom::matcher::atoms_are_equivalent;
use super::{grounded_op, regex};
use hyperon_atom::gnd::number::*;

use std::convert::TryInto;

#[derive(Clone, Debug)]
pub struct UniqueAtomOp {}

grounded_op!(UniqueAtomOp, "unique-atom");

impl Grounded for UniqueAtomOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_EXPRESSION, ATOM_TYPE_ATOM])
    }

    fn as_execute(&self) -> Option<&dyn CustomExecute> {
        Some(self)
    }
}

impl CustomExecute for UniqueAtomOp {    
    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {    
        let arg_error = || ExecError::from("unique expects single expression atom as an argument");    
        let expr = TryInto::<&ExpressionAtom>::try_into(args.get(0).ok_or_else(arg_error)?)?;    
    
        let mut atoms: Vec<Atom> = expr.children().into();    
        let mut seen: Vec<Atom> = Vec::new();    
        atoms.retain(|x| {    
            let not_contained = !seen.iter().any(|seen_atom| atoms_are_equivalent(seen_atom, x));    
            if not_contained { seen.push(x.clone()) };    
            not_contained    
        });    
        Ok(vec![Atom::expr(atoms)])    
    }    
}


#[derive(Clone, Debug)]
pub struct UnionAtomOp {}

grounded_op!(UnionAtomOp, "union-atom");

impl Grounded for UnionAtomOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_EXPRESSION, ATOM_TYPE_EXPRESSION, ATOM_TYPE_ATOM])
    }

    fn as_execute(&self) -> Option<&dyn CustomExecute> {
        Some(self)
    }
}

impl CustomExecute for UnionAtomOp {
    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("union expects and executable LHS and RHS atom");
        let mut lhs: Vec<Atom> = TryInto::<&ExpressionAtom>::try_into(args.get(0).ok_or_else(arg_error)?)?.children().into();
        let rhs: Vec<Atom> = TryInto::<&ExpressionAtom>::try_into(args.get(1).ok_or_else(arg_error)?)?.children().into();

        lhs.extend(rhs);

        Ok(vec![Atom::expr(lhs)])
    }
}

#[derive(Clone, Debug)]
pub struct IntersectionAtomOp {}

grounded_op!(IntersectionAtomOp, "intersection-atom");

impl Grounded for IntersectionAtomOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_EXPRESSION, ATOM_TYPE_EXPRESSION, ATOM_TYPE_ATOM])
    }

    fn as_execute(&self) -> Option<&dyn CustomExecute> {
        Some(self)
    }
}

impl CustomExecute for IntersectionAtomOp {
    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("intersection expects and executable LHS and RHS atom");
        let mut lhs: Vec<Atom> = TryInto::<&ExpressionAtom>::try_into(args.get(0).ok_or_else(arg_error)?)?.children().into();
        let rhs = TryInto::<&ExpressionAtom>::try_into(args.get(1).ok_or_else(arg_error)?)?.children();

        // Consume each equal RHS occurrence once, retaining LHS order and values.
        // Variables are compared by identity, not matched as trie wildcards.
        let mut used = vec![false; rhs.len()];
        lhs.retain(|candidate| {
            match rhs.iter().enumerate().find(|(i, r)| !used[*i] && *r == candidate) {
                Some((i, _)) => { used[i] = true; true }
                None => false,
            }
        });

        Ok(vec![Atom::expr(lhs)])
    }
}

#[derive(Clone, Debug)]
pub struct MaxAtomOp {}

grounded_op!(MaxAtomOp, "max-atom");

impl Grounded for MaxAtomOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_UNDEFINED, ATOM_TYPE_NUMBER])
    }

    fn as_execute(&self) -> Option<&dyn CustomExecute> {
        Some(self)
    }
}

impl CustomExecute for MaxAtomOp {
    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("max-atom expects one argument: expression");
        let expr = TryInto::<&ExpressionAtom>::try_into(args.get(0).ok_or_else(arg_error)?)?;
        let children = expr.children();
        if children.is_empty() {
            Err(ExecError::from("Empty expression"))
        } else {
            children.into_iter().fold(Ok(f64::NEG_INFINITY), |res, x| {
                match (res, Number::from_atom(x)) {
                    (res @ Err(_), _) => res,
                    (_, None) => Err(ExecError::from(format!("Only numbers are allowed in expression: {}", expr))),
                    (Ok(max), Some(x)) => Ok(f64::max(max, x.into())),
                }
            })
        }.map(|max| vec![Atom::gnd(Number::Float(max))])
    }
}

#[derive(Clone, Debug)]
pub struct MinAtomOp {}

grounded_op!(MinAtomOp, "min-atom");

impl Grounded for MinAtomOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_UNDEFINED, ATOM_TYPE_NUMBER])
    }

    fn as_execute(&self) -> Option<&dyn CustomExecute> {
        Some(self)
    }
}

impl CustomExecute for MinAtomOp {
    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("min-atom expects one argument: expression");
        let expr = TryInto::<&ExpressionAtom>::try_into(args.get(0).ok_or_else(arg_error)?)?;
        let children = expr.children();
        if children.is_empty() {
            Err(ExecError::from("Empty expression"))
        } else {
            children.into_iter().fold(Ok(f64::INFINITY), |res, x| {
                match (res, Number::from_atom(x)) {
                    (res @ Err(_), _) => res,
                    (_, None) => Err(ExecError::from(format!("Only numbers are allowed in expression: {}", expr))),
                    (Ok(min), Some(x)) => Ok(f64::min(min, x.into())),
                }
            })
        }.map(|min| vec![Atom::gnd(Number::Float(min))])
    }
}

#[derive(Clone, Debug)]
pub struct SizeAtomOp {}

grounded_op!(SizeAtomOp, "size-atom");

impl Grounded for SizeAtomOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_EXPRESSION, ATOM_TYPE_NUMBER])
    }

    fn as_execute(&self) -> Option<&dyn CustomExecute> {
        Some(self)
    }
}

impl CustomExecute for SizeAtomOp {
    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("size-atom expects one argument: expression");
        let children = TryInto::<&ExpressionAtom>::try_into(args.get(0).ok_or_else(arg_error)?)?.children();
        let size = children.len();
        Ok(vec![Atom::gnd(Number::Integer(size as i64))])
    }
}

#[derive(Clone, Debug)]
pub struct IndexAtomOp {}

grounded_op!(IndexAtomOp, "index-atom");

impl Grounded for IndexAtomOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_EXPRESSION, ATOM_TYPE_NUMBER, ATOM_TYPE_ATOM])
    }

    fn as_execute(&self) -> Option<&dyn CustomExecute> {
        Some(self)
    }
}

impl CustomExecute for IndexAtomOp {
    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("index-atom expects two arguments: expression and atom");
        let children = TryInto::<&ExpressionAtom>::try_into(args.get(0).ok_or_else(arg_error)?)?.children();
        let index = args.get(1).and_then(Number::from_atom).ok_or_else(arg_error)?;
        match children.get(Into::<i64>::into(index) as usize) {
            Some(atom) => Ok(vec![atom.clone()]),
            None => Err(ExecError::from("Index is out of bounds")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubtractionAtomOp {}

grounded_op!(SubtractionAtomOp, "subtraction-atom");

impl Grounded for SubtractionAtomOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_EXPRESSION, ATOM_TYPE_EXPRESSION, ATOM_TYPE_ATOM])
    }

    fn as_execute(&self) -> Option<&dyn CustomExecute> {
        Some(self)
    }
}

impl CustomExecute for SubtractionAtomOp {
    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("subtraction expects and executable LHS and RHS atom");
        let mut lhs: Vec<Atom> = TryInto::<&ExpressionAtom>::try_into(args.get(0).ok_or_else(arg_error)?)?.children().into();
        let rhs = TryInto::<&ExpressionAtom>::try_into(args.get(1).ok_or_else(arg_error)?)?.children();

        // Consume each equal RHS occurrence once; keep unmatched LHS values.
        let mut used = vec![false; rhs.len()];
        lhs.retain(|candidate| {
            match rhs.iter().enumerate().find(|(i, r)| !used[*i] && *r == candidate) {
                Some((i, _)) => { used[i] = true; false }
                None => true,
            }
        });

        Ok(vec![Atom::expr(lhs)])
    }
}

#[derive(Clone, Debug)]
pub struct GetTypeOp {
    space: DynSpace,
}

grounded_op!(GetTypeOp, "get-type");

impl GetTypeOp {
    pub fn new(space: DynSpace) -> Self {
        Self{ space }
    }
}

impl Grounded for GetTypeOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_ATOM, ATOM_TYPE_UNDEFINED])
    }

    fn as_execute(&self) -> Option<&dyn CustomExecute> {
        Some(self)
    }
}

impl CustomExecute for GetTypeOp {
    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("get-type expects single atom as an argument");
        let atom = args.get(0).ok_or_else(arg_error)?;
        let space = match args.get(1) {
            Some(space) => Atom::as_gnd::<DynSpace>(space)
                .ok_or("match expects a space as the first argument"),
            None => Ok(&self.space),
        }?;
        let types = get_atom_types(space, atom);
        if types.iter().all(AtomType::is_error) {
            Ok(vec![EMPTY_SYMBOL])
        } else {
            Ok(types.into_iter().filter(AtomType::is_valid).map(AtomType::into_atom).collect())
        }
    }
}

#[derive(Clone, Debug)]
pub struct GetMetaTypeOp { }

grounded_op!(GetMetaTypeOp, "get-metatype");

impl Grounded for GetMetaTypeOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_ATOM, ATOM_TYPE_ATOM])
    }

    fn as_execute(&self) -> Option<&dyn CustomExecute> {
        Some(self)
    }
}

impl CustomExecute for GetMetaTypeOp {
    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("get-metatype expects single atom as an argument");
        let atom = args.get(0).ok_or_else(arg_error)?;

        Ok(vec![get_meta_type(&atom)])
    }
}

#[derive(Clone, Debug)]
pub struct GetTypeSpaceOp {}

grounded_op!(GetTypeSpaceOp, "get-type-space");

impl Grounded for GetTypeSpaceOp {
    fn type_(&self) -> Atom {
        Atom::expr([ARROW_SYMBOL, ATOM_TYPE_SPACE, ATOM_TYPE_ATOM, ATOM_TYPE_ATOM])
    }

    fn as_execute(&self) -> Option<&dyn CustomExecute> {
        Some(self)
    }
}

impl CustomExecute for GetTypeSpaceOp {
    fn execute(&self, args: &[Atom]) -> Result<Vec<Atom>, ExecError> {
        let arg_error = || ExecError::from("get-type-space expects two arguments: space and atom");
        let space = args.get(0).ok_or_else(arg_error)?;
        let space = Atom::as_gnd::<DynSpace>(space).ok_or("get-type-space expects a space as the first argument")?;
        let atom = args.get(1).ok_or_else(arg_error)?;
        log::debug!("GetTypeSpaceOp::execute: space: {}, atom: {}", space, atom);
        let types = get_atom_types(space, atom);
        if types.iter().all(AtomType::is_error) {
            Ok(vec![EMPTY_SYMBOL])
        } else {
            Ok(types.into_iter().filter(AtomType::is_valid).map(AtomType::into_atom).collect())
        }
    }
}

pub(super) fn register_context_dependent_tokens(tref: &mut Tokenizer, space: &DynSpace) {
    let get_type_op = Atom::gnd(GetTypeOp::new(space.clone()));
    tref.register_token(regex(r"get-type"), move |_| { get_type_op.clone() });
}

pub(super) fn register_context_independent_tokens(tref: &mut Tokenizer) {
    let get_type_space_op = Atom::gnd(GetTypeSpaceOp{});
    tref.register_token(regex(r"get-type-space"), move |_| { get_type_space_op.clone() });
    let get_meta_type_op = Atom::gnd(GetMetaTypeOp{});
    tref.register_token(regex(r"get-metatype"), move |_| { get_meta_type_op.clone() });
    let min_atom_op = Atom::gnd(MinAtomOp{});
    tref.register_token(regex(r"min-atom"), move |_| { min_atom_op.clone() });
    let max_atom_op = Atom::gnd(MaxAtomOp{});
    tref.register_token(regex(r"max-atom"), move |_| { max_atom_op.clone() });
    let size_atom_op = Atom::gnd(SizeAtomOp{});
    tref.register_token(regex(r"size-atom"), move |_| { size_atom_op.clone() });
    let index_atom_op = Atom::gnd(IndexAtomOp{});
    tref.register_token(regex(r"index-atom"), move |_| { index_atom_op.clone() });
    let unique_op = Atom::gnd(UniqueAtomOp{});
    tref.register_token(regex(r"unique-atom"), move |_| { unique_op.clone() });
    let subtraction_op = Atom::gnd(SubtractionAtomOp{});
    tref.register_token(regex(r"subtraction-atom"), move |_| { subtraction_op.clone() });
    let intersection_op = Atom::gnd(IntersectionAtomOp{});
    tref.register_token(regex(r"intersection-atom"), move |_| { intersection_op.clone() });
    let union_op = Atom::gnd(UnionAtomOp{});
    tref.register_token(regex(r"union-atom"), move |_| { union_op.clone() });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metta::text::SExprParser;
    use crate::metta::runner::EnvBuilder;
    use hyperon_atom::gnd::str::Str;
    use crate::space::grounding::metta_space;
    use crate::metta::runner::run_program;
    use crate::metta::runner::Metta;
    use crate::metta::runner::stdlib::arithmetics::SumOp;
    use hyperon_common::{assert_eq_metta_results, assert_eq_no_order};

    #[test]
    fn metta_car_atom() {
        let result = run_program("!(car-atom (A $b))");
        assert_eq!(result, Ok(vec![vec![expr!("A")]]));
        let result = run_program("!(car-atom ($a B))");
        assert_eq!(result, Ok(vec![vec![expr!(a)]]));
        let result = run_program("!(car-atom ())");
        assert_eq!(result, Ok(vec![vec![expr!("Error" ("car-atom" ()) {Str::from_str("car-atom expects a non-empty expression as an argument")})]]));
        let result = run_program("!(car-atom A)");
        assert_eq!(result, Ok(vec![vec![expr!("Error" ("car-atom" "A") {Str::from_str("car-atom expects a non-empty expression as an argument")})]]));
    }

    #[test]
    fn metta_cdr_atom() {
        assert_eq!(run_program(&format!("!(cdr-atom (a b c))")), Ok(vec![vec![expr!("b" "c")]]));
        assert_eq!(run_program(&format!("!(cdr-atom ($a b $c))")), Ok(vec![vec![expr!("b" c)]]));
        assert_eq!(run_program(&format!("!(cdr-atom ())")), Ok(vec![vec![expr!("Error" ("cdr-atom" ()) {Str::from_str("cdr-atom expects a non-empty expression as an argument")})]]));
        assert_eq!(run_program(&format!("!(cdr-atom a)")), Ok(vec![vec![expr!("Error" ("cdr-atom" "a") {Str::from_str("cdr-atom expects a non-empty expression as an argument")})]]));
        assert_eq!(run_program(&format!("!(cdr-atom $a)")), Ok(vec![vec![expr!("Error" ("cdr-atom" a) {Str::from_str("cdr-atom expects a non-empty expression as an argument")})]]));
    }

    #[test]
    fn metta_size_atom() {
        assert_eq!(run_program(&format!("!(size-atom (5 4 3 2 1))")), Ok(vec![vec![expr!({Number::Integer(5)})]]));
        assert_eq!(run_program(&format!("!(size-atom ())")), Ok(vec![vec![expr!({Number::Integer(0)})]]));
    }

    #[test]
    fn metta_min_atom() {
        assert_eq!(run_program(&format!("!(min-atom (5 4 5.5))")), Ok(vec![vec![expr!({Number::Integer(4)})]]));
        assert_eq!(run_program(&format!("!(min-atom ())")), Ok(vec![vec![expr!("Error" ({ MinAtomOp{} } ()) "Empty expression")]]));
        assert_eq!(run_program(&format!("!(min-atom (3 A B 5))")), Ok(vec![vec![expr!("Error" ({ MinAtomOp{} } ({Number::Integer(3)} "A" "B" {Number::Integer(5)})) "Only numbers are allowed in expression: (3 A B 5)")]]));
        assert_eq!(run_program(&format!("(= (nums) (5 4 5.5)) !(min-atom (nums))")), Ok(vec![vec![expr!({Number::Integer(4)})]]));
    }

    #[test]
    fn metta_max_atom() {
        assert_eq!(run_program(&format!("!(max-atom (5 4 5.5))")), Ok(vec![vec![expr!({Number::Float(5.5)})]]));
        assert_eq!(run_program(&format!("!(max-atom ())")), Ok(vec![vec![expr!("Error" ({ MaxAtomOp{} } ()) "Empty expression")]]));
        assert_eq!(run_program(&format!("!(max-atom (3 A B 5))")), Ok(vec![vec![expr!("Error" ({ MaxAtomOp{} } ({Number::Integer(3)} "A" "B" {Number::Integer(5)})) "Only numbers are allowed in expression: (3 A B 5)")]]));
        assert_eq!(run_program(&format!("(= (nums) (5 4 5.5)) !(max-atom (nums))")), Ok(vec![vec![expr!({Number::Float(5.5)})]]));
    }

    #[test]
    fn metta_index_atom() {
        assert_eq!(run_program(&format!("!(index-atom (5 4 3 2 1) 2)")), Ok(vec![vec![expr!({Number::Integer(3)})]]));
        assert_eq!(run_program(&format!("!(index-atom (A B C D E) 5)")), Ok(vec![vec![expr!("Error" ({ IndexAtomOp{} } ("A" "B" "C" "D" "E") {Number::Integer(5)}) "Index is out of bounds")]]));
    }

    #[test]
    fn metta_filter_atom() {
        assert_eq!(run_program("!(filter-atom () $x (eval (if-error $x False True)))"), Ok(vec![vec![expr!()]]));
        assert_eq!(run_program("!(filter-atom (a (b) $c) $x (eval (if-error $x False True)))"), Ok(vec![vec![expr!("a" ("b") c)]]));
        assert_eq!(run_program("!(filter-atom (a (Error (b) \"Test error\") $c) $x (eval (if-error $x False True)))"), Ok(vec![vec![expr!("a" c)]]));
    }

    #[test]
    fn metta_map_atom() {
        assert_eq!(run_program("!(map-atom () $x ($x mapped))"), Ok(vec![vec![expr!()]]));
        assert_eq!(run_program("!(map-atom (a (b) $c) $x (mapped $x))"), Ok(vec![vec![expr!(("mapped" "a") ("mapped" ("b")) ("mapped" c))]]));
    }

    #[test]
    fn metta_foldl_atom() {
        assert_eq!(run_program("!(foldl-atom () 1 $a $b (eval (+ $a $b)))"), Ok(vec![vec![expr!({Number::Integer(1)})]]));
        assert_eq!(run_program("!(foldl-atom (1 2 3) 0 $a $b (eval (+ $a $b)))"), Ok(vec![vec![expr!({Number::Integer(6)})]]));
    }

    #[test]
    fn size_atom_op() {
        let res = SizeAtomOp{}.execute(&mut vec![expr!({Number::Integer(5)} {Number::Integer(4)} {Number::Integer(3)} {Number::Integer(2)} {Number::Integer(1)})]).expect("No result returned");
        assert_eq!(res, vec![expr!({Number::Integer(5)})]);
        let res = SizeAtomOp{}.execute(&mut vec![expr!()]).expect("No result returned");
        assert_eq!(res, vec![expr!({Number::Integer(0)})]);
    }

    #[test]
    fn min_atom_op() {
        let res = MinAtomOp{}.execute(&mut vec![expr!({Number::Integer(5)} {Number::Integer(4)} {Number::Float(5.5)})]).expect("No result returned");
        assert_eq!(res, vec![expr!({Number::Integer(4)})]);
        let res = MinAtomOp{}.execute(&mut vec![expr!({Number::Integer(5)} {Number::Integer(4)} "A")]);
        assert_eq!(res, Err(ExecError::from("Only numbers are allowed in expression: (5 4 A)")));
        let res = MinAtomOp{}.execute(&mut vec![expr!()]);
        assert_eq!(res, Err(ExecError::from("Empty expression")));
    }

    #[test]
    fn max_atom_op() {
        let res = MaxAtomOp{}.execute(&mut vec![expr!({Number::Integer(5)} {Number::Integer(4)} {Number::Float(5.5)})]).expect("No result returned");
        assert_eq!(res, vec![expr!({Number::Float(5.5)})]);
        let res = MaxAtomOp{}.execute(&mut vec![expr!({Number::Integer(5)} {Number::Integer(4)} "A")]);
        assert_eq!(res, Err(ExecError::from("Only numbers are allowed in expression: (5 4 A)")));
        let res = MaxAtomOp{}.execute(&mut vec![expr!()]);
        assert_eq!(res, Err(ExecError::from("Empty expression")));
        let res = MaxAtomOp{}.execute(&mut vec![expr!({Number::Integer(4)} {Str::from_str("5")})]);
        assert_eq!(res, Err(ExecError::from("Only numbers are allowed in expression: (4 \"5\")")));
    }

    #[test]
    fn index_atom_op() {
        let res = IndexAtomOp{}.execute(&mut vec![expr!({Number::Integer(5)} {Number::Integer(4)} {Number::Integer(3)} {Number::Integer(2)} {Number::Integer(1)}), expr!({Number::Integer(2)})]).expect("No result returned");
        assert_eq!(res, vec![expr!({Number::Integer(3)})]);
        let res = IndexAtomOp{}.execute(&mut vec![expr!({Number::Integer(5)} {Number::Integer(4)} {Number::Integer(3)} {Number::Integer(2)} {Number::Integer(1)}), expr!({Number::Integer(5)})]);
        assert_eq!(res, Err(ExecError::from("Index is out of bounds")));
    }

    #[test]
    fn test_error_is_used_as_an_argument() {
        let metta = Metta::new(Some(EnvBuilder::test_env()));
        let parser = SExprParser::new(r#"
            !(get-type Error)
            !(get-metatype Error)
            !(get-type (Error Foo Boo))
            !(Error (+ 1 2) (+ 1 +))
        "#);

        assert_eq_metta_results!(metta.run(parser), Ok(vec![
            vec![expr!("->" "Atom" "Atom" "ErrorType")],
            vec![expr!("Symbol")],
            vec![expr!("ErrorType")],
            vec![expr!("Error" ({SumOp{}} {Number::Integer(1)} {Number::Integer(2)}) ({SumOp{}} {Number::Integer(1)} {SumOp{}}))],
        ]));
    }


    #[test]
    fn unique_op() {
        let unique_op = UniqueAtomOp{};
        let actual = unique_op.execute(&mut vec![expr!(
                ("A" ("B" "C"))
                ("A" ("B" "C"))
                ("f" "g")
                ("f" "g")
                ("f" "g")
                "Z"
        )]).unwrap();
        assert_eq_no_order!(actual,
                   vec![expr!(("A" ("B" "C")) ("f" "g") "Z")]);
    }

     #[test]  
    fn unique_op_() {  
        let unique_op = UniqueAtomOp{};  
        let yonas = VariableAtom::new("yonas");  
        let tol = VariableAtom::new("tol");  
        let name_var = VariableAtom::new("name");  
    
        // Build the input: ((name $yonas) (name $tol) ($name $tol))  
        let input = Atom::expr([  
            Atom::expr([Atom::sym("name"), Atom::Variable(yonas.clone())]),  
            Atom::expr([Atom::sym("name"), Atom::Variable(tol.clone())]),  
            Atom::expr([Atom::Variable(name_var.clone()), Atom::Variable(tol.clone())])  
        ]);  
    
        let actual = unique_op.execute(&[input]).unwrap();  
        // the old implementation was giving us only ((name $yonas)) which is incorrect but now it is fixed
        // Expected: ((name $yonas) ($name $tol))  
        let expected = vec![Atom::expr([  
            Atom::expr([Atom::sym("name"), Atom::Variable(yonas.clone())]),  
            Atom::expr([Atom::Variable(name_var.clone()), Atom::Variable(tol.clone())])  
        ])];  
    
        assert_eq!(actual, expected);  
    }

    #[test]
    fn union_op() {
        let union_op = UnionAtomOp{};
        let actual = union_op.execute(&mut vec![expr!(
                ("A" ("B" "C"))
                ("A" ("B" "C"))
                ("f" "g")
                ("f" "g")
                ("f" "g")
                "Z"
            ), expr!(
                ("A" ("B" "C"))
                "p"
                "p"
                ("Q" "a")
            )]).unwrap();
        assert_eq_no_order!(actual,
                   vec![expr!(("A" ("B" "C")) ("A" ("B" "C"))
                        ("f" "g") ("f" "g") ("f" "g") "Z"
                        ("A" ("B" "C")) "p" "p" ("Q" "a"))]);
    }

    #[test]
    fn intersection_op() {
        let intersection_op = IntersectionAtomOp{};
        let actual = intersection_op.execute(&mut vec![expr!(
                "Z"
                ("A" ("B" "C"))
                ("A" ("B" "C"))
                ("f" "g")
                ("f" "g")
                ("f" "g")
                ("P" "b")
            ), expr!(
                ("f" "g")
                ("f" "g")
                ("A" ("B" "C"))
                "p"
                "p"
                ("Q" "a")
                "Z"
            )]).unwrap();
        assert_eq_no_order!(actual, vec![expr!("Z" ("A" ("B" "C")) ("f" "g") ("f" "g"))]);

        assert_eq_no_order!(intersection_op.execute(&mut vec![expr!(
                { Number::Integer(5) }
                { Number::Integer(4) }
                { Number::Integer(3) }
                { Number::Integer(2) }
            ), expr!(
                { Number::Integer(5) }
                { Number::Integer(3) }
            )]).unwrap(), vec![expr!({Number::Integer(5)} {Number::Integer(3)})]);
    }

    #[test]
    fn subtraction_op() {
        let subtraction_op = SubtractionAtomOp{};
        let actual = subtraction_op.execute(&mut vec![expr!(
                "Z"
                "S"
                "S"
                ("A" ("B" "C"))
                ("A" ("B" "C"))
                ("f" "g")
                ("f" "g")
                ("f" "g")
                ("P" "b")
            ), expr!(
                ("f" "g")
                ("A" ("B" "C"))
                "p"
                "P"
                ("Q" "a")
                "Z"
                "S"
                "S"
                "S"
            )]).unwrap();
        assert_eq_no_order!(actual,
                   vec![expr!(("A" ("B" "C")) ("f" "g") ("f" "g") ("P" "b"))]);
    }

    #[test]
    fn multiset_ops_preserve_variable_identity_and_left_order() {
        let x = Atom::var("x");
        let other_x = Atom::Variable(VariableAtom::new_id("x", 1));
        let lhs = Atom::expr([x.clone(), sym!("A"), other_x.clone(), x.clone(), sym!("B")]);
        let rhs = Atom::expr([sym!("B"), x.clone(), sym!("A")]);

        assert_eq!(IntersectionAtomOp{}.execute(&[lhs.clone(), rhs.clone()]).unwrap(),
            vec![Atom::expr([x.clone(), sym!("A"), sym!("B")])]);
        assert_eq!(SubtractionAtomOp{}.execute(&[lhs, rhs]).unwrap(),
            vec![Atom::expr([other_x, x])]);
    }

    #[test]
    fn multiset_ops_keep_all_variable_members_in_every_rhs_order() {
        use itertools::Itertools;

        let members = vec![sym!("A"), Atom::var("x"), sym!("B"), Atom::var("y"), sym!("C")];
        let lhs = Atom::expr(members.clone());
        for order in members.into_iter().permutations(5) {
            let rhs = Atom::expr(order);
            assert_eq!(IntersectionAtomOp{}.execute(&[lhs.clone(), rhs.clone()]).unwrap(),
                vec![lhs.clone()], "RHS: {rhs}");
            assert_eq!(SubtractionAtomOp{}.execute(&[lhs.clone(), rhs.clone()]).unwrap(),
                vec![expr!()], "RHS: {rhs}");
        }
    }

    #[test]
    fn multiset_ops_consume_numeric_duplicates_once() {
        let lhs = expr!({Number::Integer(2)} {Number::Float(1.0)} {Number::Integer(1)}
            {Number::Float(-0.0)} {Number::Integer(2)});
        let rhs = expr!({Number::Integer(0)} {Number::Integer(1)} {Number::Integer(2)});

        assert_eq!(IntersectionAtomOp{}.execute(&[lhs.clone(), rhs.clone()]).unwrap(),
            vec![expr!({Number::Integer(2)} {Number::Float(1.0)} {Number::Float(-0.0)})]);
        assert_eq!(SubtractionAtomOp{}.execute(&[lhs, rhs]).unwrap(),
            vec![expr!({Number::Integer(1)} {Number::Integer(2)})]);
    }

    #[test]
    fn multiset_ops_use_grounded_equality_instead_of_serialization() {
        #[derive(Clone, Debug)]
        struct NumericAlias(i64);

        // Equal grounded values may have different numeric encodings.
        impl PartialEq for NumericAlias {
            fn eq(&self, _: &Self) -> bool { true }
        }
        impl std::fmt::Display for NumericAlias {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "alias:{}", self.0)
            }
        }
        impl Grounded for NumericAlias {
            fn type_(&self) -> Atom { ATOM_TYPE_NUMBER }
            fn serialize(&self, serializer: &mut dyn hyperon_atom::serial::Serializer)
                -> hyperon_atom::serial::Result {
                serializer.serialize_i64(self.0)
            }
        }

        let first = Atom::gnd(NumericAlias(1));
        let equal = Atom::gnd(NumericAlias(3));
        assert_eq!(first, equal);
        let lhs = Atom::expr([first.clone(), first]);
        let rhs = Atom::expr([equal]);

        for result in [IntersectionAtomOp{}.execute(&[lhs.clone(), rhs.clone()]).unwrap(),
            SubtractionAtomOp{}.execute(&[lhs, rhs]).unwrap()] {
            let output = <&ExpressionAtom>::try_from(&result[0]).unwrap().children();
            assert_eq!(output.len(), 1);
            assert_eq!(output[0].as_gnd::<NumericAlias>().unwrap().0, 1);
        }
    }

    #[test]
    fn multiset_ops_preserve_empty_and_disjoint_results() {
        for (lhs, rhs, common, remaining) in [
            (expr!(), expr!(), expr!(), expr!()),
            (expr!("A" "A"), expr!(), expr!(), expr!("A" "A")),
            (expr!(), Atom::expr([sym!("A")]), expr!(), expr!()),
            (Atom::expr([sym!("A")]), Atom::expr([sym!("B")]), expr!(), Atom::expr([sym!("A")])),
        ] {
            assert_eq!(IntersectionAtomOp{}.execute(&[lhs.clone(), rhs.clone()]).unwrap(),
                vec![common]);
            assert_eq!(SubtractionAtomOp{}.execute(&[lhs, rhs]).unwrap(), vec![remaining]);
        }
    }

    #[test]
    fn get_type_op() {
        let space = metta_space("
            (: B Type)
            (: C Type)
            (: A B)
            (: A C)
        ");

        let get_type_op = GetTypeOp::new(space.clone());
        assert_eq_no_order!(get_type_op.execute(&mut vec![sym!("A"), expr!({space.clone()})]).unwrap(),
            vec![sym!("B"), sym!("C")]);
    }

    #[test]
    fn get_type_op_non_valid_atom() {
        let space = metta_space("
            (: f (-> Number String))
            (: 42 Number)
            (: \"test\" String)
        ");

        let get_type_op = GetTypeOp::new(space.clone());
        assert_eq_no_order!(get_type_op.execute(&mut vec![expr!("f" "42"), expr!({space.clone()})]).unwrap(),
            vec![sym!("String")]);
        assert_eq_no_order!(get_type_op.execute(&mut vec![expr!("f" "\"test\""), expr!({space.clone()})]).unwrap(),
            vec![EMPTY_SYMBOL]);
    }
}
