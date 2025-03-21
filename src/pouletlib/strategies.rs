use std::rc::Rc;

use crate::pouletlib::logic;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub goals: Vec<(Rc<logic::Prop>, Vec<Rc<logic::Prop>>)>,
    active_goal: usize,
}

impl Proof {
    pub fn new() -> Proof {
        Proof {
            goals: vec![],
            active_goal: 0,
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
                if b.as_ref() == target_prop.as_ref() {
                    if keep_old {
                        self.goals[self.active_goal].1.push(a.clone());
                    } else {
                        self.goals[self.active_goal].1[i_target] = a.clone();
                    }
                    Ok(())
                } else {
                    Err("Strategy could not be applied")
                }
            }
            _ => Err("Strategy could not be applied"),
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
        assert_eq!(new_proof.intro(), Err("No goal to test strategy"));
        assert_eq!(new_proof.split(), Err("No goal to test strategy"));
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
            goals: vec![(
                Rc::new(Prop::imply(
                    Prop::from_name(String::from("a")),
                    Prop::from_name(String::from("b")),
                )),
                vec![],
            )],
            active_goal: 0,
        };

        let proof_after = Proof {
            goals: vec![(
                Rc::new(Prop::from_name(String::from("b"))),
                vec![Rc::new(Prop::from_name(String::from("a")))],
            )],
            active_goal: 0,
        };

        assert_eq!(proof_before.intro(), Ok(()));
        assert_eq!(proof_before, proof_after);
    }
}
