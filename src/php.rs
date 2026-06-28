// ==========================================
// PHP MANAGEMENT
// ==========================================
use crate::public::{ARROW, ERROR_PC, ERROR_YOU, INFO, LOG_ERRORES, OK, Settings, WARNING, clear_screen, error_log, evaluate, findout_software, line, output, print_header, read_in};
use crate::servicios::{update};
use std::fs::{self, OpenOptions};
use std::io::{Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::collections::BTreeSet;

//No visual
/// Consulta en apt-cache los paquetes PHP disponibles en los repositorios
pub fn get_availables_php(solo_php8: bool) -> Vec<String> {
    let (stdout, exito, _) = output("apt-cache", &["pkgnames", "php"]);
    if !exito {
        return Vec::new();
    }
    
    extract_and_sort_versions(&stdout, solo_php8)
}

/// Consulta en dpkg la base de datos real de paquetes PHP instalados
pub fn get_installed_php() -> Vec<String> {
    // -W: Muestra registros | -f='${Package}\n': Solo el nombre crudo de instalados
    let (stdout, exito, _) = output("dpkg-query", &["-W", "-f=${Package}\n", "php*"]);
    if !exito {
        return Vec::new();
    }

    extract_and_sort_versions(&stdout, false)
}

// =====================================================================
// MOTOR DE PARSEO ESTRICTO (Cero duplicación de código)
// =====================================================================

#[inline]
fn extract_and_sort_versions(raw_text: &str, solo_php8: bool) -> Vec<String> {
    let mut versiones: BTreeSet<(u32, u32)> = BTreeSet::new();

    for linea in raw_text.lines() {
        let pkg = linea.trim();

        // 1. Debe empezar por "php" seguido de un número (Descarta phpmyadmin, phpunit...)
        let Some(resto) = pkg.strip_prefix("php") else { continue };
        if !resto.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }

        // 2. Tomamos bytes de dígitos y el punto principal (ej: "8.1-cli" -> "8.1")
        let version_str: String = resto
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();

        // 3. Corte exacto en el primer punto
        let Some((major_str, minor_str)) = version_str.split_once('.') else {
            continue;
        };

        // 4. Transformación segura a enteros (Protege contra "8." o "8.1.0")
        let minor_clean: String = minor_str.chars().take_while(|c| c.is_ascii_digit()).collect();
        
        let (Ok(major), Ok(minor)) = (major_str.parse::<u32>(), minor_clean.parse::<u32>()) else {
            continue;
        };

        if !solo_php8 || major >= 8 {
            versiones.insert((major, minor));
        }
    }

    // El BTreeSet ordena matemáticamente (8, 2) antes que (8, 10).
    // Al convertir a String al final, blindamos el orden visual en Linux.
    versiones
        .into_iter()
        .map(|(maj, min)| format!("{}.{}", maj, min))
        .collect()
}

pub fn install_php() {
    let config = Settings::load();
    let versiones_disponibles = get_availables_php(config.admin_mode);

    if versiones_disponibles.is_empty() {
        println!("Error: No se encontraron versiones de PHP disponibles en tus repositorios de apt.");
        println!("Asegúrate de tener configurado el repositorio de Ondřej Surý.");
        return;
    }

    clear_screen();
    print_header("VERSIONES DE PHP DISPONIBLES");
    for (i, ver) in versiones_disponibles.iter().enumerate() {
        println!("{}) PHP {}", i + 1, ver);
    }
    line();
    
    let seleccion_raw = read_in(&format!("Selecciona una opción [1-{}]: ", versiones_disponibles.len()));
    let seleccion: usize = seleccion_raw.trim().parse().unwrap_or(0);

    if seleccion < 1 || seleccion > versiones_disponibles.len() {
        println!("[X] Opción inválida.");
        return;
    }

    clear_screen();

    println!("Actualizando repositorios...");
    update();

    let version_php = &versiones_disponibles[seleccion - 1];
    println!("Filtrando módulos compatibles para PHP {}...", version_php);

    let paquetes_raw = &[
        "common", "cli", "dev", "mysql", "sqlite3", "pgsql", "mongodb", "gd", 
        "imagick", "exif", "curl", "openssl", "ssl", "sodium", "zip", "bz2", 
        "xml", "xmlrpc", "soap", "opcache", "memcache", "redis", "intl", 
        "mbstring", "bcmath", "imap", "pspell", "snmp", "tidy", "mcrypt", 
        "json", "recode", "pear", "zlib", "fpm"
    ];

    let mut paquetes_solicitados = paquetes_raw
    .iter()
    .map(|sufijo| format!("php{}-{}", version_php, sufijo))
    .collect();

    let (_,paquetes_solicitados) = findout_software(paquetes_solicitados);

    let mut paquetes_validos = Vec::new();
    
    if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open(LOG_ERRORES) {
        for pkg in paquetes_solicitados {
            // Ejecutamos la consulta de política del paquete
            let output = Command::new("apt-cache")
                .args(&["policy", &pkg])
                .output();

            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                
                // CORRECCIÓN CLAVE: Verificamos que exista la palabra "Candidate:" (lo que confirma que el paquete es real en el repo)
                // y que NO diga "Candidate: (none)" (lo que indicaría que existe en la base pero no se puede descargar).
                if stdout.contains("Candidate:") && !stdout.contains("Candidate: (none)") {
                    paquetes_validos.push(pkg);
                } else {
                    let _ = writeln!(log_file, "E: Unable to locate package {}", pkg);
                    println!("[!] No disponible para esta versión, se omite: {}", pkg);
                }
            }
        }
    }

    if paquetes_validos.is_empty() {
        println!("[X] No se encontró ningún paquete válido para instalar.");
        return;
    }

    println!("\nInstalando PHP {} con {} módulos válidos detectados...", version_php, paquetes_validos.len());

    let mut apt_inst = Command::new("apt-get");
    apt_inst
        .arg("install")
        .arg("-y")
        .args(&paquetes_validos)
        .stdout(Stdio::null())
        .stderr(error_log());
    
    let nombre_modulo = format!("php{}", version_php);
    let nombre_fpm = format!("php{}-fpm", version_php);
    match apt_inst.status() {
        Ok(status) => {
            if status.success() {
                println!("[✓] PHP {} e instalaciones completadas con éxito.", version_php);
                    if let Ok(enabling) = Command::new("a2dismod").arg(nombre_modulo).stdout(Stdio::null()).status() {
                        if enabling.success() {
                            println!("[✓] Se deshabilito php{}.", version_php);
                            match Command::new("a2enconf").arg(&nombre_fpm).stdout(Stdio::null()).status() {
                                Ok(conf) if conf.success() => println!("[✓] Se habilito php{}-fpm.", version_php),
                                _ => println!("[X] Hubo un problema al habilitar php{}-fpm.", version_php),
                                }
                            }
                        } else {
                            println!("[X] Hubo un problema al deshabilitar php{}.", version_php);
                        }
                } else {
                if Path::new(&format!("/usr/bin/php{}", version_php)).exists() {
                    println!("[✓] PHP {} base instalado. Módulos no disponibles omitidos (Ver en: {})", version_php, LOG_ERRORES);
                } else {
                    println!("[X] Hubo errores críticos durante la instalación. Revisa: {}", LOG_ERRORES);
                }
            }
        }
        Err(_) => {
            println!("[X] No se pudo ejecutar el gestor de paquetes apt.");
        }
    }
}

pub fn versiones_instaladas_php() -> (bool,Vec<String>) {
    print_header("VERSIONES DE PHP INSTALADAS");

    let versiones_instaladas = get_installed_php();

    if versiones_instaladas.is_empty() {
        println!("{WARNING} No hay ninguna versión de PHP instalada.");
        line();
        return (false, versiones_instaladas);
    }

    for (i, ver) in versiones_instaladas.iter().enumerate() {
        println!("{}) PHP {}", i + 1, ver);
    }
    line();
    return (true, versiones_instaladas); 
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
    let (_,versiones_instaladas) = versiones_instaladas_php();
    
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