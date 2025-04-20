use std::{
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
    Left,
    Right,
    HypLeft(usize),
    HypRight(usize),
    FalseIsHyp,
    Exact(usize),
    Apply(usize),
    ApplyIn(usize, usize),
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

    pub fn intro(&mut self) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        match self.goals[self.active_goal].0.as_ref().clone() {
            logic::Prop::Implies(a, b) => {
                self.goals[self.active_goal].0 = b.clone();
                self.goals[self.active_goal].1.push(a.clone());
                Ok(())
            }
            _ => Err("Strategy could not be applied"),
        }
    }

    pub fn split(&mut self) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        match self.goals[self.active_goal].0.as_ref() {
            logic::Prop::And(a, b) => {
                let new_goal_a = (a.clone(), self.goals[self.active_goal].1.clone());
                let new_goal_b = (b.clone(), self.goals[self.active_goal].1.clone());
                self.goals[self.active_goal] = new_goal_a;
                self.goals.push(new_goal_b);
                Ok(())
            }
            _ => Err("Strategy could not be applied"),
        }
    }

    pub fn hyp_split(&mut self, i: usize) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        if i >= self.goals[self.active_goal].1.len() {
            return Err("Out of bounds");
        }
        match self.goals[self.active_goal].1[i].as_ref().clone() {
            logic::Prop::And(a, b) => {
                self.goals[self.active_goal].1[i] = a.clone();
                self.goals[self.active_goal].1.push(b.clone());
                Ok(())
            }
            _ => Err("Strategy could not be applied"),
        }
    }

    pub fn or_split(&mut self, left: bool) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        match self.goals[self.active_goal].0.as_ref() {
            logic::Prop::Or(a, b) => {
                let new_goal = if left {
                    (a.clone(), self.goals[self.active_goal].1.clone())
                } else {
                    (b.clone(), self.goals[self.active_goal].1.clone())
                };

                self.goals[self.active_goal] = new_goal;
                Ok(())
            }
            _ => Err("Strategy could not be applied"),
        }
    }

    pub fn left(&mut self) -> Result<(), &str> {
        self.or_split(true)
    }

    pub fn right(&mut self) -> Result<(), &str> {
        self.or_split(false)
    }

    pub fn hyp_or_split(&mut self, left: bool, i: usize) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        if i >= self.goals[self.active_goal].1.len() {
            return Err("Out of bounds");
        }
        match self.goals[self.active_goal].1[i].as_ref() {
            logic::Prop::Or(a, b) => {
                if left {
                    self.goals[self.active_goal].1[i] = a.clone();
                } else {
                    self.goals[self.active_goal].1[i] = b.clone();
                }
                Ok(())
            }
            _ => Err("Strategy could not be applied"),
        }
    }

    pub fn hyp_left(&mut self, i: usize) -> Result<(), &str> {
        self.hyp_or_split(true, i)
    }

    pub fn hyp_right(&mut self, i: usize) -> Result<(), &str> {
        self.hyp_or_split(false, i)
    }

    pub fn false_is_hyp(&mut self) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        for hyp in self.goals[self.active_goal].1.iter() {
            if *(*hyp) == logic::Prop::False {
                self.goals[self.active_goal].0 = Rc::new(logic::Prop::True);
                return Ok(());
            }
        }
        Err("Strategy could not be applied")
    }

    pub fn goal_is_exact_hyp(&mut self, i: usize) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        if i >= self.goals[self.active_goal].1.len() {
            return Err("Out of bounds");
        }
        if self.goals[self.active_goal].1[i] == self.goals[self.active_goal].0 {
            self.goals[self.active_goal].0 = Rc::new(logic::Prop::True);
            Ok(())
        } else {
            Err("Strategy could not be applied")
        }
    }

    pub fn assumption(&mut self) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        let size = self.goals[self.active_goal].1.len();
        for i in 0..size {
            match self.goal_is_exact_hyp(i) {
                Ok(()) => return Ok(()),
                _ => continue,
            }
        }
        Err("Strategy could not be applied")
    }

    pub fn apply(&mut self, i: usize) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        if i >= self.goals[self.active_goal].1.len() {
            return Err("Out of bounds");
        }
        match self.goals[self.active_goal].1[i].as_ref() {
            logic::Prop::Implies(a, b) => {
                if b.as_ref() == self.goals[self.active_goal].0.as_ref() {
                    self.goals[self.active_goal].0 = a.clone();
                    return Ok(());
                }
                Err("Strategy could not be applied")
            }
            _ => Err("Strategy could not be applied"),
        }
    }

    pub fn apply_in_hyp(
        &mut self,
        i_target: usize,
        i_applied: usize,
        keep_old: bool,
    ) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        if i_target >= self.goals[self.active_goal].1.len() {
            return Err("Out of bounds");
        }
        if i_applied >= self.goals[self.active_goal].1.len() {
            return Err("Out of bounds");
        }
        match self.goals[self.active_goal].1[i_applied].as_ref().clone() {
            logic::Prop::Implies(a, b) => {
                let target_prop = &self.goals[self.active_goal].1[i_target];
                if a.as_ref() == target_prop.as_ref() {
                    if keep_old {
                        self.goals[self.active_goal].1.push(b.clone());
                    } else {
                        self.goals[self.active_goal].1[i_target] = b.clone();
                    }
                    Ok(())
                } else {
                    Err("Strategy could not be applied")
                }
            }
            _ => Err("Strategy could not be applied"),
        }
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
                        result.push((3, index_goal, StrategyArg::Right));
                        result.push((4, index_goal, StrategyArg::Left));
                    } else if *b.as_ref() == logic::Prop::False {
                        result.push((3, index_goal, StrategyArg::Left));
                        result.push((4, index_goal, StrategyArg::Right));
                    } else {
                        result.push((3, index_goal, StrategyArg::Left));
                        result.push((3, index_goal, StrategyArg::Right));
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
                            result.push((2, index_goal, StrategyArg::HypLeft(index)));
                            result.push((4, index_goal, StrategyArg::HypRight(index)));
                        } else if *b.as_ref() == logic::Prop::False {
                            result.push((4, index_goal, StrategyArg::HypLeft(index)));
                            result.push((2, index_goal, StrategyArg::HypRight(index)));
                        } else {
                            result.push((4, index_goal, StrategyArg::HypLeft(index)));
                            result.push((4, index_goal, StrategyArg::HypRight(index)));
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
                                result.push((4, index_goal, StrategyArg::ApplyIn(index, i)))
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

pub fn strat_to_string(strat: (usize, usize, StrategyArg)) -> String {
    let (prio, goalnum, strat_name) = strat;
    match strat_name {
        StrategyArg::Intro => format!("prio: {} - goal: {} - intro", prio, goalnum),
        StrategyArg::Split => format!("prio: {} - goal: {} - split", prio, goalnum),
        StrategyArg::HypSplit(arg1) => {
            format!("prio: {} - goal: {} - hyp_split {}", prio, goalnum, arg1)
        }
        StrategyArg::Left => format!("prio: {} - goal: {} - left", prio, goalnum),
        StrategyArg::Right => format!("prio: {} - goal: {} - right", prio, goalnum),
        StrategyArg::HypLeft(arg1) => {
            format!("prio: {} - goal: {} - hyp_left {}", prio, goalnum, arg1)
        }
        StrategyArg::HypRight(arg1) => {
            format!("prio: {} - goal: {} - hyp_right {}", prio, goalnum, arg1)
        }
        StrategyArg::FalseIsHyp => {
            format!("prio: {} - goal: {} - false_is_hyp", prio, goalnum)
        }
        StrategyArg::Exact(arg1) => format!("prio: {} - goal: {} - exact {}", prio, goalnum, arg1),
        StrategyArg::Apply(arg1) => format!("prio: {} - goal: {} - apply {}", prio, goalnum, arg1),
        StrategyArg::ApplyIn(arg1, arg2) => format!(
            "prio: {} - goal: {} - apply_in_hyp {} {}",
            prio, goalnum, arg1, arg2
        ),
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
        assert_eq!(new_proof.intro(), Err("No goal to test strategy"));
        assert_eq!(new_proof.split(), Err("No goal to test strategy"));
        assert_eq!(new_proof.hyp_split(0), Err("No goal to test strategy"));
        assert_eq!(new_proof.left(), Err("No goal to test strategy"));
        assert_eq!(new_proof.right(), Err("No goal to test strategy"));
        assert_eq!(new_proof.hyp_left(0), Err("No goal to test strategy"));
        assert_eq!(new_proof.hyp_right(0), Err("No goal to test strategy"));
        assert_eq!(new_proof.false_is_hyp(), Err("No goal to test strategy"));
        assert_eq!(
            new_proof.goal_is_exact_hyp(0),
            Err("No goal to test strategy")
        );
        assert_eq!(new_proof.assumption(), Err("No goal to test strategy"));
        assert_eq!(new_proof.apply(0), Err("No goal to test strategy"));
        assert_eq!(
            new_proof.apply_in_hyp(0, 0, false),
            Err("No goal to test strategy")
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

        assert_eq!(proof_before.intro(), Ok(()));
        assert_eq!(proof_before, proof_after);
        let _ = proof_before.set_active_goal(1);
        assert_eq!(proof_before.intro(), Err("Strategy could not be applied"));
        let _ = proof_before.set_active_goal(2);
        assert_eq!(proof_before.intro(), Err("Strategy could not be applied"));
        let _ = proof_before.set_active_goal(3);
        assert_eq!(proof_before.intro(), Err("Strategy could not be applied"));
        let _ = proof_before.set_active_goal(4);
        assert_eq!(proof_before.intro(), Err("Strategy could not be applied"));
        let _ = proof_before.set_active_goal(5);
        assert_eq!(proof_before.intro(), Err("Strategy could not be applied"));
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

        assert_eq!(proof_before.split(), Ok(()));
        assert_eq!(proof_before, proof_after);
        assert_eq!(proof_before.split(), Err("Strategy could not be applied"));
        let _ = proof_before.set_active_goal(1);
        assert_eq!(proof_before.split(), Err("Strategy could not be applied"));
        let _ = proof_before.set_active_goal(2);
        assert_eq!(proof_before.split(), Err("Strategy could not be applied"));
        let _ = proof_before.set_active_goal(3);
        assert_eq!(proof_before.split(), Err("Strategy could not be applied"));
        let _ = proof_before.set_active_goal(4);
        assert_eq!(proof_before.split(), Err("Strategy could not be applied"));
        let _ = proof_before.set_active_goal(5);
        assert_eq!(proof_before.split(), Err("Strategy could not be applied"));
        let _ = proof_before.set_active_goal(6);
        assert_eq!(proof_before.split(), Err("Strategy could not be applied"));
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

        assert_eq!(proof_before.hyp_split(0), Ok(()));
        assert_eq!(proof_before, proof_after);
        assert_eq!(
            proof_before.hyp_split(1),
            Err("Strategy could not be applied")
        );
        assert_eq!(
            proof_before.hyp_split(2),
            Err("Strategy could not be applied")
        );
        assert_eq!(
            proof_before.hyp_split(3),
            Err("Strategy could not be applied")
        );
        assert_eq!(
            proof_before.hyp_split(4),
            Err("Strategy could not be applied")
        );
        assert_eq!(
            proof_before.hyp_split(5),
            Err("Strategy could not be applied")
        );
        assert_eq!(
            proof_before.hyp_split(6),
            Err("Strategy could not be applied")
        );
        assert_eq!(proof_before.hyp_split(8), Err("Out of bounds"))
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

        assert_eq!(proof_before_left.left(), Ok(()));
        assert_eq!(proof_before_left, proof_after_left);
        assert_eq!(proof_before_right.right(), Ok(()));
        assert_eq!(proof_before_right, proof_after_right);

        assert_eq!(
            proof_before_left.left(),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(1);
        assert_eq!(
            proof_before_left.left(),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(2);
        assert_eq!(
            proof_before_left.left(),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(3);
        assert_eq!(
            proof_before_left.left(),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(4);
        assert_eq!(
            proof_before_left.left(),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(5);
        assert_eq!(
            proof_before_left.left(),
            Err("Strategy could not be applied")
        );
        let _ = proof_before_left.set_active_goal(6);
        assert_eq!(
            proof_before_left.left(),
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
            vec![(3, 0, StrategyArg::Left), (3, 0, StrategyArg::Right)]
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
            vec![(3, 0, StrategyArg::Left), (4, 0, StrategyArg::Right)]
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
            vec![(3, 0, StrategyArg::Right), (4, 0, StrategyArg::Left)]
        );
    }
}
