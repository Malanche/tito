extern crate log;
extern crate chrono;

use log::{Log, Record, Level, Metadata};
use std::fs::{File, OpenOptions, read_dir};
use std::path::PathBuf;
use std::io::{prelude::*, BufReader};

// Para imprimir el tiempo adecuadamente
use chrono::prelude::*;

pub struct TitoLogger{}

impl Log for TitoLogger {
    /// Indica qué nivel de bitácora se usará
    fn enabled(&self, metadata: &Metadata) -> bool {
        match metadata.level() {
            Level::Error => true,
            Level::Warn => true,
            Level::Info => true,
            Level::Debug => true,
            Level::Trace => true
        }
    }

    /// Lidia con los registros de manera individual
    fn log(&self, record: &Record) {
        // Sólo hacemos caso a los mensajes que planeamos guardar en la bitácora
        if self.enabled(record.metadata()) {
            // Ahora sí, escribimos el mensaje.
            let dt = Local::now();//Utc.ymd(2014, 11, 28).and_hms(12, 0, 9);
            let registry = match record.level() {
                Level::Error => {
                    format!("{} tito[\u{001b}[0;31m{}\u{001b}[0m]: {}", dt.format("%b %e %T"), record.level(), record.args())
                },
                Level::Warn => {
                    format!("{} tito[\u{001b}[0;33m{}\u{001b}[0m]: {}", dt.format("%b %e %T"), record.level(), record.args())
                }, 
                Level::Info => {
                    format!("{} tito[\u{001b}[0;34m{}\u{001b}[0m]: {}", dt.format("%b %e %T"), record.level(), record.args())
                }
                Level::Debug => {
                    format!("{} tito[\u{001b}[0;36m{}\u{001b}[0m]: {}", dt.format("%b %e %T"), record.level(), record.args())
                }
                _ => format!("{} tito[{}]: {}", dt.format("%b %e %T"), record.level(), record.args())
            };
            println!("{}",&registry);
        }
    }

    /// Función vacía
    fn flush(&self) {}
}

impl TitoLogger {
    /// Genera una nueva instancia del gestor de bitácoras.
    pub fn new() -> TitoLogger {
        TitoLogger{}
    }
}