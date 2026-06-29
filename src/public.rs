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


use serde_json::Value;

use std::io::Result;

use crate::data::*;

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

pub fn write_error(mensaje: &str) {
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

pub fn tovec(valor: &Value) -> Vec<String> {
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

//Imprime una lista 
pub fn list(list: &[String]) -> usize {
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




 
