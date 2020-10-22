extern crate tito;
extern crate serde_json;
extern crate clap;
extern crate lazy_static;
extern crate log;

use log::{info, warn, error, LevelFilter};
use clap::{App, Arg, SubCommand, AppSettings};
use tito::{Tito, TitoLogger, Competitor, Settings, Arena, Evaluation};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref LOGGER: TitoLogger = TitoLogger::new();
}

fn main() {
    let matches = App::new("tito")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("build")
            .about("Builds the file with the answers to the problems to be evaluated")
            .arg(Arg::with_name("settings")
                .required(true)
                .takes_value(true)
                .short("s")
                .long("settings")
            )
        ).subcommand(SubCommand::with_name("run")
            .about("Runs tito")
            .arg(Arg::with_name("arena")
                .short("a")
                .long("arena")
                .required(true)
                .takes_value(true)
            ).arg(Arg::with_name("competitor")
                .short("c")
                .long("competitor")
                .required(true)
                .multiple(true)
                .takes_value(true)
            )
        ).get_matches();

    log::set_logger(&(*LOGGER)).map(|()| log::set_max_level(LevelFilter::Info)).unwrap();
    info!("Beep Boop \u{1f916}");

    match matches.subcommand() {
        ("build", Some(sub_m)) => {
            let settings_path = sub_m.value_of("settings").unwrap();
            // We check if there are indeed settings in the path
            let settings: Settings = match File::open(&settings_path) {
                Ok(mut f) => {
                    let mut content = String::new();
                    f.read_to_string(&mut content);
                    let value: serde_json::Value =  match serde_json::from_str(&content) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("Could not load json from file, {}", e);
                            return;
                        }
                    };
                    match serde_json::from_value(value) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("Could not load settings, {}", e);
                            return;
                        }
                    }
                },
                Err(e) => {
                    error!("Could not load settings, {}", e);
                    return;
                }
            };

            let mut t = match Tito::new() {
                Ok(v) => v,
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };
            let arena = match t.build(settings) {
                Ok(v) => v,
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };
            // We write the arena to an arena.json file
            match serde_json::to_string_pretty(&arena) {
                Ok(v) => match File::create("arena.json") {
                    Ok(mut f) => {
                        match f.write_all(v.as_bytes()) {
                            Ok(_) => (),
                            Err(e) => {
                                error!("{}", e);
                                return;
                            }
                        }
                    },
                    Err(e) => {
                        error!("{}", e);
                        return;
                    }
                },
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            }
            info!("Arena saved to ./arena.json");
        },
        ("run", Some(sub_m)) => {
            let competitors_string = sub_m.values_of("competitor").unwrap();
            let mut competitors = Vec::new();
            let mut competitor_dir = HashMap::new();
            for competitor_string in competitors_string {
                let tokens: Vec<&str> = competitor_string.split(":").collect();
                if tokens.len() != 3 {
                    error!("Competitor must be described as id:path_to_files:path_for_result");
                    return;
                }
                if tokens[0].len() == 0 {
                    error!("Every competitor needs an id.");
                    return;
                }
                if tokens[1].len() == 0 {
                    error!("Every competitor needs a path to the files");
                    return;
                }
                let competitor = Competitor {
                    id: tokens[0].into(),
                    files: tokens[1].into(),
                    result: if tokens[2].len() == 0 { None } else { Some(tokens[2].into()) }
                };
                competitor_dir.insert(competitor.id.clone(), competitor.result.clone());
                competitors.push(competitor);
            }

            let mut t = match Tito::new() {
                Ok(v) => v,
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };

            let arena: Arena = match File::open(sub_m.value_of("arena").unwrap()) {
                Ok(mut f) => {
                    let mut content = String::new();
                    match f.read_to_string(&mut content) {
                        Ok(_) => (),
                        Err(e) => {
                            error!("{}", e);
                            return;
                        }
                    }
                    match serde_json::from_str(&content) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("{}", e);
                            return;
                        }
                    }
                },
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };

            match t.run(competitors, arena.clone()) {
                Ok(v) => {
                    let content = match serde_json::to_string_pretty(&v) {
                        Ok(s) => s,
                        Err(e) => {
                            error!("{}", e);
                            return;
                        }
                    };
                    match File::create("result.json") {
                        Ok(mut f) => {
                            match f.write_all(content.as_bytes()) {
                                Ok(_) => (),
                                Err(e) => {
                                    error!("{}", e);
                                    return;
                                }
                            }
                        },
                        Err(e) => {
                            error!("{}", e);
                            return;
                        }
                    }
                    info!("Result save to result.json");

                    for (user_name, problems) in v.iter() {
                        if let Some(dir) = competitor_dir.get(user_name) {
                            if let Some(dir) = dir {
                                info!("Adding result to {}", user_name);
                                let mut report: String = "Beep boop! Tus resultados están aquí:\n\n".into();
                                for (p_name, evaluation) in problems.iter() {
                                    if let Some(problem) = arena.problems.get(p_name) {
                                        match evaluation {
                                            Evaluation::Grade{score} => {
                                                report += &format!("-> Problema \"{}\": {}\n", p_name, score*10.0);
                                            },
                                            Evaluation::RunError => {
                                                report += &format!("-> Problema \"{}\": Problema de ejecución/compilación\n", p_name);
                                            },
                                            Evaluation::NoFile => {
                                                report += &format!("-> Problema \"{}\": No se encontró el archivo \"{}\"\n", p_name, problem.filename);
                                            }
                                        }
                                    } else {
                                        error!("Problem data not found");
                                        return;
                                    }
                                }
                                let mut result_file = PathBuf::from(dir.clone());
                                result_file.push("resultados.txt");
                                match File::create(result_file) {
                                    Ok(mut f) => {
                                        match f.write_all(report.as_bytes()) {
                                            Ok(_) => (),
                                            Err(e) => warn!("Could not write report for user \"{}\", {}", user_name, e)
                                        };
                                    },
                                    Err(e) => warn!("Could not create report for user \"{}\", {}", user_name, e) 
                                }
                            } else {
                                info!("No place to store results for user \"{}\"", user_name);
                            }
                        } else {
                            warn!("User does not seem to exist again \"{}\"", user_name);
                        }
                    }
                },
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };
        },
        _ => {
            panic!("Impossible");
        }, 
    }
}

/*
T      T
|      |
--------
|O    O|
|  --  |
==0==0==
   ||
*/