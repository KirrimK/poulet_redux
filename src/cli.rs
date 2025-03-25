use std::{io::Write, process::exit};

use crate::libpoulet::logic;
use crate::libpoulet::strategies;

fn parse_input<'a>(proof: &'a mut strategies::Proof, input: &'a str) -> Result<bool, &'a str> {
    let input = input.trim();
    match input.split_once(char::is_whitespace) {
        Some(("add_goal_rpn", rest)) => match logic::Prop::parse_rpn(rest) {
            Ok(prop) => {
                proof.add_goal_from_prop(prop);
                Ok(false)
            }
            Err(msg) => Err(msg),
        },
        Some(("set_active", rest)) => match rest.trim().parse::<usize>() {
            Ok(0) => Err("Invalid argument"),
            Ok(goal_num) => match proof.set_active_goal(goal_num - 1) {
                Ok(()) => Ok(false),
                Err(msg) => Err(msg),
            },
            Err(_) => Err("Invalid argument"),
        },
        Some(("hyp_split", rest)) => match rest.trim().parse::<usize>() {
            Ok(goal_num) => match proof.hyp_split(goal_num) {
                Ok(()) => Ok(false),
                Err(msg) => Err(msg),
            },
            Err(_) => Err("Invalid argument"),
        },
        Some(("hyp_left", rest)) => match rest.trim().parse::<usize>() {
            Ok(goal_num) => match proof.hyp_left(goal_num) {
                Ok(()) => Ok(false),
                Err(msg) => Err(msg),
            },
            Err(_) => Err("Invalid argument"),
        },
        Some(("hyp_right", rest)) => match rest.trim().parse::<usize>() {
            Ok(goal_num) => match proof.hyp_right(goal_num) {
                Ok(()) => Ok(false),
                Err(msg) => Err(msg),
            },
            Err(_) => Err("Invalid argument"),
        },
        Some(("exact", rest)) => match rest.trim().parse::<usize>() {
            Ok(goal_num) => match proof.goal_is_exact_hyp(goal_num) {
                Ok(()) => Ok(false),
                Err(msg) => Err(msg),
            },
            Err(_) => Err("Invalid argument"),
        },
        Some(("apply", rest)) => match rest.trim().parse::<usize>() {
            Ok(goal_num) => match proof.apply(goal_num) {
                Ok(()) => Ok(false),
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
                        match proof.apply_in_hyp(first_num, second_num, false) {
                            Ok(()) => Ok(false),
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
                        match proof.apply_in_hyp(first_num, second_num, true) {
                            Ok(()) => Ok(false),
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
            "quit" => Ok(true),
            "purge" => {
                *proof = strategies::Proof::new();
                Ok(false)
            }
            "intro" => match proof.intro() {
                Ok(()) => Ok(false),
                Err(msg) => Err(msg),
            },
            "clean" => {
                proof.clean();
                Ok(false)
            }
            "split" => match proof.split() {
                Ok(()) => Ok(false),
                Err(msg) => Err(msg),
            },
            "left" => match proof.left() {
                Ok(()) => Ok(false),
                Err(msg) => Err(msg),
            },
            "right" => match proof.right() {
                Ok(()) => Ok(false),
                Err(msg) => Err(msg),
            },
            "false" => match proof.false_is_hyp() {
                Ok(()) => Ok(false),
                Err(msg) => Err(msg),
            },
            "assumption" => match proof.assumption() {
                Ok(()) => Ok(false),
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
        }

        print!("> ");
        let _ = std::io::stdout().flush();
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("Unable to read from stdin");
        match parse_input(&mut proof, &buffer) {
            Ok(quit) => {
                if quit {
                    exit(0);
                }
            }
            Err(msg) => {
                println!("{}: {}", buffer.trim(), msg);
            }
        }
    }
}
