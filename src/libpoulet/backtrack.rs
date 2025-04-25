use crate::libpoulet::strategies;

pub fn auto(proof: &strategies::Proof) -> Result<Vec<(usize, usize, strategies::StrategyArg)>, ()> {
    let mut visited_states: Vec<strategies::Proof> = vec![];
    let mut steps: Vec<(usize, usize, strategies::StrategyArg)> = vec![];

    let mut starting_proof = proof.clone();
    starting_proof.clean();

    fn local_backtrack(
        proof: &strategies::Proof,
        visited_states: &mut Vec<strategies::Proof>,
        steps: &mut Vec<(usize, usize, strategies::StrategyArg)>,
    ) -> Result<(), ()> {
        let mut local_proof = proof.clone();
        local_proof.clean();
        if visited_states.contains(&local_proof) {
            println!("{} | already visited", " ".repeat(steps.len()));
            return Err(());
        } else {
            visited_states.push(local_proof.clone());
        }
        if !steps.is_empty() {
            let (_, _, strat) = steps[steps.len() - 1];
            println!("{}{}", " ".repeat(steps.len()), strat)
        }
        if local_proof.goals.is_empty() {
            println!("auto: solved");
            Ok(())
        } else {
            let applicable_strats = local_proof.get_applicable_strategies();
            for elt in applicable_strats {
                let (_, goalnum, strat) = elt;
                let mut loop_proof = local_proof.clone();
                if let Ok(()) = loop_proof.set_active_goal(goalnum) {
                    if let Ok(()) = strat.apply_to(&mut loop_proof) {
                        steps.push(elt);
                        match local_backtrack(&loop_proof, visited_states, steps) {
                            Ok(()) => return Ok(()),
                            Err(()) => continue,
                        }
                    }
                } else {
                    panic!("Inexisting goal selected")
                }
            }
            steps.pop();
            Err(())
        }
    }

    match local_backtrack(proof, &mut visited_states, &mut steps) {
        Ok(()) => Ok(steps),
        Err(()) => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use crate::libpoulet::strategies::Proof;

    use crate::libpoulet::logic::Prop;

    use super::*;

    #[test]
    fn empty() {
        let empty_proof = Proof::new();
        let result = auto(&empty_proof);
        assert_eq!(result, Ok(vec![]))
    }

    #[test]
    fn impossible() {
        let mut proof = Proof::new();
        proof.add_goal_from_prop(Prop::Name(String::from("a")));
        let result = auto(&proof);
        assert_eq!(result, Err(()));
    }

    #[test]
    fn simple_ok() {
        let mut proof = Proof::new();
        proof.add_goal_from_prop(Prop::imply(
            Prop::Name(String::from("a")),
            Prop::Name(String::from("a")),
        ));
        assert_eq!(
            auto(&proof),
            Ok(vec![
                (3, 0, strategies::StrategyArg::Intro),
                (1, 0, strategies::StrategyArg::Exact(0))
            ])
        )
    }

    #[test]
    fn simple_fail() {
        let mut proof = Proof::new();
        proof.add_goal_from_prop(Prop::imply(
            Prop::Name(String::from("a")),
            Prop::Name(String::from("b")),
        ));
        assert_eq!(auto(&proof), Err(()))
    }

    #[test]
    fn already_visited_state_simple() {
        let mut proof = Proof::new();
        proof.add_goal_from_prop(Prop::or(
            Prop::imply(Prop::Name(String::from("a")), Prop::Name(String::from("b"))),
            Prop::imply(Prop::Name(String::from("a")), Prop::Name(String::from("b"))),
        ));
        assert_eq!(auto(&proof), Err(()))
    }
}
