use std::cmp::max;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Prop {
    Name(String),
    Implies(Box<Prop>, Box<Prop>),
    And(Box<Prop>, Box<Prop>),
    Or(Box<Prop>, Box<Prop>),
    True,
    False,
}

impl Prop {

    pub fn from_name(name: String) -> Prop {
        Prop::Name(name)
    }
    
    pub fn imply(a: Prop, b: Prop) -> Prop {
        Prop::Implies(Box::new(a), Box::new(b))
    }
    
    pub fn not(prop: Prop) -> Prop {
        Prop::imply(prop, Prop::False)
    }
    
    pub fn and(a: Prop, b: Prop) -> Prop {
        Prop::And(Box::new(a), Box::new(b))
    }
    
    pub fn or(a: Prop, b: Prop) -> Prop {
        Prop::Or(Box::new(a), Box::new(b))
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
                a if !a.is_empty() => acc.push(Prop::from_name(String::from(a))),
                _ => {}
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

    pub fn depth(self: &Prop) -> i32 {
        match self {
            Prop::True | Prop::False | Prop::Name(_) => 1,
            Prop::Implies(a, b) | Prop::And(a, b) | Prop::Or(a, b) => {
                max(a.as_ref().depth(), b.as_ref().depth()) + 1
            }
        }
    }

    pub fn items(self: &Prop) -> i32 {
        match self {
            Prop::True | Prop::False | Prop::Name(_) => 1,
            Prop::Implies(a, b) | Prop::And(a, b) | Prop::Or(a, b) => {
                a.as_ref().items() + b.as_ref().items() + 1
            }
        }
    }
}
