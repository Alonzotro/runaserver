use crate::public::*;
use crate::apache::{restart_apache};
use crate::servicios::{update};
use std::ffi::OsStr;
use std::process::{Command, Stdio, ExitStatus};
use std::io::{self, Error};



pub fn confirm<T>(result: io::Result<T>) -> bool {
    match result {
        Ok(_) => {
                println!("{} {}", OK, rust_i18n::t!("FS_SUCCESS"));
            true
        }
        Err(e) => {
                eprintln!("[FS ERR] {}", e);
                println!("{} {} -> {}", ERROR_PC, rust_i18n::t!("FS_ERROR"), e);
            false
        }
    }
}

pub fn evaluate(cmd: Result<ExitStatus, Error> ) -> bool {
    match cmd {
        Ok(status) if status.success() => {
            println!("{} {}", OK, rust_i18n::t!("RESULT_OK"));
            true
        }
        Ok(status) => {
                let code = status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "?".to_string());

                println!("{} {}", WARNING, rust_i18n::t!("RESULT"));
                println!("{} {}", rust_i18n::t!("CODE"), code);
            false
        }
        Err(e) => {
                println!("{} {}", ERROR_PC, rust_i18n::t!("RESULT_ERROR"));
                println!("{} {}", rust_i18n::t!("ERROR_CAUSE"), e);
            false
        }
    }
}

//Ejecuta un comando con Status de manera silenciosa
pub fn execute<T: AsRef<OsStr>>(cmd: &str, args: &[T]) -> bool {
    let status = Command::new(cmd)
        .args(args)           // ← directo, sin Vec intermedio
        .stderr(error_log())
        .stdout(Stdio::null())
        .status();
    evaluate(status)
}

//Ejecuta un comando con Output de manera silenciosa
pub fn output(cmd: &str, args: &[&str]) -> (String, bool,) {
    let status = Command::new(cmd).args(args).output();
    match status {
        Ok(out) if out.status.success() => {
            let stdout_text = String::from_utf8_lossy(&out.stdout).to_string();
            (stdout_text, true)
        }
        Ok(out) => {
            let stdout_text = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr_text = String::from_utf8_lossy(&out.stderr);
            
            // Armamos el mensaje aquí arriba teniendo acceso limpio a 'out'
            let log_msg = format!(
                "[ERROR] Comando fallido con código ({}).\nDetalle: {}", 
                out.status, 
                stderr_text.trim()
            );
            write_error(&log_msg);
            (stdout_text, false)
        }
        Err(e) => {
            let log_msg = format!("[CRITICAL ERR] Fallo al lanzar el binario '{}': {}", cmd, e);
            write_error(&log_msg);
            (String::new(), false)
        }
    }
}

pub fn valid_input(input: &str, lista_len: usize) -> bool {
    // Intentamos parsear y asignamos directamente el resultado a 'idx'
    let idx: usize = match input.trim().parse() {
        Ok(num) => num, // <- AQUÍ está la clave: devolvemos 'num' para asignarlo a 'idx'
        Err(_) => {
            println!("[X] Opción inválida.");
            return false;
        }
    };

    // Verificamos el rango
    idx >= 1 && idx <= lista_len
}

pub fn valid_name(nombre: &str) -> bool {
    // 1. Longitud
    if nombre.is_empty() || nombre.len() > 64 {
        return false;
    }

    // 2. No debe empezar con guion ni terminar con guion (mejor práctica)
    if nombre.starts_with('-') || nombre.starts_with('_') || nombre.ends_with('-') || nombre.ends_with('_') {
        return false;
    }

    // 3. Caracteres permitidos (quitamos el punto y coma final para que sea el resultado de la función)
    nombre.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

