extern crate log;
extern crate tempdir;
extern crate wait_timeout;

use crate::{
    Settings, Arena, Competitor, Proposal, Problem, Language, LanguageSettings, Scenario, Tool, Evaluation
};
use wait_timeout::ChildExt;
use log::{info, warn, error};
use tempdir::TempDir;
use std::fs::File;
use std::io::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{PathBuf, Path};
use std::process::{Command, Stdio};

pub struct Tito {
    arena: Option<Arena>,
    language_settings: HashMap<Language, LanguageSettings>,
    dir: TempDir
} 

impl Tito {
    pub fn new() -> Result<Tito, Error> {
        let dir = TempDir::new("tito").map_err(|e| Error::IOError(e))?;
        Ok(Tito {
            arena: None,
            language_settings: HashMap::new(),
            dir
        })
    }

    pub fn build(&mut self, settings: Settings) -> Result<Arena, Error> {
        let mut languages = HashSet::new();

        for (_name, proposal) in settings.proposals.iter() {
            languages.insert(proposal.language.clone());
        }

        info!("Gathering languages information...");
        self.gather_language_info(languages)?;

        // Last but obviously not least, we test proposal codes
        let mut problems = HashMap::new();
        for (name, proposal) in settings.proposals.iter() {
            info!("Evaluating \"{}\"", name);
            // We extract the filename
            let filename = match PathBuf::from(&proposal.solution).as_path().file_stem() {
                Some(os_str) => os_str.to_string_lossy().to_string(),
                None => return Err(Error::NoFileName(name.clone()))
            };
            let solutions = self.test_proposal(proposal)?;
            let scenarios = proposal.scenarios.iter().zip(solutions.iter()).map(|(sc, so)| {
                let mut sc = sc.clone();
                sc.output = Some(so.trim().into());
                sc
            }).collect();
            problems.insert(name.clone(), Problem{
                scenarios,
                filename,
                language: Some(proposal.language.clone()),
                points: proposal.points
            });
        }
        Ok(Arena{problems})
    }

    pub fn run(&mut self, competitors: Vec<Competitor>, arena: Arena) -> Result<HashMap<String, HashMap<String, Evaluation>>, Error> {
        let mut grades = HashMap::new();
        let mut languages = HashSet::new();

        for (_name, problem) in arena.problems.iter() {
            languages.insert(problem.language.clone().unwrap());
        }

        info!("Gathering languages information...");
        self.gather_language_info(languages)?;

        for competitor in competitors {
            // User grades for ever
            let mut user_grades = HashMap::new();

            for (name, problem) in arena.problems.iter() {
                info!("Evaluating problem \"{}\" for competitor \"{}\"", name, competitor.id);
                match self.evaluate(PathBuf::from(&competitor.files), problem) {
                    Ok(mut outputs) => {
                        let mut score = 0.0;
                        // We will trim the answers
                        for output in &mut outputs {
                            *output = String::from(output.trim());
                        }

                        // Now, we compare them to give this guy a grade
                        for (idx, (candidate, solution)) in outputs.iter().zip(problem.scenarios.iter()).enumerate() {
                            if let Some(output) = &solution.output {
                                if candidate == output {
                                    score += solution.points as f64;
                                }
                            } else {
                                return Err(Error::NoSolution(name.clone(), idx));
                            }
                        }

                        score /= problem.points as f64;

                        //
                        user_grades.insert(name.clone(), Evaluation::Grade{score});
                    },
                    Err(e) => match e {
                        Error::NoFileFound => {
                            info!("File not found!");
                            user_grades.insert(name.clone(), Evaluation::NoFile);
                        },
                        other => {
                            warn!("{}", other);
                            user_grades.insert(name.clone(), Evaluation::RunError);
                        }
                    }
                };
            }

            grades.insert(competitor.id.clone(), user_grades);
        }
        Ok(grades)
    }

    fn gather_language_info(&mut self, languages: HashSet<Language>) -> Result<(), Error> {
        for language in languages.iter() {
            info!("Checking default tools for language {}", serde_json::to_string(&language).unwrap());
            let language_settings = LanguageSettings::default(language.clone());

            if let Some(pre_tools) = &language_settings.pre_tools {
                for pre_tool in pre_tools {
                    if !pre_tool.temporal {
                        match Tito::tool_exists(&pre_tool.utility) {
                            Ok(found) => if found {
                                info!("Found pre-tool \"{}\"", &pre_tool.utility)
                            } else {
                                return Err(Error::MissingTool(format!("{}", &pre_tool.utility)));
                            },
                            Err(e) => return Err(Error::ToolLookup(format!("{} ({})", e, &pre_tool.utility)))
                        }
                    }
                }
            }
            if !language_settings.tool.temporal {
                match Tito::tool_exists(&language_settings.tool.utility) {
                    Ok(found) => if found {
                        info!("Found tool \"{}\"", &language_settings.tool.utility)
                    } else {
                        return Err(Error::MissingTool(format!("{}", &language_settings.tool.utility)));
                    },
                    Err(e) => return Err(Error::ToolLookup(format!("{} ({})", e, &language_settings.tool.utility)))
                }
            }
            self.language_settings.insert(language.clone(), language_settings);
        }
        Ok(())
    }

    fn evaluate(&self, directory: PathBuf, problem: &Problem) -> Result<Vec<String>, Error> {
        // We lookup for the source code pointed in the proposal
        let (source, language_settings) = if let Some(language) = &problem.language {
            let language_settings = match self.language_settings.get(&language) {
                Some(v) => v.clone(),
                None => return Err(Error::NoLangSettings(serde_json::to_string(&language).unwrap()))
            };

            let mut filename = directory.clone();
            filename.push(problem.filename.clone() + &format!(".{}", language_settings.extension));
            let source = match File::open(&filename) {
                Ok(mut f) => {
                    let mut source = String::new();
                    match f.read_to_string(&mut source) {
                        Ok(_) => (),
                        Err(e) => return Err(Error::IOError(e))
                    }
                    source
                },
                Err(_e) => return Err(Error::NoFileFound)
            };
            (source, language_settings)
        } else {
            // We guess with the file extension
            panic!("Unsupported!");
        };

        // We load the languae settings
        let outputs = self.run_tools(source, problem.scenarios.clone(), &language_settings)?;
        let mut result = Vec::new();
        for output in outputs.into_iter() {
            let output = output?;
            result.push(output);
        }
        Ok(result)
    }

    fn tool_exists<T: AsRef<str>>(tool_name: T) -> Result<bool, String> {
        if cfg!(target_os = "windows") {
            let output = match Command::new("where")
                .arg(tool_name.as_ref())
                .stdin(Stdio::null()) 
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .output() {
                    Ok(v) => v,
                    Err(e) => return Err(format!("{}", e))
            };

            Ok(output.status.success())
        } else {
            Err("Still unimplemented".into())
        }
    }

    fn test_proposal(&self, proposal: &Proposal) -> Result<Vec<String>, Error> {
        // We lookup for the source code pointed in the proposal
        let source = match File::open(&proposal.solution) {
            Ok(mut f) => {
                let mut source = String::new();
                match f.read_to_string(&mut source) {
                    Ok(_) => (),
                    Err(e) => return Err(Error::IOError(e))
                }
                source
            },
            Err(e) => return Err(Error::IOError(e))
        };
        // We load the languae settings
        if let Some(language_setting) = self.language_settings.get(&proposal.language) {
            let outputs = self.run_tools(source, proposal.scenarios.clone(), &language_setting)?;
            let mut result = Vec::new();
            for output in outputs.into_iter() {
                let output = output?;
                result.push(output);
            }
            Ok(result)
        } else {
            Err(Error::NoLangSettings(serde_json::to_string(&proposal.language).unwrap()))
        }
    }

    fn run_tools(&self, source: String, scenarios: Vec<Scenario>, language_settings: &LanguageSettings) -> Result<Vec<Result<String, Error>>, Error> {
        // We will put all the tools in a single vector, and take note of the index of the main tool
        let mut tools = Vec::new();
        if let Some(pre_tools) = &language_settings.pre_tools {
            tools = pre_tools.iter().map(|t| (t.clone(), false)).collect();
        }
        // Now we add the main one, the one that gives the results
        tools.push((language_settings.tool.clone(), true));

        // We will write the source code, BAE
        let path = PathBuf::from(self.dir.path());
        let mut filename = path.clone();
        filename.push(String::from("source.") + &language_settings.extension);

        // Now we create the file
        match File::create(&filename) {
            Ok(mut f) => {
                f.write_all(&source.as_bytes()).map_err(|e| Error::IOError(e))?;
            },
            Err(e) => return Err(Error::IOError(e))
        };

        let mut values = Vec::new();

        // Now, tool execution
        for (idx, (tool, queen)) in tools.into_iter().enumerate() {
            let args: Vec<String> = tool.arguments.iter().map(|arg| {
                let arg = arg.replace("{filename}", &filename.to_string_lossy());
                arg.replace("{pwd}", &path.to_string_lossy())
            }).collect();

            let utility = tool.utility.replace("{pwd}", &path.to_string_lossy());

            if queen {
                // We go through each scenario
                for scenario in &scenarios {
                    let mut child = match Command::new(&utility)
                        .current_dir(path.clone())
                        .args(&args)
                        .stdin(Stdio::piped()) // Para poder pasar argumentos al programa
                        .stderr(Stdio::piped()) // Para poder capturar la salida de error
                        .stdout(Stdio::piped()).spawn() {
                            Ok(v) => v,
                            Err(e) => return Err(Error::ChildProcessError(format!("{}", e)))
                    };

                    // We declare the child_stdin inside a block of code in case an input is mandatory
                    if let Some(input) = &scenario.input {
                        // Mandamos el caso como cadena al hijo
                        let child_stdin = match child.stdin.as_mut() {
                            Some(v) => v,
                            None => {
                                child.kill();
                                return Err(Error::ChildStdinRef)
                            }
                        };
                        match child_stdin.write_all(input.as_bytes()) {
                            Ok(_) => (),
                            Err(e) => {
                                child.kill();
                                return Err(Error::ChildStdinFeed);
                            }
                        }
                    }
            
                    match child.wait_timeout(std::time::Duration::from_millis((scenario.max_time * 1000.0) as u64)) {
                        Ok(v) => match v{
                            Some(status) => status.code(),
                            None => {
                                // child hasn't exited yet, so we kill it
                                match child.kill() {
                                    Ok(_) => (),
                                    Err(e) => {
                                        warn!("Could not kill process: {}", e);
                                    }
                                };
                                values.push(Err(Error::TimeExceeded));
                                continue;
                            }
                        },
                        Err(e) => return Err(Error::WaitTimeoutError(format!("{}", e)))
                    };
            
                    let output = match child.wait_with_output() {
                        Ok(v) => v,
                        Err(e) => return Err(Error::WaitOutputError(format!("{}", e)))
                    };
            
                    if output.status.success() {
                        values.push(String::from_utf8(output.stdout).map_err(|_e| Error::Utf8));
                    } else {
                        return Err(Error::RuntimeError(String::from_utf8_lossy(&output.stderr).to_string()));
                    }
                }
            } else {
                // We just execute carelessly
                let output = match Command::new(utility)
                    .current_dir(path.clone())
                    .args(&args)
                    .stdin(Stdio::null()) // Para poder pasar argumentos al programa
                    .stderr(Stdio::piped()) // Para poder capturar la salida de error
                    .stdout(Stdio::null()).output() {
                        Ok(v) => v,
                        Err(e) => return Err(Error::ChildProcessError(format!("{}", e)))
                };
                if !output.status.success() {
                    return Err(Error::ToolFailure(idx, String::from_utf8_lossy(&output.stderr).to_string()));
                }
            };
        }
        Ok(values)
    }
}

#[derive(Debug)]
pub enum Error {
    MissingTool(String),
    ToolLookup(String),
    NoFileName(String),
    NoLangSettings(String),
    ChildProcessError(String),
    ToolFailure(usize, String),
    ChildStdinRef,
    ChildStdinFeed,
    WaitOutputError(String),
    WaitTimeoutError(String),
    RuntimeError(String),
    TimeExceeded,
    NoFileFound,
    Utf8,
    NoSolution(String, usize),
    SettingsError,
    IOError(std::io::Error)
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let content = match self {
            Error::MissingTool(tool) => format!("The tool \"{}\" was not found", tool),
            Error::ToolLookup(detail) => format!("Could not perform tool search, {}", detail),
            Error::NoFileName(problem) => format!("Problem \"{}\" did not contain a valid filename", problem),
            Error::NoLangSettings(detail) => format!("No language settings were found for language {}", detail),
            Error::ChildProcessError(detail) => format!("Could not spawn child process, {}", detail),
            Error::ToolFailure(idx, detail) => format!("Tool {} failed with the following stderr: {}", idx, detail),
            Error::ChildStdinRef => format!("Could not obtain reference to child stdin"),
            Error::ChildStdinFeed => format!("Could not feed input to child process"),
            Error::WaitTimeoutError(detail) => format!("Wait timeout command failed, {}", detail),
            Error::WaitOutputError(detail) => format!("Wait for output failed, {}", detail),
            Error::RuntimeError(detail) => format!("Runtime error, {}", detail),
            Error::TimeExceeded => format!("The execution exceeded the maximum time"),
            Error::NoFileFound => format!("Could not evaluete due to lack of file"),
            Error::Utf8 => format!("A byte stream received was not utf-8 valid"),
            Error::NoSolution(name, idx) => format!("No solution was provided for problem {}, scenario {}", name, idx),
            Error::SettingsError => format!("An error with the settings has occured"),
            Error::IOError(e) => format!("An io error occured, {}", e)
        };
        write!(formatter, "{}", content)
    }
}

impl std::error::Error for Error {}