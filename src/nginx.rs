use crate::public::{ARROW, ERROR_PC, ERROR_YOU, INFO, LOG_ERRORES, OK, WARNING, clear_screen, error_log, evaluate, line, print_header, read_in, execute, output};
use crate::servicios::permisos;
use crate::data::{get_installed_php};

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, BufRead, BufReader, ErrorKind};
use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use regex::Regex;
use ureq::http::status;


pub fn restart_nginx() {
    // 1. Describimos la acción que se va a ejecutar
    println!("{}", rust_i18n::t!("RESTARTING_NGINX"));

    // 2. Evaluamos el comando directamente con la macro
    if !execute("systemctl", &["restart", "nginx"]) {
        println!("[X] {}", rust_i18n::t!("APACHE_RESTART_ERROR_TIP"));
    }
}

pub fn inspect_nginx() {
    // 1. Describimos la acción que se va a ejecutar
    println!("{}", rust_i18n::t!("INSPECTING_APACHE"));

    // 2. Evaluamos el comando directamente con la macro
    if !execute("nginx", &["-t"]) {
        println!("[X] {}", rust_i18n::t!("APACHE_RESTART_ERROR_TIP"));
    }
}