use crate::libpoulet::strategies;

pub fn auto(
    proof: &strategies::Proof,
) -> Result<Vec<(usize, usize, strategies::Strategies, usize, usize)>, ()> {
    let mut visited_states: Vec<strategies::Proof> = vec![];
    let mut steps: Vec<(usize, usize, strategies::Strategies, usize, usize)> = vec![];

    let mut starting_proof = proof.clone();
    starting_proof.clean();

    fn local_backtrack(
        proof: &strategies::Proof,
        visited_states: &mut Vec<strategies::Proof>,
        steps: &mut Vec<(usize, usize, strategies::Strategies, usize, usize)>,
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
            println!(
                "{}{}",
                " ".repeat(steps.len()),
                strategies::strat_to_string(steps[steps.len() - 1])
            )
        }
        if local_proof.goals.is_empty() {
            println!("auto: solved");
            Ok(())
        } else {
            let applicable_strats = local_proof.get_applicable_strategies();
            for elt in applicable_strats {
                let (_, goalnum, strat_name, arg1, arg2) = elt;
                let mut loop_proof = local_proof.clone();
                if let Ok(()) = loop_proof.set_active_goal(goalnum) {
                    match strat_name {
                        strategies::Strategies::Intro => {
                            if let Ok(()) = loop_proof.intro() {
                                steps.push(elt);
                                match local_backtrack(&loop_proof, visited_states, steps) {
                                    Ok(()) => return Ok(()),
                                    Err(()) => continue,
                                }
                            }
                        }
                        strategies::Strategies::Split => {
                            if let Ok(()) = loop_proof.split() {
                                steps.push(elt);
                                match local_backtrack(&loop_proof, visited_states, steps) {
                                    Ok(()) => return Ok(()),
                                    Err(()) => continue,
                                }
                            }
                        }
                        strategies::Strategies::HypSplit => {
                            if let Ok(()) = loop_proof.hyp_split(arg1) {
                                steps.push(elt);
                                match local_backtrack(&loop_proof, visited_states, steps) {
                                    Ok(()) => return Ok(()),
                                    Err(()) => continue,
                                }
                            }
                        }
                        strategies::Strategies::Left => {
                            if let Ok(()) = loop_proof.left() {
                                steps.push(elt);
                                match local_backtrack(&loop_proof, visited_states, steps) {
                                    Ok(()) => return Ok(()),
                                    Err(()) => continue,
                                }
                            }
                        }
                        strategies::Strategies::Right => {
                            if let Ok(()) = loop_proof.right() {
                                steps.push(elt);
                                match local_backtrack(&loop_proof, visited_states, steps) {
                                    Ok(()) => return Ok(()),
                                    Err(()) => continue,
                                }
                            }
                        }
                        strategies::Strategies::HypLeft => {
                            if let Ok(()) = loop_proof.hyp_left(arg1) {
                                steps.push(elt);
                                match local_backtrack(&loop_proof, visited_states, steps) {
                                    Ok(()) => return Ok(()),
                                    Err(()) => continue,
                                }
                            }
                        }
                        strategies::Strategies::HypRight => {
                            if let Ok(()) = loop_proof.hyp_right(arg1) {
                                steps.push(elt);
                                match local_backtrack(&loop_proof, visited_states, steps) {
                                    Ok(()) => return Ok(()),
                                    Err(()) => continue,
                                }
                            }
                        }
                        strategies::Strategies::FalseIsHyp => {
                            if let Ok(()) = loop_proof.false_is_hyp() {
                                steps.push(elt);
                                match local_backtrack(&loop_proof, visited_states, steps) {
                                    Ok(()) => return Ok(()),
                                    Err(()) => continue,
                                }
                            }
                        }
                        strategies::Strategies::Exact => {
                            if let Ok(()) = loop_proof.goal_is_exact_hyp(arg1) {
                                steps.push(elt);
                                match local_backtrack(&loop_proof, visited_states, steps) {
                                    Ok(()) => return Ok(()),
                                    Err(()) => continue,
                                }
                            }
                        }
                        strategies::Strategies::Apply => {
                            if let Ok(()) = loop_proof.apply(arg1) {
                                steps.push(elt);
                                match local_backtrack(&loop_proof, visited_states, steps) {
                                    Ok(()) => return Ok(()),
                                    Err(()) => continue,
                                }
                            }
                        }
                        strategies::Strategies::ApplyIn => {
                            if let Ok(()) = loop_proof.apply_in_hyp(arg1, arg2, true) {
                                steps.push(elt);
                                match local_backtrack(&loop_proof, visited_states, steps) {
                                    Ok(()) => return Ok(()),
                                    Err(()) => continue,
                                }
                            }
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
