use terminal_size::{Width, terminal_size};
use rust_i18n::t;
use std::fs::{OpenOptions};
use std::io::{self, Write};
use std::process::{Command, Stdio, ExitStatus};
use std::path::PathBuf;
use std::process::Output;
use chrono::Local;

use crate::evaluate;

pub const OK: &str = "[OK]";
pub const INFO: &str = "[•]";
pub const WARNING: &str = "[!]";
pub const ERROR_YOU: &str = "[X]";
pub const ERROR_PC: &str = "[ERROR]";
pub const ARROW: &str = "-->";
pub const LOG_ERRORES: &str = "/var/log/errores_mantenimiento.log";


pub fn error_log() -> Stdio {
    //Abre el achivo de log, anade la informacion hasta abajo, lo crea si es necesario y verifica que todo esta bien
    if let Ok(file) = OpenOptions::new().create(true).append(true).open(LOG_ERRORES) {
        // 1. Creamos la estampa de tiempo
        //let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S] - ").to_string();
        // 2. La escribimos en el archivo ANTES de pasárselo a systemctl
        //let _ = file.write_all(timestamp.as_bytes());
        //Crea una tuveria en log files
        Stdio::from(file)
    } else {
        Stdio::null()
    }
}

pub fn clear_screen() {
    // \x1B[2J limpia la pantalla visible
    // \x1B[3J borra el historial de desplazamiento (scrollback buffer)
    // \x1B[1;1H reposiciona el cursor en la esquina superior izquierda
    print!("{}[2J{}[3J{}[1;1H", 27 as char, 27 as char, 27 as char);
    let _ = io::stdout().flush();
}

pub fn print_header(titulo: &str) {
    // 1. Detectamos el ancho completo de la terminal. Fallback de 80 si no hay TTY.
    let ancho_terminal = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    };

    let titulo_upper = titulo.to_uppercase();
    let largo_titulo = titulo_upper.len();

    // 2. Calculamos el espacio a la izquierda para centrar el texto en toda la pantalla
    let espacios_izquierda = if ancho_terminal > largo_titulo {
        (ancho_terminal - largo_titulo) / 2
    } else {
        0
    };

    let padding = " ".repeat(espacios_izquierda);

    // 3. Imprimimos el bloque de puros guiones limpios
    println!("{}", "=".repeat(ancho_terminal));
    println!("{}{}", padding, titulo_upper);
    println!("{}", "=".repeat(ancho_terminal));
}

pub fn line() {
    let ancho_terminal = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    };
    println!("{}", "=".repeat(ancho_terminal));
}

pub fn read_in(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input
}

#[macro_export]
macro_rules! read_in {
    // Caso 1: Cuando le pasas un texto con variables tipo format!
    ($fmt:expr, $($arg:tt)*) => {{
        print!("{}", format!($fmt, $($arg)*));
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input).expect("Error al leer línea");
        input.trim().to_string()
    }};

    // Caso 2: Cuando solo le pasas un texto simple o una traducción directa
    ($prompt:expr) => {{
        print!("{}", $prompt);
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input).expect("Error al leer línea");
        input.trim().to_string()
    }};
}

pub fn command(command: &str, args: &[&str], output: bool) -> bool {
    evaluate!(Command::new(command)
    .args(args)
    .stdout(Stdio::null())
    .stderr(error_log()).status()
    , output)
}

// 1. Creamos la "interfaz" para que cualquier cosa pueda ser evaluada
pub trait Evaluable {
    type Output;
    fn evaluate(self, show: bool) -> Self::Output;
}

// 2. Implementación para Comandos de Terminal (ExitStatus)
impl Evaluable for io::Result<ExitStatus> {
    type Output = bool;
    fn evaluate(self, show: bool) -> Self::Output {
        match self {
            Ok(status) if status.success() => {
                if show { println!("{} {}", OK, rust_i18n::t!("RESULT_OK")); }
                true
            }
            Ok(status) => {
                if show { 
                    println!("{} {}", WARNING, rust_i18n::t!("RESULT")); 
                    println!("{} {}", rust_i18n::t!("CODE"), status);
                }
                false
            }
            Err(e) => {
                if show { 
                    println!("{} {}", ERROR_PC, rust_i18n::t!("RESULT_ERROR"));
                    println!("{} {}", rust_i18n::t!("CODE"), e);
            }
                false
            }
        }
    }
}

// 3. Implementación para Operaciones de Archivos (fs con tipo vacío '()')
impl Evaluable for io::Result<()> {
    type Output = bool;
    fn evaluate(self, show: bool) -> Self::Output {
        match self {
            Ok(_) => {
                if show { println!("{} {}", OK, rust_i18n::t!("RESULT_OK")); }
                true
            }
            Err(e) => {
                if show { 
                    println!("{} {}", ERROR_PC, rust_i18n::t!("RESULT_ERROR"));
                    println!("{} {}", rust_i18n::t!("CODE"), e);
                }
                false
            }
        }
    }
}

// 5. Implementación para operaciones que devuelven un conteo numérico (como fs::copy)
impl Evaluable for io::Result<u64> {
    type Output = bool;
    fn evaluate(self, show: bool) -> Self::Output {
        match self {
            Ok(_) => { // Ignoramos el número de bytes, solo nos importa que fue Ok
                if show { println!("{} {}", OK, rust_i18n::t!("RESULT_OK")); }
                true
            }
            Err(e) => {
                if show { 
                    println!("{} {}", ERROR_PC, rust_i18n::t!("RESULT_ERROR"));
                    println!("{} {}", rust_i18n::t!("CODE"), e);
                }
                false
            }
        }
    }
}

impl Evaluable for io::Result<PathBuf> {
    type Output = Option<PathBuf>; // <-- Si sale bien te da la ruta, si falla te da None
    fn evaluate(self, show: bool) -> Self::Output {
        match self {
            Ok(path) => {
                if show { println!("{} {}", OK, rust_i18n::t!("RESULT_OK")); }
                Some(path) // <-- Aquí te entrego tu ruta vivita y coleando
            }
            Err(e) => {
                if show { 
                    println!("{} {}", ERROR_PC, rust_i18n::t!("RESULT_ERROR"));
                    println!("{} {}", rust_i18n::t!("CODE"), e);
                }
                None // <-- Te regreso un None para avisar que falló
            }
        }
    }
}
// Implementación para comandos de los que necesitamos extraer texto (.output())
impl Evaluable for io::Result<Output> {
    type Output = Option<Output>; // Nos devuelve el objeto Output dentro de un Option
    fn evaluate(self, show: bool) -> Self::Output {
        match self {
            Ok(out) if out.status.success() => Some(out),
            Ok(out) => {
                if show { 
                    println!("{} {}", WARNING, rust_i18n::t!("RESULT")); 
                    println!("{} {}", rust_i18n::t!("CODE"), out.status);
                }
                None
            }
            Err(e) => {
                if show { 
                    println!("{} {} ", ERROR_PC, rust_i18n::t!("RESULT_ERROR"));
                    println!("{} {}", rust_i18n::t!("CODE"), e);
                }
                None
            }
        }
    }
}

#[macro_export]
macro_rules! evaluate {
    // Si no le pasas segundo argumento, por defecto asume true (mostrar)
    ($resultado:expr) => {
        $resultado.evaluate(true)
    };
    // Si le pasas un booleano, usa ese booleano
    ($resultado:expr, $show:expr) => {
        $resultado.evaluate($show)
    };
}



