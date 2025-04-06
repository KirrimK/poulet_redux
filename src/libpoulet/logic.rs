// use std::cmp::max;

use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Prop {
    Name(String),
    Implies(Rc<Prop>, Rc<Prop>),
    And(Rc<Prop>, Rc<Prop>),
    Or(Rc<Prop>, Rc<Prop>),
    True,
    False,
}

impl Prop {
    pub fn from_name(name: String) -> Prop {
        Prop::Name(name)
    }

    pub fn imply(a: Prop, b: Prop) -> Prop {
        Prop::Implies(Rc::new(a), Rc::new(b))
    }

    pub fn not(prop: Prop) -> Prop {
        Prop::imply(prop, Prop::False)
    }

    pub fn and(a: Prop, b: Prop) -> Prop {
        Prop::And(Rc::new(a), Rc::new(b))
    }

    pub fn or(a: Prop, b: Prop) -> Prop {
        Prop::Or(Rc::new(a), Rc::new(b))
    }

    pub fn equiv(a: Prop, b: Prop) -> Prop {
        Prop::and(Prop::imply(a.clone(), b.clone()), Prop::imply(b, a))
    }

    pub fn parse_rpn(s: &str) -> Result<Prop, &str> {
        let mut acc: Vec<Prop> = vec![];
        for string in s.split_whitespace() {
            match string {
                "=>" => {
                    let b = acc.pop();
                    let a = acc.pop();
                    match (a, b) {
                        (Some(thing_a), Some(thing_b)) => acc.push(Prop::imply(thing_a, thing_b)),
                        (None, Some(_)) => {
                            return Err(
                                "During parsing of '=>', two items expected in accumulator, found one",
                            );
                        }
                        _ => {
                            return Err(
                                "During parsing of '=>', two items expected in accumulator, found zero",
                            );
                        }
                    }
                }
                "^" => {
                    let b = acc.pop();
                    let a = acc.pop();
                    match (a, b) {
                        (Some(thing_a), Some(thing_b)) => acc.push(Prop::and(thing_a, thing_b)),
                        (None, Some(_)) => {
                            return Err(
                                "During parsing of '^', two items expected in accumulator, found one",
                            );
                        }
                        _ => {
                            return Err(
                                "During parsing of '^', two items expected in accumulator, found zero",
                            );
                        }
                    }
                }
                "|" => {
                    let b = acc.pop();
                    let a = acc.pop();
                    match (a, b) {
                        (Some(thing_a), Some(thing_b)) => acc.push(Prop::or(thing_a, thing_b)),
                        (None, Some(_)) => {
                            return Err(
                                "During parsing of '|', two items expected in accumulator, found one",
                            );
                        }
                        _ => {
                            return Err(
                                "During parsing of '|', two items expected in accumulator, found zero",
                            );
                        }
                    }
                }
                "<=>" => {
                    let b = acc.pop();
                    let a = acc.pop();
                    match (a, b) {
                        (Some(thing_a), Some(thing_b)) => acc.push(Prop::equiv(thing_a, thing_b)),
                        (None, Some(_)) => {
                            return Err(
                                "During parsing of '<=>', two items expected in accumulator, found one",
                            );
                        }
                        _ => {
                            return Err(
                                "During parsing of '<=>', two items expected in accumulator, found zero",
                            );
                        }
                    }
                }
                "!" => {
                    let a = acc.pop();
                    match a {
                        None => {
                            return Err(
                                "During parsing of '!', one item expected in accumulator, found zero",
                            );
                        }
                        Some(thing) => acc.push(Prop::not(thing)),
                    }
                }
                "T" => acc.push(Prop::True),
                "F" => acc.push(Prop::False),
                name => acc.push(Prop::from_name(String::from(name))),
            }
        }
        if acc.len() > 1 {
            return Err("At the end of parsing, > 1 item left unused in the accumulator");
        }
        match acc.pop() {
            None => Err("At the end of parsing, accumulator is empty"),
            Some(thing) => Ok(thing),
        }
    }

    pub fn to_string(self: &Prop) -> String {
        match self {
            Prop::True => String::from("T"),
            Prop::False => String::from("F"),
            Prop::Name(name) => name.clone(),
            Prop::Implies(a, b) => {
                let str_a = a.as_ref().to_string();
                let str_b = b.as_ref().to_string();
                format!("( {str_a} => {str_b} )")
            }
            Prop::And(a, b) => {
                let str_a = a.as_ref().to_string();
                let str_b = b.as_ref().to_string();
                format!("( {str_a} ^ {str_b} )")
            }
            Prop::Or(a, b) => {
                let str_a = a.as_ref().to_string();
                let str_b = b.as_ref().to_string();
                format!("( {str_a} | {str_b} )")
            }
        }
    }

    pub fn to_string_rpn(self: &Prop) -> String {
        match self {
            Prop::True => String::from("T"),
            Prop::False => String::from("F"),
            Prop::Name(name) => name.clone(),
            Prop::Implies(a, b) => {
                let str_a = a.as_ref().to_string_rpn();
                let str_b = b.as_ref().to_string_rpn();
                format!("{str_a} {str_b} =>")
            }
            Prop::And(a, b) => {
                let str_a = a.as_ref().to_string_rpn();
                let str_b = b.as_ref().to_string_rpn();
                format!("{str_a} {str_b} ^")
            }
            Prop::Or(a, b) => {
                let str_a = a.as_ref().to_string_rpn();
                let str_b = b.as_ref().to_string_rpn();
                format!("{str_a} {str_b} |")
            }
        }
    }

    // pub fn depth(self: &Prop) -> usize {
    //     match self {
    //         Prop::True | Prop::False | Prop::Name(_) => 1,
    //         Prop::Implies(a, b) | Prop::And(a, b) | Prop::Or(a, b) => {
    //             max(a.as_ref().depth(), b.as_ref().depth()) + 1
    //         }
    //     }
    // }

    // pub fn items(self: &Prop) -> usize {
    //     match self {
    //         Prop::True | Prop::False | Prop::Name(_) => 1,
    //         Prop::Implies(a, b) | Prop::And(a, b) | Prop::Or(a, b) => {
    //             a.as_ref().items() + b.as_ref().items() + 1
    //         }
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_base() {
        let true_a = Prop::True;
        let true_b = Prop::True;
        let false_a = Prop::False;
        let false_b = Prop::False;
        let name_a = Prop::Name(String::from("name"));
        let name_b = Prop::Name(String::from("name"));
        assert_eq!(true_a, true_b);
        assert_eq!(false_a, false_b);
        assert_eq!(name_a, name_b);
    }

    #[test]
    fn eq_nested() {
        let imply_a = Prop::Implies(Rc::new(Prop::True), Rc::new(Prop::True));
        let imply_b = Prop::Implies(Rc::new(Prop::True), Rc::new(Prop::True));
        let and_a = Prop::And(
            Rc::new(Prop::True),
            Rc::new(Prop::Name(String::from("name"))),
        );
        let and_b = Prop::And(
            Rc::new(Prop::True),
            Rc::new(Prop::Name(String::from("name"))),
        );
        let or_a = Prop::Or(
            Rc::new(Prop::False),
            Rc::new(Prop::Name(String::from("name"))),
        );
        let or_b = Prop::Or(
            Rc::new(Prop::False),
            Rc::new(Prop::Name(String::from("name"))),
        );
        let nested_a = Prop::Implies(
            Rc::new(Prop::Implies(Rc::new(Prop::True), Rc::new(Prop::True))),
            Rc::new(Prop::False),
        );
        let nested_b = Prop::Implies(
            Rc::new(Prop::Implies(Rc::new(Prop::True), Rc::new(Prop::True))),
            Rc::new(Prop::False),
        );
        assert_eq!(imply_a, imply_b);
        assert_eq!(and_a, and_b);
        assert_eq!(or_a, or_b);
        assert_eq!(nested_a, nested_b);
    }

    #[test]
    fn constructors() {
        let name = Prop::from_name(String::from("name"));
        assert_eq!(name, Prop::Name(String::from("name")));

        let imply = Prop::imply(Prop::True, Prop::False);
        assert_eq!(
            imply,
            Prop::Implies(Rc::new(Prop::True), Rc::new(Prop::False))
        );

        let and = Prop::and(Prop::True, Prop::False);
        assert_eq!(and, Prop::And(Rc::new(Prop::True), Rc::new(Prop::False)));

        let or = Prop::or(Prop::True, Prop::False);
        assert_eq!(or, Prop::Or(Rc::new(Prop::True), Rc::new(Prop::False)));

        let not = Prop::not(Prop::True);
        assert_eq!(
            not,
            Prop::Implies(Rc::new(Prop::True), Rc::new(Prop::False))
        );
    }

    // #[test]
    // fn depth() {
    //     let p_true = Prop::True;
    //     let p_false = Prop::False;
    //     let p_name = Prop::from_name(String::from("name"));

    //     assert_eq!(p_true.depth(), 1);
    //     assert_eq!(p_false.depth(), 1);
    //     assert_eq!(p_name.depth(), 1);

    //     let p_imply_a = Prop::imply(Prop::True, Prop::False);
    //     let p_and_a = Prop::and(Prop::True, Prop::False);
    //     let p_or_a = Prop::or(Prop::True, Prop::False);

    //     assert_eq!(p_imply_a.depth(), 2);
    //     assert_eq!(p_and_a.depth(), 2);
    //     assert_eq!(p_or_a.depth(), 2);

    //     let p_complex = Prop::imply(
    //         Prop::imply(Prop::not(Prop::False), Prop::and(Prop::True, Prop::False)),
    //         Prop::or(Prop::from_name(String::from("name")), Prop::False),
    //     );
    //     assert_eq!(p_complex.depth(), 4);
    // }

    // #[test]
    // fn items() {
    //     let p_true = Prop::True;
    //     let p_false = Prop::False;
    //     let p_name = Prop::from_name(String::from("name"));

    //     assert_eq!(p_true.items(), 1);
    //     assert_eq!(p_false.items(), 1);
    //     assert_eq!(p_name.items(), 1);

    //     let p_imply_a = Prop::imply(Prop::True, Prop::False);
    //     let p_and_a = Prop::and(Prop::True, Prop::False);
    //     let p_or_a = Prop::or(Prop::True, Prop::False);

    //     assert_eq!(p_imply_a.items(), 3);
    //     assert_eq!(p_and_a.items(), 3);
    //     assert_eq!(p_or_a.items(), 3);

    //     let p_complex = Prop::imply(
    //         Prop::imply(Prop::not(Prop::False), Prop::and(Prop::True, Prop::False)),
    //         Prop::or(Prop::from_name(String::from("name")), Prop::False),
    //     );
    //     assert_eq!(p_complex.items(), 11);
    // }

    #[test]
    fn parse_rpn() {
        let rpn_true = "T";
        let rpn_false = "F";
        let rpn_name = "name";

        assert_eq!(Prop::parse_rpn(rpn_true), Ok(Prop::True));
        assert_eq!(Prop::parse_rpn(rpn_false), Ok(Prop::False));
        assert_eq!(
            Prop::parse_rpn(rpn_name),
            Ok(Prop::from_name(String::from("name")))
        );

        let rpn_simple_imply = "a b =>";
        let rpn_simple_and = "a b ^";
        let rpn_simple_or = "a b |";
        let rpn_simple_not = "a !";

        assert_eq!(
            Prop::parse_rpn(rpn_simple_imply),
            Ok(Prop::imply(
                Prop::from_name(String::from("a")),
                Prop::from_name(String::from("b"))
            ))
        );
        assert_eq!(
            Prop::parse_rpn(rpn_simple_and),
            Ok(Prop::and(
                Prop::from_name(String::from("a")),
                Prop::from_name(String::from("b"))
            ))
        );
        assert_eq!(
            Prop::parse_rpn(rpn_simple_or),
            Ok(Prop::or(
                Prop::from_name(String::from("a")),
                Prop::from_name(String::from("b"))
            ))
        );
        assert_eq!(
            Prop::parse_rpn(rpn_simple_not),
            Ok(Prop::imply(Prop::from_name(String::from("a")), Prop::False))
        );

        let rpn_complex = "a b => c ^ d d => |";
        assert_eq!(
            Prop::parse_rpn(rpn_complex),
            Ok(Prop::or(
                Prop::and(
                    Prop::imply(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b"))
                    ),
                    Prop::from_name(String::from("c"))
                ),
                Prop::imply(
                    Prop::from_name(String::from("d")),
                    Prop::from_name(String::from("d"))
                )
            ))
        )
    }

    #[test]
    fn parse_rpn_bad() {
        assert_eq!(
            Prop::parse_rpn(""),
            Err("At the end of parsing, accumulator is empty")
        );
        assert_eq!(
            Prop::parse_rpn("a b"),
            Err("At the end of parsing, > 1 item left unused in the accumulator")
        );

        assert_eq!(
            Prop::parse_rpn("a =>"),
            Err("During parsing of '=>', two items expected in accumulator, found one")
        );
        assert_eq!(
            Prop::parse_rpn("=>"),
            Err("During parsing of '=>', two items expected in accumulator, found zero")
        );

        assert_eq!(
            Prop::parse_rpn("a ^"),
            Err("During parsing of '^', two items expected in accumulator, found one")
        );
        assert_eq!(
            Prop::parse_rpn("^"),
            Err("During parsing of '^', two items expected in accumulator, found zero")
        );

        assert_eq!(
            Prop::parse_rpn("a |"),
            Err("During parsing of '|', two items expected in accumulator, found one")
        );
        assert_eq!(
            Prop::parse_rpn("|"),
            Err("During parsing of '|', two items expected in accumulator, found zero")
        );

        assert_eq!(
            Prop::parse_rpn("!"),
            Err("During parsing of '!', one item expected in accumulator, found zero")
        );
    }

    #[test]
    fn strings() {
        assert_eq!(Prop::True.to_string(), "T");
        assert_eq!(Prop::False.to_string(), "F");
        assert_eq!(Prop::Name(String::from("name")).to_string(), "name");

        assert_eq!(
            Prop::imply(Prop::Name(String::from("a")), Prop::Name(String::from("b"))).to_string(),
            "( a => b )"
        );
        assert_eq!(
            Prop::and(Prop::Name(String::from("a")), Prop::Name(String::from("b"))).to_string(),
            "( a ^ b )"
        );
        assert_eq!(
            Prop::or(Prop::Name(String::from("a")), Prop::Name(String::from("b"))).to_string(),
            "( a | b )"
        );
    }

    #[test]
    fn strings_rpn() {
        assert_eq!(Prop::True.to_string_rpn(), "T");
        assert_eq!(Prop::False.to_string_rpn(), "F");
        assert_eq!(Prop::Name(String::from("name")).to_string_rpn(), "name");

        assert_eq!(
            Prop::imply(Prop::Name(String::from("a")), Prop::Name(String::from("b")))
                .to_string_rpn(),
            "a b =>"
        );
        assert_eq!(
            Prop::and(Prop::Name(String::from("a")), Prop::Name(String::from("b"))).to_string_rpn(),
            "a b ^"
        );
        assert_eq!(
            Prop::or(Prop::Name(String::from("a")), Prop::Name(String::from("b"))).to_string_rpn(),
            "a b |"
        );
    }
}
