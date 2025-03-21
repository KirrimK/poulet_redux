use crate::pouletlib::logic;

#[derive(Clone, Debug)]
pub struct Proof {
    goals: Vec<(logic::Prop, Vec<logic::Prop>)>,
    active_goal: usize,
}

impl Proof {
    pub fn new() -> Proof {
        Proof {
            goals: vec![],
            active_goal: 0,
        }
    }
    // str as return type not the best for error types: cf. crate thiserror
    pub fn set_active_goal(&mut self, i: usize) -> Result<(), &str> {
        if i >= self.goals.len() {
            Err("Out of bounds")
        } else {
            self.active_goal = i;
            Ok(())
        }
    }

    pub fn add_goal_from_prop(&mut self, goal: logic::Prop) {
        self.goals.push((goal, vec![]))
    }

    pub fn clean(&mut self) {
        for elt in self.goals.iter_mut() {
            elt.1.sort()
        }
        self.goals.sort();
        self.goals.dedup();
        self.goals.retain(|x| x.0 != logic::Prop::True);
        self.active_goal = 0
    }

    pub fn intro(&mut self) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        let active_goal = self.goals[self.active_goal].clone();
        match &active_goal.0 {
            logic::Prop::Implies(a, b) => {
                self.goals[self.active_goal].0 = b.as_ref().clone();
                self.goals[self.active_goal].1.push(a.as_ref().clone()); 
                Ok(())
            },
            _ => Err("Strategy could not be applied")
        }
    }

    pub fn split(&mut self) -> Result<(), &str> {
        if self.goals.is_empty() {
            return Err("No goal to test strategy");
        }
        let active_goal = self.goals[self.active_goal].clone();
        match &active_goal.0 {
            logic::Prop::And(a, b) => {
                let new_goal_a = (a.as_ref().clone(), active_goal.1.clone());
                let new_goal_b = (b.as_ref().clone(), active_goal.1.clone());
                self.goals[self.active_goal] = new_goal_a;
                self.goals.push(new_goal_b);
                Ok(())
            },
            _ => Err("Strategy could not be applied")
        }
    }

}
