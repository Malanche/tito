extern crate tito;
extern crate serde_json;
extern crate clap;
extern crate lazy_static;
extern crate log;

use clap::{Parser};
use tito::{Tito, SimpleLogger, Competitor, Settings, Arena, Evaluation};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
enum Args {
    #[clap(about = "build subcommand to precompute the answers to the problems")]
    Build(BuildArgs),
    #[clap(about = "run subcommand for executing the robot")]
    Run(RunArgs)
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct BuildArgs {
   #[clap(long, help = "path to the location of the problem configuration file")]
   settings: Option<String>,
   #[clap(long, help = "generates a very basic example config and a very basic shell problem")]
   example_config: bool
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct RunArgs {
   #[clap(long, help = "path to the location of the arena file")]
   arena: String,
   #[clap(long, help = "competitor list")]
   competitor: Vec<String>
}

fn main() {
    let matches = Args::parse();
    
    SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();
    log::info!("Beep Boop \u{1f916}");

    match matches {
        Args::Build(build_args) => {
            if build_args.example_config {
                match serde_json::to_string_pretty(&Settings::example()) {
                    Ok(content) => {
                        match File::create("./settings.json") {
                            Ok(mut f) => {
                                match f.write_all(content.as_bytes()) {
                                    Ok(_) => log::info!("example settings written to `settings.json`"),
                                    Err(e) => {
                                        log::error!("{}", e);
                                    }
                                }
                                // Now we write the example problem
                                match File::create("problem-a.sh".to_string()) {
                                    Ok(mut f) => {
                                        match f.write_all(b"echo \"Hello, ${1}!\"\n") {
                                            Ok(_) => log::info!("example poblem written to `problem-a.sh`"),
                                            Err(e) => {
                                                log::error!("{}", e);
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        log::error!("{}", e);
                                    }
                                }
                            },
                            Err(e) => {
                                log::error!("{}", e);
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            } else {
                let settings_path = build_args.settings.unwrap_or("./settings.json".to_string());
                // We check if there are indeed settings in the path
                let settings: Settings = match File::open(&settings_path) {
                    Ok(mut f) => {
                        let mut content = String::new();
                        match f.read_to_string(&mut content) {
                            Ok(_) => (),
                            Err(e) => {
                                log::error!("{}", e);
                                return;
                            }
                        }
                        let value: serde_json::Value =  match serde_json::from_str(&content) {
                            Ok(v) => v,
                            Err(e) => {
                                log::error!("Could not load json from file, {}", e);
                                return;
                            }
                        };
                        match serde_json::from_value(value) {
                            Ok(v) => v,
                            Err(e) => {
                                log::error!("Could not load settings, {}", e);
                                return;
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("Could not load settings, {}", e);
                        return;
                    }
                };
    
                let mut t = match Tito::new() {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("{}", e);
                        return;
                    }
                };
    
                let arena = match t.build(settings) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("{}", e);
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
                                    log::error!("{}", e);
                                    return;
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("{}", e);
                            return;
                        }
                    },
                    Err(e) => {
                        log::error!("{}", e);
                        return;
                    }
                }
                log::info!("Arena saved to ./arena.json");
            }
        },
        Args::Run(run_args) => {
            let competitors: Vec<_> = match run_args.competitor.iter().map(|competitor_string| {
                let tokens: Vec<&str> = competitor_string.split(":").collect();
                if tokens.len() != 3 {
                    return Err("Competitors must be described as id:path_to_files:path_for_result");
                }
                if tokens[0].len() == 0 {
                    return Err("Every competitor needs an id.");
                }
                if tokens[1].len() == 0 {
                    return Err("Every competitor needs a path to the files");
                }
                Ok(Competitor {
                    id: tokens[0].into(),
                    files: tokens[1].into(),
                    result: if tokens[2].len() == 0 { None } else { Some(tokens[2].into()) }
                })
            }).collect::<Result<_, _>>() {
                Ok(v) => v,
                Err(e) => {
                    log::error!("{}", e);
                    return;
                }
            };
            
            let mut tito = match Tito::new() {
                Ok(v) => v,
                Err(e) => {
                    log::error!("{}", e);
                    return;
                }
            };

            let arena: Arena = match File::open(run_args.arena) {
                Ok(mut f) => {
                    let mut content = String::new();
                    match f.read_to_string(&mut content) {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!("{}", e);
                            return;
                        }
                    }
                    match serde_json::from_str(&content) {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("{}", e);
                            return;
                        }
                    }
                },
                Err(e) => {
                    log::error!("{}", e);
                    return;
                }
            };

            match tito.run(competitors.clone(), arena.clone()) {
                Ok(results) => {
                    let content = match serde_json::to_string_pretty(&results) {
                        Ok(s) => s,
                        Err(e) => {
                            log::error!("{}", e);
                            return;
                        }
                    };
                    match File::create("result.json") {
                        Ok(mut f) => {
                            match f.write_all(content.as_bytes()) {
                                Ok(_) => (),
                                Err(e) => {
                                    log::error!("{}", e);
                                    return;
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("{}", e);
                            return;
                        }
                    }
                    log::info!("Result save to result.json");

                    for competitor in competitors {
                        if let Some(grades) = results.get(&competitor.id) {
                            if let Some(dir) = competitor.result {
                                log::info!("Adding result to {}", competitor.id);
                                let mut report: String = "Beep boop! Your results are here:\n\n".into();
                                for (p_name, evaluation) in grades.iter() {
                                    if let Some(problem) = arena.problems.get(p_name) {
                                        match evaluation {
                                            Evaluation::Grade{score} => {
                                                report += &format!("-> problem \"{}\": {}\n", p_name, score*10.0);
                                            },
                                            Evaluation::RunError => {
                                                report += &format!("-> problem \"{}\": execution/compilation error\n", p_name);
                                            },
                                            Evaluation::NoFile => {
                                                report += &format!("-> problem \"{}\": file not found \"{}\"\n", p_name, problem.filename);
                                            }
                                        }
                                    } else {
                                        log::error!("Problem data not found");
                                        return;
                                    }
                                }
                                let mut result_file = PathBuf::from(dir.clone());
                                result_file.push("resultados.txt");
                                match File::create(result_file) {
                                    Ok(mut f) => {
                                        match f.write_all(report.as_bytes()) {
                                            Ok(_) => (),
                                            Err(e) => log::warn!("Could not write report for user \"{}\", {}", competitor.id, e)
                                        };
                                    },
                                    Err(e) => log::warn!("Could not create report for user \"{}\", {}", competitor.id, e) 
                                }
                            } else {
                                log::info!("competitor {} will not receive grades", competitor.id);
                            }
                        } else {
                            log::error!("somehow, grades for competitor {} are not present", competitor.id);
                        }
                    }
                },
                Err(e) => {
                    log::error!("{}", e);
                    return;
                }
            }
        }
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