// ==========================================
// PHP MANAGEMENT
// ==========================================
use crate::{registrar_log_error, leer_linea, limpiar_pantalla, Evaluable, evaluate, LOG_ERRORES, OK, WARNING, ERROR_YOU, ERROR_PC};
use std::fs::{self, OpenOptions};
use std::io::{Write};
use std::path::Path;
use std::process::{Command, Stdio};

//No visual

fn get_availables_php() -> Vec<String> {
    let mut versiones = Vec::new();
    
    // Un solo comando con toda la tubería para buscar, filtrar y ordenar las versiones
    let cmd = "apt-cache policy php[0-9].* 2>/dev/null | grep -oE '^php[0-9]\\.[0-9]' | grep -oE '[0-9]+\\.[0-9]+' | sort -uV";      
    
    // Pasamos el comando directo a evaluate!. 
    // Usamos 'false' para que trabaje en silencio si todo va bien.
    if let Some(out) = evaluate!(Command::new("bash").args(&["-c", cmd]).output(), false) {
        // Convertimos los bytes de la terminal a texto legible
        let stdout = String::from_utf8_lossy(&out.stdout);
        
        // Procesamos línea por línea
        for line in stdout.lines() {
            let versión_limpia = line.trim();
            if !versión_limpia.is_empty() {
                versiones.push(versión_limpia.to_string());
            }
        }
    }
        
    versiones
}

//No visual
fn get_installed_php() -> Vec<String> {
    let mut versiones = Vec::new();
    
    // Leemos el directorio /usr/bin de forma nativa
    if let Ok(entries) = fs::read_dir("/usr/bin") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                // Buscamos archivos que empiecen con "php" (ej. php8.1, php8.2)
                if file_name.starts_with("php") && file_name.len() > 3 {
                    let resto = &file_name[3..];
                    
                    // Nos aseguramos de que lo que sigue sea un número (para ignorar php-config, phpize, etc.)
                    if resto.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                        // Extraemos solo los números y puntos (por si hay cosas como php8.2-cgi)
                        let version: String = resto
                            .chars()
                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                            .collect();
                        
                        if version.contains('.') {
                            versiones.push(version);
                        }
                    }
                }
            }
        }
    }
    
    // Ordenamos y eliminamos duplicados (el equivalente a | sort -u)
    versiones.sort();
    versiones.dedup();
    
    versiones
}

pub fn install_php() {
    let versiones_disponibles = get_availables_php();



    if versiones_disponibles.is_empty() {
        println!("Error: No se encontraron versiones de PHP disponibles en tus repositorios de apt.");
        println!("Asegúrate de tener configurado el repositorio de Ondřej Surý.");
        return;
    }

    limpiar_pantalla();

    println!("=========================================");
    println!("       VERSIONES DE PHP DISPONIBLES      ");
    println!("=========================================");
    for (i, ver) in versiones_disponibles.iter().enumerate() {
        println!("{}) PHP {}", i + 1, ver);
    }
    println!("=========================================");
    
    let seleccion_raw = leer_linea(&format!("Selecciona una opción [1-{}]: ", versiones_disponibles.len()));
    let seleccion: usize = seleccion_raw.trim().parse().unwrap_or(0);

    if seleccion < 1 || seleccion > versiones_disponibles.len() {
        println!("[X] Opción inválida.");
        return;
    }

    limpiar_pantalla();

    println!("Actualizando repositorios...");
    let _ = Command::new("apt-get").args(&["update"]).stdout(Stdio::null()).stderr(registrar_log_error()).status();
    let _ = Command::new("apt-get").args(&["upgrade", "-y"]).stdout(Stdio::null()).stderr(registrar_log_error()).status();

    let version_php = &versiones_disponibles[seleccion - 1];
    println!("Filtrando módulos compatibles para PHP {}...", version_php);

    let paquetes_solicitados = vec![
        format!("libapache2-mod-php{}", version_php),
        format!("php{}-common", version_php),
        format!("php{}-cli", version_php),
        format!("php{}-dev", version_php),
        format!("php{}-mysql", version_php),
        format!("php{}-sqlite3", version_php),
        format!("php{}-pgsql", version_php),
        format!("php{}-mongodb", version_php),
        format!("php{}-gd", version_php),
        format!("php{}-imagick", version_php),
        format!("php{}-exif", version_php),
        format!("php{}-curl", version_php),
        format!("php{}-openssl", version_php),
        format!("php{}-ssl", version_php),
        format!("php{}-sodium", version_php),
        format!("php{}-zip", version_php),
        format!("php{}-bz2", version_php),
        format!("php{}-xml", version_php),
        format!("php{}-xmlrpc", version_php),
        format!("php{}-soap", version_php),
        format!("php{}-opcache", version_php),
        format!("php{}-memcache", version_php),
        format!("php{}-redis", version_php),
        format!("php{}-intl", version_php),
        format!("php{}-mbstring", version_php),
        format!("php{}-bcmath", version_php),
        format!("php{}-imap", version_php),
        format!("php{}-pspell", version_php),
        format!("php{}-snmp", version_php),
        format!("php{}-tidy", version_php),
        format!("php{}-mcrypt", version_php),
        format!("php{}-json", version_php),
        format!("php{}-recode", version_php),
        format!("php{}-pear", version_php),
        format!("php-pear"),
        format!("php{}-zlib", version_php),
        format!("php{}-fpm", version_php),

    ];

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
        .stderr(registrar_log_error());
    
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


pub fn versiones_instaladas_php() -> bool {
    println!("=========================================");
    println!("       VERSIONES DE PHP INSTALADAS       ");
    println!("=========================================");
    let versiones = get_installed_php();

    if versiones.is_empty() {
        println!("[!] No hay ninguna versión de PHP instalada.");
        println!("=========================================");
        return false;
    }

    for (i, ver) in versiones.iter().enumerate() {
        println!("{}) PHP {}", i + 1, ver);
    }
    println!("=========================================");
    true
}

pub fn desinstalacion_php() {
    // 1. Obtener las versiones que realmente están instaladas
    let versiones_instaladas = get_installed_php();

    if versiones_instaladas.is_empty() {
        println!("[X] No se detectó ninguna versión de PHP instalada en el sistema.");
        return;
    }

    limpiar_pantalla();

    println!("=========================================");
    println!("      VERSIONES DE PHP INSTALADAS        ");
    println!("=========================================");
    // 2. Iterar visualmente con base 1
    for (i, ver) in versiones_instaladas.iter().enumerate() {
        println!("{}) PHP {}", i + 1, ver);
    }
    println!("=========================================");
    println!("Enter o cualquier otra tecla para cancelar.");
    println!("=========================================");
    
    let seleccion_raw = leer_linea(&format!("Selecciona la versión que deseas eliminar [1-{}]: ", versiones_instaladas.len()));
    
    // 3. Leer y parsear seguro (si mete texto o Enter vacío, cae en 0)
    let seleccion: usize = seleccion_raw.trim().parse().unwrap_or(0);

    // 4. Validación de límites
    if seleccion < 1 || seleccion > versiones_instaladas.len() {
        println!("Operación cancelada o opción inválida.");
        return;
    }

    // 5. Mapeo inverso para obtener la versión exacta del array
    let version = &versiones_instaladas[seleccion - 1];

    limpiar_pantalla();

    println!("=== Iniciando la desinstalación completa de PHP {} ===", version);
    println!("Eliminando paquetes y configuraciones de PHP {}...", version);

    let target_pkg = format!("php{}*", version);
    let target_mod = format!("libapache2-mod-php{}", version);

    // Usamos la Santísima Trinidad para scripts (noninteractive, -y) y apt-get estable
    let status = Command::new("apt-get")
        .env("DEBIAN_FRONTEND", "noninteractive")
        .args(["purge", "-y", &target_pkg, &target_mod])
        .stdout(Stdio::null())
        .stderr(registrar_log_error())
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
        .stderr(registrar_log_error())
        .status();

    let _ = Command::new("apt-get")
        .args(["clean"])
        .stdout(Stdio::null())
        .stderr(registrar_log_error())
        .status();

    println!("=========================================");
    println!("     ¡Proceso de limpieza terminado!     ");
    println!("=========================================");
}

pub fn modulos_php() {
    // 1. Obtenemos las versiones instaladas directamente en un Vec<String>
    let versiones_instaladas = get_installed_php();

    if versiones_instaladas.is_empty() {
        println!("   [!] No hay ninguna versión de PHP instalada.");
        return;
    }

    limpiar_pantalla();

    println!("=========================================");
    println!("      VERSIONES DE PHP INSTALADAS        ");
    println!("=========================================");
    // Imprimimos las versiones con su número correspondiente
    for (i, ver) in versiones_instaladas.iter().enumerate() {
        println!("{}) PHP {}", i + 1, ver);
    }
    println!("=========================================");
    
    // Leemos la opción del usuario
    let seleccion_raw = leer_linea(&format!("Selecciona la versión para gestionar sus módulos [1-{}]: ", versiones_instaladas.len()));
    let seleccion: usize = seleccion_raw.trim().parse().unwrap_or(0);

    if seleccion < 1 || seleccion > versiones_instaladas.len() {
        println!("[X] Opción inválida. Operación cancelada.");
        return;
    }

    // 2. Extraemos la versión exacta basada en el número seleccionado
    let ver_mod = &versiones_instaladas[seleccion - 1];

    limpiar_pantalla();
    println!("--- Módulos instalados en el sistema para PHP {} ---", ver_mod);
    
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
    println!("--------------------------------------------------------");

    // 4. Permitimos borrar múltiples módulos ingresando solo sus números
    let input_mods_raw = leer_linea("Ingresa los NÚMEROS de los módulos a borrar separados por espacio (Ej: 1 3 5) o Enter para omitir: ");
    
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
            .stderr(registrar_log_error());

        if apt_purge.status().is_ok() {
            let _ = Command::new("apt-get").args(&["autoremove", "-y"]).stdout(Stdio::null()).stderr(registrar_log_error()).status();
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
    println!("=========================================");
    println!("          ESTADO ACTUAL DE PHP           ");
    println!("=========================================");

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
    println!("=========================================");
}