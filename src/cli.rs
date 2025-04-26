use std::{io::Write, process::exit};

use crate::libpoulet::backtrack;
use crate::libpoulet::logic;
use crate::libpoulet::strategies;

fn parse_input<'a>(
    proof: &'a mut strategies::Proof,
    prevs: &mut Vec<strategies::Proof>,
    input: &'a str,
) -> Result<u8, &'a str> {
    let input = input.trim();
    match input.split_once(char::is_whitespace) {
        Some(("load", rest)) => {
            let path = rest.trim();
            match strategies::Proof::from_file(path) {
                Ok(loaded_proof) => {
                    *proof = loaded_proof;
                    Ok(1)
                }
                Err(msg) => Err(msg.leak()),
            }
        }
        Some(("save", rest)) => {
            let path = rest.trim();
            match proof.to_file(path) {
                Ok(_) => Ok(1),
                Err(msg) => Err(msg.leak()),
            }
        }
        Some(("add_goal_rpn", rest)) => match logic::Prop::parse_rpn(rest) {
            Ok(prop) => {
                proof.add_goal_from_prop(prop);
                Ok(1)
            }
            Err(msg) => Err(msg),
        },
        Some(("set_active", rest)) => match rest.trim().parse::<usize>() {
            Ok(0) => Err("Invalid argument"),
            Ok(goal_num) => match proof.set_active_goal(goal_num - 1) {
                Ok(()) => Ok(1),
                Err(msg) => Err(msg),
            },
            Err(_) => Err("Invalid argument"),
        },
        Some(("hyp_split", rest)) => match rest.trim().parse::<usize>() {
            Ok(goal_num) => match proof.execute(&strategies::StrategyArg::HypSplit(goal_num)) {
                Ok(()) => Ok(1),
                Err(msg) => Err(msg),
            },
            Err(_) => Err("Invalid argument"),
        },
        Some(("hyp_left", rest)) => match rest.trim().parse::<usize>() {
            Ok(goal_num) => {
                match proof.execute(&strategies::StrategyArg::HypOrSplit(goal_num, true)) {
                    Ok(()) => Ok(1),
                    Err(msg) => Err(msg),
                }
            }
            Err(_) => Err("Invalid argument"),
        },
        Some(("hyp_right", rest)) => match rest.trim().parse::<usize>() {
            Ok(goal_num) => {
                match proof.execute(&strategies::StrategyArg::HypOrSplit(goal_num, false)) {
                    Ok(()) => Ok(1),
                    Err(msg) => Err(msg),
                }
            }
            Err(_) => Err("Invalid argument"),
        },
        Some(("exact", rest)) => match rest.trim().parse::<usize>() {
            Ok(goal_num) => match proof.execute(&strategies::StrategyArg::Exact(goal_num)) {
                Ok(()) => Ok(1),
                Err(msg) => Err(msg),
            },
            Err(_) => Err("Invalid argument"),
        },
        Some(("apply", rest)) => match rest.trim().parse::<usize>() {
            Ok(goal_num) => match proof.execute(&strategies::StrategyArg::Apply(goal_num)) {
                Ok(()) => Ok(1),
                Err(msg) => Err(msg),
            },
            Err(_) => Err("Invalid argument"),
        },
        Some(("apply_in", rest)) => match rest.trim().split_once(char::is_whitespace) {
            Some((first, second)) => {
                match (
                    first.trim().parse::<usize>(),
                    second.trim().parse::<usize>(),
                ) {
                    (Ok(first_num), Ok(second_num)) => {
                        match proof.execute(&strategies::StrategyArg::ApplyIn(
                            first_num, second_num, false,
                        )) {
                            Ok(()) => Ok(1),
                            Err(msg) => Err(msg),
                        }
                    }
                    (Ok(_), _) => Err("first argument incorrect: <hyp id (0..N) target>"),
                    (_, Ok(_)) => Err("second argument incorrect: <hyp id (0..N) to apply>"),
                    (_, _) => {
                        Err("arguments incorrect: <hyp id (0..N) target> <hyp id (0..N) to apply>")
                    }
                }
            }
            None => Err("missing argument: <hyp id (0..N) to apply>"),
        },
        Some(("apply_in_keep", rest)) => match rest.trim().split_once(char::is_whitespace) {
            Some((first, second)) => {
                match (
                    first.trim().parse::<usize>(),
                    second.trim().parse::<usize>(),
                ) {
                    (Ok(first_num), Ok(second_num)) => {
                        match proof.execute(&strategies::StrategyArg::ApplyIn(
                            first_num, second_num, true,
                        )) {
                            Ok(()) => Ok(1),
                            Err(msg) => Err(msg),
                        }
                    }
                    (Ok(_), _) => Err("first argument incorrect: <hyp id (0..N) target>"),
                    (_, Ok(_)) => Err("second argument incorrect: <hyp id (0..N) to apply>"),
                    (_, _) => {
                        Err("arguments incorrect: <hyp id (0..N) target> <hyp id (0..N) to apply>")
                    }
                }
            }
            None => Err("missing argument: <hyp id (0..N) to apply>"),
        },
        Some((_, _)) => Err("Unknown command"),
        None => match input {
            "quit" => Ok(0),
            "info" => {
                if proof.goals.is_empty() {
                    println!("No goals to display info about.")
                } else {
                    println!(
                        "Active goal proposition has {} items and is of depth {}.",
                        proof.goals[proof.active_goal_index()].0.items(),
                        proof.goals[proof.active_goal_index()].0.depth()
                    )
                }
                Ok(1)
            }
            "purge" => {
                *proof = strategies::Proof::new();
                *prevs = vec![];
                Ok(1)
            }
            "back" => match prevs.pop() {
                Some(a) => {
                    *proof = a;
                    Ok(2)
                }
                None => Err("Cannot go back further"),
            },
            "auto" => match backtrack::auto(proof) {
                Ok(steps) => {
                    println!("Solved using auto:");
                    for (_, goalnum, strat) in steps {
                        println!("    goal: {} - {}", goalnum, strat);
                    }
                    *proof = strategies::Proof::new();
                    Ok(1)
                }
                Err(()) => Err("Could not solve using auto"),
            },
            "intro" => match proof.execute(&strategies::StrategyArg::Intro) {
                Ok(()) => Ok(1),
                Err(msg) => Err(msg),
            },
            "clean" => {
                proof.clean();
                Ok(1)
            }
            "split" => match proof.execute(&strategies::StrategyArg::Split) {
                Ok(()) => Ok(1),
                Err(msg) => Err(msg),
            },
            "left" => match proof.execute(&strategies::StrategyArg::OrSplit(true)) {
                Ok(()) => Ok(1),
                Err(msg) => Err(msg),
            },
            "right" => match proof.execute(&strategies::StrategyArg::OrSplit(false)) {
                Ok(()) => Ok(1),
                Err(msg) => Err(msg),
            },
            "false" => match proof.execute(&strategies::StrategyArg::FalseIsHyp) {
                Ok(()) => Ok(1),
                Err(msg) => Err(msg),
            },
            "add_goal_rpn" => Err("missing argument: <proposition rpn format>"),
            "set_active" => Err("missing argument: <goal index (1..N)>"),
            "hyp_split" | "hyp_left" | "hyp_right" | "exact" | "apply" => {
                Err("missing argument: <hyp id (0..N)>")
            }
            "apply_in" => Err("missing arguments: <hyp id (0..N) target> <hyp id (0..N) to apply>"),
            _ => Err("Unknown command"),
        },
    }
}

pub fn repl() {
    let mut proof = strategies::Proof::new();
    let mut prevs: Vec<strategies::Proof> = vec![];
    loop {
        if proof.goals.is_empty() {
            println!("Goals: None")
        } else {
            for (index, goal) in proof.goals.iter().enumerate() {
                if index == proof.active_goal_index() {
                    for (index_hyp, hyp) in goal.1.iter().enumerate() {
                        println!(" Hyp {} : {}", index_hyp, hyp.to_string());
                    }
                    println!("-----");
                    println!(" Goal : {}", goal.0.to_string());
                }
            }
            println!(
                "Goals: {} (active: n°{}) ",
                proof.number_of_goals(),
                proof.active_goal_index() + 1
            );
            println!("Applicable Strategies: ");
            for (prio, goalnum, strat) in proof.get_applicable_strategies() {
                println!("    prio: {} goal: {} - {}", prio, goalnum, strat);
            }
        }

        print!("> ");
        let _ = std::io::stdout().flush();
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("Unable to read from stdin");
        let proof_before = proof.clone();
        match parse_input(&mut proof, &mut prevs, &buffer) {
            Ok(flag) => {
                if flag == 0 {
                    exit(0);
                } else if flag == 1 {
                    prevs.push(proof_before);
                }
            }
            Err(msg) => {
                println!("{}: {}", buffer.trim(), msg);
            }
        }
    }
}
