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
use std::ffi::OsStr;
use std::io::Result;


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
#[include = "*.conf"]
//const CONF2: &str = include_str!("../assets/config2.json");
struct Assets;


pub fn search_json(archivo: &str, clave: &str) -> Vec<String> {
    // 1. Uso de OR lógico ||
    if clave.is_empty() || archivo.is_empty() {
        return Vec::new(); 
    }

    // 2. Manejo elegante sin panics
    let contenido = match Assets::get(archivo) {
        Some(f) => String::from_utf8(f.data.to_vec()).unwrap_or_default(),
        None => {
            eprintln!("[ERROR] Asset '{}' no encontrado.", archivo);
            return Vec::new();
        }
    };
 
    // 3. Manejo de JSON corrupto sin panics
    let json: Value = match serde_json::from_str(&contenido) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[ERROR] JSON corrupto en '{}': {}", archivo, e);
            return Vec::new();
        }
    };
 
    // 4. Uso de .get() seguro: si no existe la clave, devuelve un vector vacío
    match json.get(clave) {
        Some(val) => tovec(val),
        None => Vec::new(),
    }
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

pub fn generate_vhost(sitio: &str, web_dir: &str, ver: &str) -> std::io::Result<()> {
    // 1. Obtener la plantilla desde el binario
    let plantilla_bytes = Assets::get("vhost.conf")
        .expect("La plantilla vhost.conf no existe en los assets");
    
    let mut contenido = std::str::from_utf8(plantilla_bytes.data.as_ref())
        .unwrap()
        .to_string();

    // 2. Reemplazar los marcadores
    contenido = contenido.replace("{sitio}", sitio);
    contenido = contenido.replace("{web_dir}", web_dir);
    contenido = contenido.replace("{ver}", ver);

    // 3. Definir la ruta destino usando el nombre del sitio
    // Por ejemplo: /etc/apache2/sites-available/misitio.conf
    let ruta_destino = format!("/etc/apache2/sites-available/{}.conf", sitio);

    // 4. Escribir el archivo
    std::fs::write(&ruta_destino, contenido)?;
    
    println!("[✓] Archivo de configuración generado: {}", ruta_destino);
    Ok(())
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

//Imprime una lista 
pub fn list_version(list: &[String]) -> usize {
    if list.is_empty() {
        println!("{WARNING} No hay nada que enlistar o no tiene nada instalado.");
        line();
        return 0; // Devolvemos 0 si está vacía
    }

    for (i, ver) in list.iter().enumerate() {
        // Nota: cambié i por i + 1 para que el menú sea más natural para el usuario
        println!("{}) PHP{}", i + 1, ver);
    }
    line();

    list.len() // Esta es la cantidad de elementos, se devuelve implícitamente
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
