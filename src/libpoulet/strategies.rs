use std::{
    fmt,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    rc::Rc,
};

use crate::libpoulet::logic;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub goals: Vec<(Rc<logic::Prop>, Vec<Rc<logic::Prop>>)>,
    active_goal: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StrategyArg {
    Intro,
    Split,
    HypSplit(usize),
    OrSplit(bool),
    HypOrSplit(usize, bool),
    FalseIsHyp,
    Exact(usize),
    Apply(usize),
    ApplyIn(usize, usize, bool),
}

impl Proof {
    pub fn new() -> Proof {
        Proof {
            goals: vec![],
            active_goal: 0,
        }
    }

    pub fn from_file(path: &str) -> Result<Proof, String> {
        if let Ok(file) = File::open(path) {
            let mut proof = Proof::new();
            let reader = BufReader::new(file);
            for l in reader.lines().map_while(Result::ok) {
                if let Some((verb, param)) = l.split_once(':') {
                    if verb == "G" {
                        match logic::Prop::parse_rpn(param) {
                            Ok(prop) => {
                                proof.add_goal_from_prop(prop);
                                let _ = proof.set_active_goal(proof.number_of_goals() - 1);
                            }
                            Err(msg) => return Err(format!("Error while parsing file: {}", msg)),
                        }
                    } else if verb == "H" {
                        match logic::Prop::parse_rpn(param) {
                            Ok(prop) => {
                                proof.add_hyp_from_prop(prop);
                            }
                            Err(msg) => return Err(format!("Error while parsing file: {}", msg)),
                        }
                    }
                }
            }

            Ok(proof)
        } else {
            Err(format!("failed to open file '{}'", path))
        }
    }

    pub fn to_file(&self, path: &str) -> Result<(), String> {
        if let Ok(file) = File::create(path) {
            let mut file = BufWriter::new(file);
            for goal in self.goals.iter() {
                match file.write_fmt(format_args!("G:{}", goal.0.to_string_rpn())) {
                    Ok(_) => (),
                    Err(_) => return Err(format!("failed to write to file '{}'", path)),
                }
                for hyp in goal.1.iter() {
                    match file.write_fmt(format_args!("H:{}", hyp.to_string_rpn())) {
                        Ok(_) => (),
                        Err(_) => return Err(format!("failed to write to file '{}'", path)),
                    }
                }
            }

            match file.flush() {
                Ok(_) => Ok(()),
                Err(_) => Err(format!("failed to write to file '{}'", path)),
            }
        } else {
            Err(format!("failed to open file '{}'", path))
        }
    }

    pub fn number_of_goals(&self) -> usize {
        self.goals.len()
    }

    pub fn active_goal_index(&self) -> usize {
        self.active_goal
    }

    pub fn set_active_goal(&mut self, i: usize) -> Result<(), &str> {
        if i >= self.goals.len() {
            Err("Out of bounds")
        } else {
            self.active_goal = i;
            Ok(())
        }
    }

    pub fn add_goal_from_prop(&mut self, goal: logic::Prop) {
        self.goals.push((Rc::new(goal), vec![]))
    }

    pub fn add_hyp_from_prop(&mut self, hyp: logic::Prop) {
        self.goals[self.active_goal].1.push(Rc::new(hyp))
    }

    pub fn clean(&mut self) {
        for elt in self.goals.iter_mut() {
            elt.1.sort();
            elt.1.dedup();
            elt.1.retain(|x| *(x.as_ref()) != logic::Prop::True);
        }
        self.goals.sort();
        self.goals.dedup();
        self.goals.retain(|x| *(x.0.as_ref()) != logic::Prop::True);
        self.active_goal = 0
    }

    pub fn execute(&mut self, strat: &StrategyArg) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to execute strategy on.");
        }
        match strat {
            StrategyArg::Intro => {
                if let logic::Prop::Implies(a, b) = self.goals[self.active_goal].0.as_ref().clone()
                {
                    self.goals[self.active_goal].0 = b.clone();
                    self.goals[self.active_goal].1.push(a.clone());
                    return Ok(());
                }
            }
            StrategyArg::Split => {
                if let logic::Prop::And(a, b) = self.goals[self.active_goal].0.as_ref() {
                    let new_goal_a = (a.clone(), self.goals[self.active_goal].1.clone());
                    let new_goal_b = (b.clone(), self.goals[self.active_goal].1.clone());
                    self.goals[self.active_goal] = new_goal_a;
                    self.goals.push(new_goal_b);
                    return Ok(());
                }
            }
            StrategyArg::HypSplit(arg1) => {
                if *arg1 >= self.goals[self.active_goal].1.len() {
                    return Err("Out of bounds");
                }
                if let logic::Prop::And(a, b) =
                    self.goals[self.active_goal].1[*arg1].as_ref().clone()
                {
                    self.goals[self.active_goal].1[*arg1] = a.clone();
                    self.goals[self.active_goal].1.push(b.clone());
                    return Ok(());
                }
            }
            StrategyArg::OrSplit(left) => {
                if let logic::Prop::Or(a, b) = self.goals[self.active_goal].0.as_ref() {
                    let new_goal = if *left {
                        (a.clone(), self.goals[self.active_goal].1.clone())
                    } else {
                        (b.clone(), self.goals[self.active_goal].1.clone())
                    };

                    self.goals[self.active_goal] = new_goal;
                    return Ok(());
                }
            }
            StrategyArg::HypOrSplit(arg1, left) => {
                if *arg1 >= self.goals[self.active_goal].1.len() {
                    return Err("Out of bounds");
                }
                if let logic::Prop::Or(a, b) = self.goals[self.active_goal].1[*arg1].as_ref() {
                    if *left {
                        self.goals[self.active_goal].1[*arg1] = a.clone();
                    } else {
                        self.goals[self.active_goal].1[*arg1] = b.clone();
                    }
                    return Ok(());
                }
            }
            StrategyArg::FalseIsHyp => {
                for hyp in self.goals[self.active_goal].1.iter() {
                    if *(*hyp) == logic::Prop::False {
                        self.goals[self.active_goal].0 = Rc::new(logic::Prop::True);
                        return Ok(());
                    }
                }
            }
            StrategyArg::Exact(arg1) => {
                if *arg1 >= self.goals[self.active_goal].1.len() {
                    return Err("Out of bounds");
                }
                if self.goals[self.active_goal].1[*arg1] == self.goals[self.active_goal].0 {
                    self.goals[self.active_goal].0 = Rc::new(logic::Prop::True);
                    return Ok(());
                }
            }
            StrategyArg::Apply(arg1) => {
                if *arg1 >= self.goals[self.active_goal].1.len() {
                    return Err("Out of bounds");
                }
                if let logic::Prop::Implies(a, b) = self.goals[self.active_goal].1[*arg1].as_ref() {
                    if b.as_ref() == self.goals[self.active_goal].0.as_ref() {
                        self.goals[self.active_goal].0 = a.clone();
                        return Ok(());
                    }
                }
            }
            StrategyArg::ApplyIn(arg1, arg2, keep_old) => {
                if *arg1 >= self.goals[self.active_goal].1.len() {
                    return Err("Out of bounds");
                }
                if *arg2 >= self.goals[self.active_goal].1.len() {
                    return Err("Out of bounds");
                }
                if let logic::Prop::Implies(a, b) =
                    self.goals[self.active_goal].1[*arg2].as_ref().clone()
                {
                    let target_prop = &self.goals[self.active_goal].1[*arg1];
                    if a.as_ref() == target_prop.as_ref() {
                        if *keep_old {
                            self.goals[self.active_goal].1.push(b.clone());
                        } else {
                            self.goals[self.active_goal].1[*arg1] = b.clone();
                        }
                        return Ok(());
                    }
                }
            }
        };
        Err("Strategy could not be applied")
    }

    pub fn get_applicable_strategies(&self) -> Vec<(usize, usize, StrategyArg)> {
        let mut result: Vec<(usize, usize, StrategyArg)> = vec![];
        // elts in list with syntax (prio: usize, goalnum: usize, cmd: string, arg1: usize, arg2: usize])
        for (index_goal, goal) in self.goals.iter().enumerate() {
            match goal.0.as_ref() {
                logic::Prop::True => continue,
                logic::Prop::False => (),
                logic::Prop::Name(_) => (),
                logic::Prop::Implies(_, _) => result.push((3, index_goal, StrategyArg::Intro)),
                logic::Prop::And(_, _) => result.push((3, index_goal, StrategyArg::Split)),
                logic::Prop::Or(a, b) => {
                    if *a.as_ref() == logic::Prop::False {
                        result.push((3, index_goal, StrategyArg::OrSplit(false)));
                        result.push((4, index_goal, StrategyArg::OrSplit(true)));
                    } else if *b.as_ref() == logic::Prop::False {
                        result.push((3, index_goal, StrategyArg::OrSplit(true)));
                        result.push((4, index_goal, StrategyArg::OrSplit(false)));
                    } else {
                        result.push((3, index_goal, StrategyArg::OrSplit(true)));
                        result.push((3, index_goal, StrategyArg::OrSplit(false)));
                    }
                }
            };
            let num_hyps = goal.1.len();
            for (index, hyp) in goal.1.iter().enumerate() {
                match hyp.as_ref() {
                    logic::Prop::True => {}
                    logic::Prop::False => result.push((0, index_goal, StrategyArg::FalseIsHyp)),
                    logic::Prop::Name(_) => {}
                    logic::Prop::Implies(a, b) => {
                        if b.as_ref() == goal.0.as_ref() {
                            if goal.1.contains(a) {
                                result.push((2, index_goal, StrategyArg::Apply(index)));
                            } else {
                                result.push((4, index_goal, StrategyArg::Apply(index)));
                            }
                        }
                    }
                    logic::Prop::And(_, _) => {
                        result.push((4, index_goal, StrategyArg::HypSplit(index)));
                    }
                    logic::Prop::Or(a, b) => {
                        if *a.as_ref() == logic::Prop::False {
                            result.push((2, index_goal, StrategyArg::HypOrSplit(index, true)));
                            result.push((4, index_goal, StrategyArg::HypOrSplit(index, false)));
                        } else if *b.as_ref() == logic::Prop::False {
                            result.push((4, index_goal, StrategyArg::HypOrSplit(index, true)));
                            result.push((2, index_goal, StrategyArg::HypOrSplit(index, false)));
                        } else {
                            result.push((4, index_goal, StrategyArg::HypOrSplit(index, true)));
                            result.push((4, index_goal, StrategyArg::HypOrSplit(index, false)));
                        }
                    }
                };
                if hyp.as_ref() == goal.0.as_ref() {
                    result.push((1, index_goal, StrategyArg::Exact(index)));
                }
                for i in 0..num_hyps {
                    if i != index {
                        if let logic::Prop::Implies(a, _) = goal.1[i].as_ref() {
                            if a.as_ref() == hyp.as_ref() {
                                result.push((4, index_goal, StrategyArg::ApplyIn(index, i, true)))
                            }
                        }
                    }
                }
            }
        }
        result.sort();
        result
    }
}

impl fmt::Display for StrategyArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StrategyArg::Intro => write!(f, "intro"),
            StrategyArg::Split => write!(f, "split"),
            StrategyArg::HypSplit(arg1) => {
                write!(f, "hyp_split {}", arg1)
            }
            StrategyArg::OrSplit(true) => write!(f, "left"),
            StrategyArg::OrSplit(false) => write!(f, "right"),
            StrategyArg::HypOrSplit(arg1, true) => {
                write!(f, "hyp_left {}", arg1)
            }
            StrategyArg::HypOrSplit(arg1, false) => {
                write!(f, "hyp_right {}", arg1)
            }
            StrategyArg::FalseIsHyp => {
                write!(f, "false_is_hyp")
            }
            StrategyArg::Exact(arg1) => write!(f, "exact {}", arg1),
            StrategyArg::Apply(arg1) => write!(f, "apply {}", arg1),
            StrategyArg::ApplyIn(arg1, arg2, _) => write!(f, "apply_in_hyp_keep {} {}", arg1, arg2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logic::Prop;

    #[test]
    fn new() {
        let mut new_proof = Proof::new();
        assert_eq!(
            new_proof,
            Proof {
                goals: vec![],
                active_goal: 0
            }
        );
        assert_eq!(new_proof.number_of_goals(), 0);
        assert_eq!(new_proof.active_goal_index(), 0);
        assert_eq!(new_proof.set_active_goal(0), Err("Out of bounds"));
        assert_eq!(new_proof.set_active_goal(1), Err("Out of bounds"));
        assert_eq!(
            new_proof.execute(&StrategyArg::Intro),
            Err("No goal to execute strategy on.")
        );
    }

    #[test]
    fn add_goals() {
        let mut proof = Proof::new();

        proof.add_goal_from_prop(Prop::True);
        assert_eq!(proof.number_of_goals(), 1);
        assert_eq!(proof.active_goal_index(), 0);
        assert_eq!(proof.set_active_goal(0), Ok(()));
        assert_eq!(proof.set_active_goal(1), Err("Out of bounds"));
        assert_eq!(proof.goals, vec![(Rc::new(Prop::True), vec![])]);

        proof.add_goal_from_prop(Prop::False);
        assert_eq!(proof.number_of_goals(), 2);
        assert_eq!(proof.active_goal_index(), 0);
        assert_eq!(proof.set_active_goal(0), Ok(()));
        assert_eq!(proof.set_active_goal(1), Ok(()));
        assert_eq!(
            proof.goals,
            vec![
                (Rc::new(Prop::True), vec![]),
                (Rc::new(Prop::False), vec![])
            ]
        );
    }

    #[test]
    fn clean() {
        let mut proof = Proof {
            goals: vec![
                (Rc::new(Prop::False), vec![]),
                (Rc::new(Prop::True), vec![]),
                (Rc::new(Prop::from_name(String::from("name"))), vec![]),
                (Rc::new(Prop::True), vec![]),
                (
                    Rc::new(Prop::imply(Prop::True, Prop::False)),
                    vec![
                        Rc::new(Prop::from_name(String::from("b"))),
                        Rc::new(Prop::from_name(String::from("a"))),
                        Rc::new(Prop::True),
                    ],
                ),
                (
                    Rc::new(Prop::False),
                    vec![
                        Rc::new(Prop::from_name(String::from("b"))),
                        Rc::new(Prop::from_name(String::from("a"))),
                        Rc::new(Prop::True),
                    ],
                ),
            ],
            active_goal: 0,
        };
        let cleaned_proof = Proof {
            goals: vec![
                (Rc::new(Prop::from_name(String::from("name"))), vec![]),
                (
                    Rc::new(Prop::imply(Prop::True, Prop::False)),
                    vec![
                        Rc::new(Prop::from_name(String::from("a"))),
                        Rc::new(Prop::from_name(String::from("b"))),
                    ],
                ),
                (Rc::new(Prop::False), vec![]),
                (
                    Rc::new(Prop::False),
                    vec![
                        Rc::new(Prop::from_name(String::from("a"))),
                        Rc::new(Prop::from_name(String::from("b"))),
                    ],
                ),
            ],
            active_goal: 0,
        };

        proof.clean();

        assert_eq!(proof, cleaned_proof);
    }

    #[test]
    fn intro() {
        let mut proof_before = Proof {
            goals: vec![
                (
                    Rc::new(Prop::imply(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
                (Rc::new(Prop::True), vec![]),
                (Rc::new(Prop::False), vec![]),
                (Rc::new(Prop::from_name(String::from("name"))), vec![]),
                (
                    Rc::new(Prop::and(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
                (
                    Rc::new(Prop::or(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
            ],
            active_goal: 0,
        };

        let proof_after = Proof {
            goals: vec![
                (
                    Rc::new(Prop::from_name(String::from("b"))),
                    vec![Rc::new(Prop::from_name(String::from("a")))],
                ),
                (Rc::new(Prop::True), vec![]),
                (Rc::new(Prop::False), vec![]),
                (Rc::new(Prop::from_name(String::from("name"))), vec![]),
                (
                    Rc::new(Prop::and(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
                (
                    Rc::new(Prop::or(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
            ],
            active_goal: 0,
        };

        assert_eq!(proof_before.execute(&StrategyArg::Intro), Ok(()));
        assert_eq!(proof_before, proof_after);
        let _ = proof_before.set_active_goal(1);
        assert_eq!(
            proof_before.execute(&StrategyArg::Intro),
            Err("Strategy could not be applied")
        );
        let _ = proof_before.set_active_goal(2);
        assert_eq!(
            proof_before.execute(&StrategyArg::Intro),
            Err("Strategy could not be applied")
        );
        let _ = proof_before.set_active_goal(3);
        assert_eq!(
            proof_before.execute(&StrategyArg::Intro),
            Err("Strategy could not be applied")
        );
        let _ = proof_before.set_active_goal(4);
        assert_eq!(
            proof_before.execute(&StrategyArg::Intro),
            Err("Strategy could not be applied")
        );
        let _ = proof_before.set_active_goal(5);
        assert_eq!(
            proof_before.execute(&StrategyArg::Intro),
            Err("Strategy could not be applied")
        );
    }

    #[test]
    fn split() {
        let mut proof_before = Proof {
            goals: vec![
                (
                    Rc::new(Prop::and(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
                (Rc::new(Prop::True), vec![]),
                (Rc::new(Prop::False), vec![]),
                (Rc::new(Prop::from_name(String::from("name"))), vec![]),
                (
                    Rc::new(Prop::imply(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
                (
                    Rc::new(Prop::or(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
            ],
            active_goal: 0,
        };

        let proof_after = Proof {
            goals: vec![
                (Rc::new(Prop::from_name(String::from("a"))), vec![]),
                (Rc::new(Prop::True), vec![]),
                (Rc::new(Prop::False), vec![]),
                (Rc::new(Prop::from_name(String::from("name"))), vec![]),
                (
                    Rc::new(Prop::imply(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
                (
                    Rc::new(Prop::or(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
                (Rc::new(Prop::from_name(String::from("b"))), vec![]),
            ],
            active_goal: 0,
        };

        assert_eq!(proof_before.execute(&StrategyArg::Split), Ok(()));
        assert_eq!(proof_before, proof_after);
        assert_eq!(
            proof_before.execute(&StrategyArg::Split),
            Err("Strategy could not be applied")
        );
        let _ = proof_before.set_active_goal(1);
        assert_eq!(
            proof_before.execute(&StrategyArg::Split),
            Err("Strategy could not be applied")
        );
        let _ = proof_before.set_active_goal(2);
        assert_eq!(
            proof_before.execute(&StrategyArg::Split),
            Err("Strategy could not be applied")
        );
        let _ = proof_before.set_active_goal(3);
        assert_eq!(
            proof_before.execute(&StrategyArg::Split),
            Err("Strategy could not be applied")
        );
        let _ = proof_before.set_active_goal(4);
        assert_eq!(
            proof_before.execute(&StrategyArg::Split),
            Err("Strategy could not be applied")
        );
        let _ = proof_before.set_active_goal(5);
        assert_eq!(
            proof_before.execute(&StrategyArg::Split),
            Err("Strategy could not be applied")
        );
        let _ = proof_before.set_active_goal(6);
        assert_eq!(
            proof_before.execute(&StrategyArg::Split),
            Err("Strategy could not be applied")
        );
    }

    #[test]
    fn hyp_split() {
        let mut proof_before = Proof {
            goals: vec![(
                Rc::new(Prop::False),
                vec![
                    Rc::new(Prop::and(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    Rc::new(Prop::True),
                    Rc::new(Prop::False),
                    Rc::new(Prop::from_name(String::from("name"))),
                    Rc::new(Prop::imply(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    Rc::new(Prop::or(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                ],
            )],
            active_goal: 0,
        };

        let proof_after = Proof {
            goals: vec![(
                Rc::new(Prop::False),
                vec![
                    Rc::new(Prop::from_name(String::from("a"))),
                    Rc::new(Prop::True),
                    Rc::new(Prop::False),
                    Rc::new(Prop::from_name(String::from("name"))),
                    Rc::new(Prop::imply(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    Rc::new(Prop::or(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    Rc::new(Prop::from_name(String::from("b"))),
                ],
            )],
            active_goal: 0,
        };

        assert_eq!(proof_before.execute(&StrategyArg::HypSplit(0)), Ok(()));
        assert_eq!(proof_before, proof_after);
        assert_eq!(
            proof_before.execute(&StrategyArg::HypSplit(1)),
            Err("Strategy could not be applied")
        );
        assert_eq!(
            proof_before.execute(&StrategyArg::HypSplit(2)),
            Err("Strategy could not be applied")
        );
        assert_eq!(
            proof_before.execute(&StrategyArg::HypSplit(3)),
            Err("Strategy could not be applied")
        );
        assert_eq!(
            proof_before.execute(&StrategyArg::HypSplit(4)),
            Err("Strategy could not be applied")
        );
        assert_eq!(
            proof_before.execute(&StrategyArg::HypSplit(5)),
            Err("Strategy could not be applied")
        );
        assert_eq!(
            proof_before.execute(&StrategyArg::HypSplit(6)),
            Err("Strategy could not be applied")
        );
        assert_eq!(
            proof_before.execute(&StrategyArg::HypSplit(8)),
            Err("Out of bounds")
        )
    }

    #[test]
    fn or_split() {
        let mut proof_before_left = Proof {
            goals: vec![
                (
                    Rc::new(Prop::or(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
                (Rc::new(Prop::True), vec![]),
                (Rc::new(Prop::False), vec![]),
                (Rc::new(Prop::from_name(String::from("name"))), vec![]),
                (
                    Rc::new(Prop::imply(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
                (
                    Rc::new(Prop::and(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
            ],
            active_goal: 0,
        };

        let proof_after_left = Proof {
            goals: vec![
                (Rc::new(Prop::from_name(String::from("a"))), vec![]),
                (Rc::new(Prop::True), vec![]),
                (Rc::new(Prop::False), vec![]),
                (Rc::new(Prop::from_name(String::from("name"))), vec![]),
                (
                    Rc::new(Prop::imply(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
                (
                    Rc::new(Prop::and(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
            ],
            active_goal: 0,
        };

        let mut proof_before_right = proof_before_left.clone();

        let proof_after_right = Proof {
            goals: vec![
                (Rc::new(Prop::from_name(String::from("b"))), vec![]),
                (Rc::new(Prop::True), vec![]),
                (Rc::new(Prop::False), vec![]),
                (Rc::new(Prop::from_name(String::from("name"))), vec![]),
                (
                    Rc::new(Prop::imply(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
                (
                    Rc::new(Prop::and(
                        Prop::from_name(String::from("a")),
                        Prop::from_name(String::from("b")),
                    )),
                    vec![],
                ),
            ],
            active_goal: 0,
        };

        assert_eq!(
            proof_before_left.execute(&StrategyArg::OrSplit(true)),
            Ok(())
        );
        assert_eq!(proof_before_left, proof_after_left);
        assert_eq!(
            proof_before_right.execute(&StrategyArg::OrSplit(false)),
            Ok(())
        );
        assert_eq!(proof_before_right, proof_after_right);

        assert_eq!(
            proof_before_left.execute(&StrategyArg::OrSplit(true)),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(1);
        assert_eq!(
            proof_before_left.execute(&StrategyArg::OrSplit(true)),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(2);
        assert_eq!(
            proof_before_left.execute(&StrategyArg::OrSplit(true)),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(3);
        assert_eq!(
            proof_before_left.execute(&StrategyArg::OrSplit(true)),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(4);
        assert_eq!(
            proof_before_left.execute(&StrategyArg::OrSplit(true)),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(5);
        assert_eq!(
            proof_before_left.execute(&StrategyArg::OrSplit(true)),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(6);
        assert_eq!(
            proof_before_left.execute(&StrategyArg::OrSplit(true)),
            Err("Strategy could not be applied")
        );
    }

    #[test]
    fn applicable_strategies() {
        let empty_proof = Proof::new();
        assert_eq!(empty_proof.get_applicable_strategies(), vec![]);

        let only_name = Proof {
            goals: vec![(Rc::new(Prop::Name(String::from("a"))), vec![])],
            active_goal: 0,
        };
        assert_eq!(only_name.get_applicable_strategies(), vec![]);

        let only_true = Proof {
            goals: vec![(Rc::new(Prop::True), vec![])],
            active_goal: 0,
        };
        assert_eq!(only_true.get_applicable_strategies(), vec![]);

        let only_false = Proof {
            goals: vec![(Rc::new(Prop::False), vec![])],
            active_goal: 0,
        };
        assert_eq!(only_false.get_applicable_strategies(), vec![]);

        let one_intro = Proof {
            goals: vec![(
                Rc::new(Prop::imply(
                    Prop::Name(String::from("a")),
                    Prop::Name(String::from("b")),
                )),
                vec![],
            )],
            active_goal: 0,
        };
        assert_eq!(
            one_intro.get_applicable_strategies(),
            vec![(3, 0, StrategyArg::Intro)]
        );

        let one_split = Proof {
            goals: vec![(
                Rc::new(Prop::and(
                    Prop::Name(String::from("a")),
                    Prop::Name(String::from("b")),
                )),
                vec![],
            )],
            active_goal: 0,
        };
        assert_eq!(
            one_split.get_applicable_strategies(),
            vec![(3, 0, StrategyArg::Split)]
        );

        let left_right_no_false = Proof {
            goals: vec![(
                Rc::new(Prop::or(
                    Prop::Name(String::from("a")),
                    Prop::Name(String::from("b")),
                )),
                vec![],
            )],
            active_goal: 0,
        };
        assert_eq!(
            left_right_no_false.get_applicable_strategies(),
            vec![
                (3, 0, StrategyArg::OrSplit(false)),
                (3, 0, StrategyArg::OrSplit(true))
            ]
        );

        let left_right_false = Proof {
            goals: vec![(
                Rc::new(Prop::or(Prop::Name(String::from("a")), Prop::False)),
                vec![],
            )],
            active_goal: 0,
        };
        assert_eq!(
            left_right_false.get_applicable_strategies(),
            vec![
                (3, 0, StrategyArg::OrSplit(true)),
                (4, 0, StrategyArg::OrSplit(false))
            ]
        );

        let left_false_right = Proof {
            goals: vec![(
                Rc::new(Prop::or(Prop::False, Prop::Name(String::from("b")))),
                vec![],
            )],
            active_goal: 0,
        };
        assert_eq!(
            left_false_right.get_applicable_strategies(),
            vec![
                (3, 0, StrategyArg::OrSplit(false)),
                (4, 0, StrategyArg::OrSplit(true))
            ]
        );
    }
}
