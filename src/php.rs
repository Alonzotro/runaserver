// ==========================================
// PHP MANAGEMENT
// ==========================================
use crate::public::*;
use crate::data::*;
use crate::checker::*;

use crate::apache::{restart_apache};
use crate::servicios::{update};
use std::fs::{self, OpenOptions};
use std::io::{Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::collections::BTreeSet;

//No visual
/// Consulta en apt-cache los paquetes PHP disponibles en los repositorios
pub fn versiones_instaladas_php() {
    let ver = get_installed_php();
    list_vec(&ver);
}

pub fn install_php() {
    let versiones = get_installable_php();


    //Si no se encuentran php disponibles entra este bloque
    if versiones.is_empty() {
        println!("Error: No se encontraron versiones de PHP disponibles en tus repositorios de apt.");
        println!("Asegúrate de tener configurado el repositorio de Ondřej Surý.");
        return;
    }


    //Imprime el menu para decidir la version de PHP
    print_header("VERSIONES DE PHP DISPONIBLES");
    list_vec(&versiones);
    let len = versiones.len();
    let input = read_in(&format!("Selecciona una opción [1-{}]: ", len));
    
    if valid_input(&input, len) == false {
        return;
    }

    let seleccion: usize = input.trim().parse().unwrap_or(0);
    if seleccion < 1 || seleccion > versiones.len() {
        println!("[X] Opción inválida.");
        return;
    }
    let version_php = &versiones[seleccion - 1];
    clear_screen();


    println!("Actualizando repositorios...");
    update();


    println!("Filtrando módulos compatibles para PHP {}...", version_php);
    let sufijos  = search_json("php_modules.json", "modules");
    let paquetes_raw: Vec<String> = sufijos
    .into_iter()
    .filter(|s| !s.is_empty()) // Filtro de seguridad: ignorar sufijos vacíos
    .map(|s| format!("php{}-{}", version_php, s))
    .collect();
    let (packages,_) = findout_software(&paquetes_raw);



    if packages.is_empty() {
        println!("[X] No se encontró ningún paquete válido para instalar.");
        return;
    }


    println!("\nInstalando PHP {} con {} módulos válidos detectados...", version_php, packages.len());

    let mut apt_inst = Command::new("apt-get")
        .args(&["install", "y"])
        .args(&packages)
        .stdout(Stdio::null())
        .stderr(error_log()).status();
    evaluate(apt_inst);


    disable_all_fpm_apache();
}

fn disable_all_fpm_apache() {
    // 1. Aseguramos que Apache tenga el proxy cargado (necesario siempre)
    let _ = Command::new("a2enmod").args(["proxy_fcgi", "proxy"]).status();

    // 2. Buscamos todas las configuraciones de PHP-FPM habilitadas en Apache
    // y las deshabilitamos una por una.
    if let Ok(entries) = std::fs::read_dir("/etc/apache2/conf-enabled/") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                // Buscamos archivos que sigan el patrón php*-fpm.conf
                if name.starts_with("php") && name.ends_with("-fpm.conf") {
                    let conf_name = name.trim_end_matches(".conf");
                    println!("[*] Deshabilitando configuración: {}", conf_name);
                    let _ = Command::new("a2disconf").arg(conf_name).status();
                }
            }
        }
    }

    // 3. Recargamos para aplicar la limpieza
    restart_apache();
    println!("[✓] Todas las configuraciones FPM han sido deshabilitadas.");
}

pub fn desinstalacion_php() {
    // 1. Obtener las versiones que realmente están instaladas
    let versiones_instaladas = get_installed_php();

    if versiones_instaladas.is_empty() {
        println!("[X] No se detectó ninguna versión de PHP instalada en el sistema.");
        return;
    }

    clear_screen();
    print_header("VERSIONES DE PHP INSTALADAS");
    // 2. Iterar visualmente con base 1
    for (i, ver) in versiones_instaladas.iter().enumerate() {
        println!("{}) PHP {}", i + 1, ver);
    }

    print_header("Enter o cualquier otra tecla para cancelar.");

    
    let seleccion_raw = read_in(&format!("Selecciona la versión que deseas eliminar [1-{}]: ", versiones_instaladas.len()));
    
    // 3. Leer y parsear seguro (si mete texto o Enter vacío, cae en 0)
    let seleccion: usize = seleccion_raw.trim().parse().unwrap_or(0);

    // 4. Validación de límites
    if seleccion < 1 || seleccion > versiones_instaladas.len() {
        println!("Operación cancelada o opción inválida.");
        return;
    }

    // 5. Mapeo inverso para obtener la versión exacta del array
    let version = &versiones_instaladas[seleccion - 1];

    clear_screen();

    println!("=== Iniciando la desinstalación completa de PHP {} ===", version);
    println!("Eliminando paquetes y configuraciones de PHP {}...", version);

    let target_pkg = format!("php{}*", version);
    let target_mod = format!("libapache2-mod-php{}", version);

    // Usamos la Santísima Trinidad para scripts (noninteractive, -y) y apt-get estable
    let status = Command::new("apt-get")
        .env("DEBIAN_FRONTEND", "noninteractive")
        .args(["purge", "-y", &target_pkg, &target_mod])
        .stdout(Stdio::null())
        .stderr(error_log())
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("[✓] PHP {} y todos sus módulos asociados han sido eliminados.", version);
        }
        _ => {
            println!("[X] Hubo un problema al purgar los paquetes de PHP {}. Revisa el log.", version);
        }
    }

    // Limpieza residual automatizada y silenciosa
    println!("\nLimpiando dependencias residuales del sistema...");
    let _ = Command::new("apt-get")
        .env("DEBIAN_FRONTEND", "noninteractive")
        .args(["autoremove", "--purge", "-y", "-qq"])
        .stdout(Stdio::null())
        .stderr(error_log())
        .status();

    let _ = Command::new("apt-get")
        .args(["clear"])
        .stdout(Stdio::null())
        .stderr(error_log())
        .status();
    print_header("¡Proceso de limpieza terminado!")
}

pub fn modulos_php() {
    // 1. Obtenemos las versiones instaladas directamente en un Vec<String>
    let versiones_instaladas = get_installed_php();
    
    // Leemos la opción del usuario
    let seleccion_raw = read_in(&format!("Selecciona la versión para gestionar sus módulos [1-{}]: ", versiones_instaladas.len()));
    let seleccion: usize = seleccion_raw.trim().parse().unwrap_or(0);

    if seleccion < 1 || seleccion > versiones_instaladas.len() {
        println!("[X] Opción inválida. Operación cancelada.");
        return;
    }

    // 2. Extraemos la versión exacta basada en el número seleccionado
    let ver_mod = &versiones_instaladas[seleccion - 1];

    clear_screen();
    print_header("Módulos instalados en el sistema para PHP {ver_mod}");
    
    let mut modulos_instalados = Vec::new();
    if let Ok(output) = Command::new("dpkg").arg("-l").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let filtro = format!("php{}-", ver_mod);
        for line in stdout.lines() {
            if line.contains(&filtro) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    modulos_instalados.push(parts[1].to_string());
                }
            }
        }
    }

    if modulos_instalados.is_empty() {
        println!("No se encontraron módulos específicos instalados para PHP {}.", ver_mod);
        return;
    }

    // 3. Imprimimos los módulos con números para que sea fácil seleccionarlos
    for (i, modulo) in modulos_instalados.iter().enumerate() {
        println!("{}) {}", i + 1, modulo);
    }
    line();

    // 4. Permitimos borrar múltiples módulos ingresando solo sus números
    let input_mods_raw = read_in("Ingresa los NÚMEROS de los módulos a borrar separados por espacio (Ej: 1 3 5) o Enter para omitir: ");
    
    if input_mods_raw.trim().is_empty() {
        println!("Operación finalizada sin borrar módulos.");
        return;
    }

    let mut mods_a_borrar: Vec<String> = Vec::new();
    
    // Procesamos cada número ingresado por el usuario
    for num_str in input_mods_raw.split_whitespace() {
        if let Ok(idx) = num_str.parse::<usize>() {
            if idx > 0 && idx <= modulos_instalados.len() {
                // Añadimos el nombre real del módulo al vector de borrado
                mods_a_borrar.push(modulos_instalados[idx - 1].clone());
            } else {
                println!("   [!] Número '{}' fuera de rango, ignorando...", idx);
            }
        } else {
            println!("   [!] '{}' no es un número válido, ignorando...", num_str);
        }
    }

    if !mods_a_borrar.is_empty() {
        println!("Eliminando módulos seleccionados...");
        let mut apt_purge = Command::new("apt-get");
        apt_purge
            .arg("purge")
            .arg("-y")
            .args(&mods_a_borrar) // Pasamos el array de Strings con los nombres de los paquetes
            .stdout(Stdio::null())
            .stderr(error_log());

        if apt_purge.status().is_ok() {
            let _ = Command::new("apt-get").args(&["autoremove", "-y"]).stdout(Stdio::null()).stderr(error_log()).status();
            println!("   [✓] Módulos eliminados correctamente.");
        } else {
            println!("   [X] Error al intentar eliminar los módulos.");
        }
    } else {
        println!("   [X] No se seleccionó ningún módulo válido para borrar.");
    }
}

pub fn cambiar_php() {
    let _ = Command::new("update-alternatives").args(&["--config", "php"]).status();
}

pub fn php_activo() {
    print_header("ESTADO ACTUAL DE PHP");

    // 1. Consultar la versión de PHP del sistema (CLI)
    // Ejecutamos un pequeño script de PHP para que nos devuelva solo "8.1", "8.2", etc.
    let cli_php = match Command::new("php")
        .args(&["-r", "echo PHP_MAJOR_VERSION.'.'.PHP_MINOR_VERSION;"])
        .output() 
    {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).to_string()
        },
        _ => "No instalado o no disponible".to_string(),
    };

    // 2. Consultar la versión de PHP activa en Apache
    // Buscamos el archivo del módulo de PHP habilitado en la configuración de Apache
    let cmd_apache = "ls /etc/apache2/conf-enabled/php*-fpm.conf 2>/dev/null | grep -oE '[0-9]+\\.[0-9]+' | sort -uV";
    let apache_php = match Command::new("bash")
        .args(&["-c", cmd_apache])
        .output() 
    {
        Ok(output) if output.status.success() && !output.stdout.is_empty() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        },
        _ => "Ningún módulo de PHP activo".to_string(),
    };

    // Imprimir los resultados
    println!("PHP del Sistema (CLI) : {}", cli_php);
    println!("PHP activo en Apache  : {}", apache_php);
    line();
}