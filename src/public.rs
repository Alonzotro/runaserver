use mysql::serde::de::value::Error;
use terminal_size::{Width, terminal_size};
use rust_i18n::t;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::process::{Command, Stdio, ExitStatus};
use std::path::{Path, PathBuf};
use std::process::Output;
use std::{result, string, vec};
use chrono::Local;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use rust_embed::RustEmbed;
use serde_json::Value;


pub const OK: &str = "[OK]";
pub const INFO: &str = "[•]";
pub const WARNING: &str = "[!]";
pub const ERROR_YOU: &str = "[X]";
pub const ERROR_PC: &str = "[ERROR]";
pub const ARROW: &str = "-->";
pub const LOG_ERRORES: &str = "/var/log/errores_mantenimiento.log";
const JSON_EMBEDDED: &str = include_str!("../assets/config.json");

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

// Asumo que `error_log()`, `OK`, `WARNING`, `ERROR_PC` y las macros de
// rust_i18n ya están definidas en otro módulo de tu proyecto, igual que
// en tu versión original.
pub fn evaluate_fs<T>(result: io::Result<T>, action: &str, show_logs: bool) -> bool {
    match result {
        Ok(_) => {
            if show_logs {
                println!("{} {}: {}", OK, rust_i18n::t!("FS_SUCCESS"), action);
            }
            true
        }
        Err(e) => {
            // ✅ CORRECCIÓN: Enviamos el error directamente a stderr del sistema.
            // Al ser un error nativo de Rust, no usamos Stdio de procesos.
            eprintln!("[FS ERR] {}: {}", action, e);

            if show_logs {
                println!("{} {}: {} -> {}", ERROR_PC, rust_i18n::t!("FS_ERROR"), action, e);
            }
            false
        }
    }
}

pub fn evaluate(cmd: Result<ExitStatus> ) -> bool {
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
pub fn execute<T: AsRef<str>>(cmd: &str, args: &[T]) -> bool {
    let status = Command::new(cmd)
        .args(args) // Fix: arg -> args
        .stderr(error_log())
        .stdout(Stdio::null())
        .status();
    evaluate(status)
}

//Ejecuta un comando con Output de manera silenciosa
pub fn output<T: AsRef<str>>(cmd: &str, args: &[T]) -> (String, bool, Result<(), io::Error>) {
    let status = Command::new(cmd).args(args).output();
    match status {
        Ok(out) if out.status.success() => {
            let stdout_text = String::from_utf8_lossy(&out.stdout).to_string();
            (stdout_text, true, Ok(()))
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
            (stdout_text, false, Ok(()))
        }
        Err(e) => {
            let log_msg = format!("[CRITICAL ERR] Fallo al lanzar el binario '{}': {}", cmd, e);
            write_error(&log_msg);
            (String::new(), false, Err(e))
        }
    }
}

fn write_error(mensaje: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_ERRORES)
    {
        // FIX 2: timestamp añadido
        let ts = Local::now().format("[%Y-%m-%d %H:%M:%S]");
        let _ = writeln!(file, "{} {}\n---", ts, mensaje);
    }
}

pub fn findout_software(programas: &[String]) -> (Vec<String>, Vec<String>) {
    if programas.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let instalados: Vec<String> = Command::new("dpkg-query")
        .args(["-W", "-f=${Package}\n"])
        .args(programas)
        .stderr(Stdio::null())
        .output()
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
 
    let instalados_set: HashSet<&str> = instalados.iter().map(String::as_str).collect();
 
    let faltantes: Vec<String> = programas
        .iter()
        .filter(|p| !instalados_set.contains(p.as_str()))
        .cloned()
        .collect();
 
    (faltantes, instalados)
}

pub fn install_packages(packages_raw: &[String]) -> Vec<String> {
    let (mut to_install, _) = findout_software(packages_raw);

    // Si no falta nada, devolvemos un vector vacío al instante
    if to_install.is_empty() {
        return Vec::new();
    }

    // 1. Creamos la base directamente como String
    let mut args = vec!["install".to_string(), "-y".to_string()];

    // 2. FUSIONAMOS: Mueve los elementos de 'to_install' dentro de 'args'
    args.append(&mut to_install);

    args // Retorno implícito (sin punto y coma)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub admin_mode: bool,
    //pub server_name: String,
}

impl Settings {
    const FILE_PATH: &'static str = "settings.toml";

    // 2. Función para cargar (si no existe, crea uno por defecto)
    pub fn load() -> Self {
        if !Path::new(Self::FILE_PATH).exists() {
            let default_settings = Settings {
                admin_mode: false,
                //server_name: String::from("Mi Servidor Rust"),
            };
            default_settings.save();
            return default_settings;
        }

        let contenido = fs::read_to_string(Self::FILE_PATH)
            .expect("No se pudo leer el archivo de configuración");
        
        toml::from_str(&contenido)
            .expect("Formato TOML inválido")
    }

    // 3. Función para guardar los cambios en el disco
    pub fn save(&self) {
        let toml_string = toml::to_string_pretty(self)
            .expect("No se pudo serializar a TOML");
        fs::write(Self::FILE_PATH, toml_string)
            .expect("No se pudo escribir el archivo en disco");
    }

    // 4. Función específica para modificar el Admin Mode
    pub fn set_admin_mode(&mut self, mode: bool) {
        self.admin_mode = mode;
        self.save(); // Guarda automáticamente en el archivo cada vez que cambia
    }
}


#[derive(RustEmbed)]
#[folder = "assets/"]
#[include = "*.json"]
struct Assets;


pub fn search_json(archivo: &str, clave: &str) -> Vec<String> {

    if clave.is_empty() || archivo.is_empty() {
        return Vec::new(); 
    }

    let contenido = Assets::get(archivo)
        .and_then(|f| String::from_utf8(f.data.to_vec()).ok())
        .unwrap_or_else(|| panic!("assets/{archivo} no encontrado o no es UTF-8 válido"));
 
    let json: Value = serde_json::from_str(&contenido)
        .unwrap_or_else(|e| panic!("JSON inválido en assets/{archivo}: {e}"));
 
    tovec(&json[clave])
}
 
fn tovec(valor: &Value) -> Vec<String> {
    match valor {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                Value::Bool(b)   => Some(b.to_string()),
                _                => None,
            })
            .collect(),
        Value::String(s) => vec![s.clone()],
        Value::Number(n) => vec![n.to_string()],
        Value::Bool(b)   => vec![b.to_string()],
        _                => vec![],   // Null o clave inexistente
    }
}

