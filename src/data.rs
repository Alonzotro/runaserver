use crate::public::{ARROW, ERROR_PC, ERROR_YOU, INFO, LOG_ERRORES, OK, Settings, WARNING, clear_screen, error_log, evaluate, findout_software, line, output, print_header, read_in, search_json};
use crate::servicios::{update};
use std::fs::{self, OpenOptions};
use std::io::{Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::collections::BTreeSet;

pub fn sort_versions(pkg_list: Vec<String>) -> Vec<String> {
    let mut versiones: BTreeSet<(u32, u32)> = BTreeSet::new();

    // Recorremos el vector directamente (cada 'pkg' es un String del vector)
    for pkg in pkg_list {
        let pkg = pkg.trim(); // Limpiamos espacios por si acaso

        // 1. Debe empezar por "php" seguido de un número
        let Some(resto) = pkg.strip_prefix("php") else { continue };
        if !resto.starts_with(|c: char| c.is_ascii_digit()) { continue }

        // 2. Extraemos solo la parte numérica con punto ("8.1-cli" → "8.1")
        let version_str: String = resto
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();

        // 3. Separamos en major y minor
        let Some((major_str, minor_str)) = version_str.split_once('.') else { continue };

        // 4. minor_str puede ser "1" o "10-algo" → tomamos solo los dígitos
        let minor_clean: String = minor_str
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();

        // 5. Parseo seguro
        if let (Ok(major), Ok(minor)) = (major_str.parse::<u32>(), minor_clean.parse::<u32>()) {
            versiones.insert((major, minor));
        }
    }

    // Convertimos de vuelta a Vec<String> ordenados
    versiones
        .into_iter()
        .map(|(maj, min)| format!("{}.{}", maj, min))
        .collect()
}

//Paquetes PHP que puedo instalar desde apt
pub fn get_installable_php() -> Vec<String> {
    let (stdout, exito) = output("apt-cache", &["pkgnames", "php"]);
    if !exito { return Vec::new(); }

    let paquetes: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
    sort_versions(paquetes)
}

/// Consulta en dpkg la base de datos real de paquetes PHP instalados
pub fn get_installed_php() -> Vec<String> {
    let Ok(entries) = fs::read_dir("/usr/bin") else { return Vec::new(); };

    let paquetes: Vec<String> = entries.flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name.starts_with("php"))
        .collect();

    sort_versions(paquetes)
}

pub fn get_installed_php_fpm() -> Vec<String> {
    // Buscamos en /etc/php/, donde residen las configuraciones de cada versión
    let Ok(entries) = fs::read_dir("/etc/php/") else { return Vec::new(); };
    
    let paquetes: Vec<String> = entries.flatten()
        .filter_map(|e| {
            let path = e.path();
            let file_name = e.file_name().into_string().ok()?;
            
            // Verificamos que sea un directorio y contenga la subcarpeta 'fpm'
            if path.is_dir() && path.join("fpm").exists() {
                // Generamos un nombre que sort_versions pueda procesar (ej: "php8.1")
                Some(format!("php{}", file_name))
            } else {
                None
            }
        })
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