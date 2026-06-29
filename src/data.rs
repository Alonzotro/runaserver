use crate::public::*;
use crate::checker::*;

use std::fs::{self};
use std::path::Path;
use std::process::{Command, Stdio};
use std::collections::BTreeSet;
use std::collections::HashSet;

use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use serde_json::Value;


//Comunes
pub fn sort_versions(pkg_list: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
    let mut versiones: BTreeSet<(u32, u32)> = BTreeSet::new();

    for pkg in pkg_list {
        let s = pkg.as_ref().trim();
        let Some(resto) = s.strip_prefix("php") else { continue };
        
        let version_str: String = resto.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        let Some((maj_str, min_str)) = version_str.split_once('.') else { continue };

        let min_clean: String = min_str.chars().take_while(|c| c.is_ascii_digit()).collect();

        if let (Ok(maj), Ok(min)) = (maj_str.parse::<u32>(), min_clean.parse::<u32>()) {
            versiones.insert((maj, min));
        }
    }

    versiones.into_iter().map(|(maj, min)| format!("{}.{}", maj, min)).collect()
}

fn get_dir(path: &str) -> io::Result<Vec<String>> {
    match fs::read_dir(path) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect(),
            Err(e) => {
                eprintln!("[FS WARN] No se pudo leer '{}': {}", path, e);
                Vec::new()
            }
        }
}

#[derive(RustEmbed)]
#[folder = "assets/"]
#[include = "*.json"]
#[include = "*.conf"]
//const CONF2: &str = include_str!("../assets/config2.json");
pub struct Assets;

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

//PHP
//Paquetes PHP que puedo instalar desde apt
pub fn get_installable_php() -> Vec<String> {
    let (stdout, exito) = output("apt-cache", &["pkgnames", "php"]);
    if !exito { return Vec::new(); }

    let paquetes: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
    sort_versions(paquetes)
}

/// Consulta en dpkg la base de datos real de paquetes PHP instalados
pub fn get_installed_php() -> Vec<String> {
    // 1. Manejamos el Result de list_dir inmediatamente
    let Ok(lista) = get_dir("/usr/bin") else {
        return Vec::new();
    };

    // 2. Filtramos la lista ya obtenida
    let paquetes: Vec<String> = lista.into_iter()
        .filter(|name| name.starts_with("php"))
        .collect();

    sort_versions(paquetes)
}

pub fn get_installed_php_fpm() -> Vec<String> {
    let Ok(names) = get_dir("/etc/php/") else { return Vec::new(); };

    let paquetes: Vec<String> = names
        .into_iter()
        .filter(|name| Path::new("/etc/php/").join(name).join("fpm").exists())
        .map(|name| format!("php{}", name))
        .collect();

    sort_versions(paquetes)
}

pub fn search_module(version: &str, modulo: &str) -> bool {
    // Construimos el nombre del paquete esperado: php8.5-fpm
    let package_name = format!("php{}-{}", version, modulo);

    // dpkg-query -W nos dice si el paquete está instalado o no
    // -W = --show (busca en la base de datos local)
    // -f = --showformat (especificamos que solo devuelva el estado)
    let output = Command::new("dpkg-query")
        .args(["-W", "-f='${db:Status-Status}'", &package_name])
        .output();

    match output {
        Ok(out) => {
            // Si el estado es "installed", entonces existe
            let status = String::from_utf8_lossy(&out.stdout);
            status.contains("installed")
        }
        Err(_) => false, // Si dpkg-query falla (ej: paquete no encontrado), retorna false
    }
}


//Apache
fn get_apache_dir(subdir: &str) -> Vec<String> {
    let path = format!("/etc/apache2/{}/", subdir);
    get_dir(&path).unwrap_or_else(|e| {
        println!("{} Error leyendo {}: {}", ERROR_PC, subdir, e);
        Vec::new()
    })
}

pub fn get_available_sites_apache() -> Vec<String> { get_apache_dir("sites-available") }
pub fn get_enabled_sites_apache()   -> Vec<String> { get_apache_dir("sites-enabled") }


