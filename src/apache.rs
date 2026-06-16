// ==========================================
// APACHE MANAGEMENT
// ==========================================
//Hola, smn
//mod php;
use crate::{registrar_log_error, leer_linea, limpiar_pantalla, Evaluable, evaluate, OK, WARNING, ERROR_YOU, ERROR_PC};
use crate::php::{versiones_instaladas_php};
use std::fs::{File, OpenOptions};
use std::fs;
use std::io::{self, Write, BufRead, BufReader};
use std::error::Error;
use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use regex::Regex;

pub fn version_apache() -> io::Result<()> {
    println!("=========================================");
    
    // 1. Usamos la macro POR DENTRO para ejecutar y validar el comando de forma segura.
    // Usamos 'false' para que no imprima errores feos de sistema si Apache no existe.
    let Some(output) = evaluate!(Command::new("apache2").arg("-v").output(), false) else {
        // Si el comando ni siquiera existe, devolvemos un error NotFound compatible con la macro
        return Err(io::Error::new(io::ErrorKind::NotFound, rust_i18n::t!("APACHE_NOT_FOUND")));
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 2. Procesamos la salida del comando para buscar la versión
    if let Some(idx) = stdout.find("Apache/") {
        let sub = &stdout[idx..];
        let version = sub.split_whitespace().next().unwrap_or("");
        
        println!("{} {}", OK, rust_i18n::t!("APACHE_INSTALLED"));
        println!("[•] {}: {}", rust_i18n::t!("DETECTED_VERSION"), version);
        
        return Ok(()); // Éxito rotundo
    }

    // Si el comando se ejecutó pero la salida no era la esperada
    Err(io::Error::new(io::ErrorKind::InvalidData, rust_i18n::t!("APACHE_NOT_FOUND")))
}

pub fn config_apache() {
    println!("{}", rust_i18n::t!("CONFIGURING_APACHE"));
    // 1. Configurar el Firewall
    println!("{}", rust_i18n::t!("CONFIGURING_FIREWALL"));
    if !evaluate!(Command::new("ufw").args(&["allow", "Apache Full"]).stdout(Stdio::null()).stderr(registrar_log_error()).status(), true) {
        return;
    }
    // 2. Reiniciar Apache
    println!("{}", rust_i18n::t!("RESTARTING_APACHE"));
    if !evaluate!(Command::new("systemctl").args(&["restart", "apache2"]).stdout(Stdio::null()).stderr(registrar_log_error()).status(), true) {
        return;
    }
    // 3. Deshabilitar MPM Prefork
    println!("{}", rust_i18n::t!("DISABLING_MPM"));
    if !evaluate!(Command::new("a2dismod").arg("mpm_prefork").stdout(Stdio::null()).stderr(registrar_log_error()).status(), true) {
        return;
    }
    // 4. Habilitar MPM Event y módulos FCGI
    println!("{}", rust_i18n::t!("ENABLING_FPM"));
    if !evaluate!(Command::new("a2enmod").args(&["mpm_event", "proxy_fcgi", "setenvif"]).stdout(Stdio::null()).stderr(registrar_log_error()).status(), true) {
        return;
    }
    // 5. Habilitar módulos adicionales
    println!("{}", rust_i18n::t!("ENABLING_ADDITIONAL_MODULES"));
    if !evaluate!(Command::new("a2enmod").args(&["actions", "fcgid", "alias", "proxy_fcgi"]).stdout(Stdio::null()).stderr(registrar_log_error()).status(), true) {
        return;
    }
    // ¡Éxito total!
    println!("{} {}", OK, rust_i18n::t!("CONFIGURED_SUCCESS"));
}

pub fn reiniciar_apache() {
    // 1. Describimos la acción que se va a ejecutar
    println!("{}", rust_i18n::t!("RESTARTING_APACHE"));

    // 2. Evaluamos el comando directamente con la macro
    if !evaluate!(Command::new("systemctl").args(&["restart", "apache2"]).status(), true) {
        // Si la macro detecta un fallo, añadimos el tip de ayuda internacionalizado
        println!("[X] {}", rust_i18n::t!("APACHE_RESTART_ERROR_TIP"));
    }
}

pub fn add_site_apache(ip: &str) {
    // 1. Solicitar y validar el nombre del sitio
    let sitio_raw = leer_linea(&rust_i18n::t!("PROMPT_SITE_NAME"));
    let sitio = sitio_raw.trim();

    if sitio.is_empty() || !sitio.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-') {
        println!("[X] {}", rust_i18n::t!("INVALID_SITE_NAME"));
        return;
    }

    let conf_dir = "/etc/apache2/sites-available";
    let conf_file = format!("{}/{}.conf", conf_dir, sitio);
    let web_dir = format!("/var/www/{}", sitio);

    if Path::new(&conf_file).exists() {
        println!("[X] {}", rust_i18n::t!("SITE_ALREADY_EXISTS", site = sitio));
        return;
    }

    // 2. DETECCIÓN DE VERSIÓN FPM (Usa el impl de comandos con salida de texto)
    let cmd = "ls /etc/apache2/conf-available/php*-fpm.conf 2>/dev/null | grep -oE '[0-9]+\\.[0-9]+' | sort -uV";
    let mut versiones = Vec::new();
    
    if let Some(out) = evaluate!(Command::new("bash").args(&["-c", cmd]).output(), false) {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                versiones.push(line.trim().to_string());
            }
        }
    }

    if versiones.is_empty() {
        println!("[X] {}", rust_i18n::t!("NO_PHP_FPM_CONFIGS"));
        return;
    }

    // 3. Menú de Selección de Versión de PHP
    println!("=========================================");
    println!("     {}     ", rust_i18n::t!("SELECT_PHP_VERSION_TITLE"));
    println!("=========================================");
    for (i, ver) in versiones.iter().enumerate() {
        println!("{}) php{}", i + 1, ver);
    }
    println!("=========================================");
    
    let sel_ver = leer_linea(&rust_i18n::t!("PROMPT_PHP_VERSION"));
    let idx: usize = sel_ver.trim().parse().unwrap_or(0);

    if idx < 1 || idx > versiones.len() {
        println!("[X] {}", rust_i18n::t!("INVALID_PHP_VERSION"));
        return;
    }
    let ver_elegida = &versiones[idx - 1];

    // 4. Generación del Contenido del VirtualHost
    let contenido_vhost = format!(
        "<VirtualHost *:80>\n\
        \tServerName {sitio}.lan\n\
        \tDocumentRoot {web_dir}\n\n\
        \t<Directory {web_dir}>\n\
        \t\tAllowOverride All\n\
        \t\tRequire all granted\n\
        \t</Directory>\n\n\
        \t<FilesMatch \"\\.php$\">\n\
        \t\t<IfModule mod_proxy_fcgi.c>\n\
        \t\t\tSetHandler \"proxy:unix:/run/php/php{ver}-fpm.sock|fcgi://localhost\"\n\
        \t\t</IfModule>\n\
        \t</FilesMatch>\n\
        </VirtualHost>",
        sitio = sitio,
        web_dir = web_dir,
        ver = ver_elegida
    );

    // 5. EJECUCIÓN DE COMANDOS CRÍTICOS PASO A PASO CON EVALUATE!

    // Crear el archivo de configuración (.conf)
    println!("{}", rust_i18n::t!("CREATING_VHOST"));
    if !evaluate!(fs::write(&conf_file, contenido_vhost.as_bytes()), true) {
        return;
    }

    // Crear el directorio web (Nativo de Rust en lugar de invocar `mkdir -p`)
    println!("{}", rust_i18n::t!("CREATING_WEB_DIR"));
    if !evaluate!(fs::create_dir_all(&web_dir), true) {
        return;
    }

    // Asignar los permisos del directorio
    println!("{}", rust_i18n::t!("SETTING_WEB_PERMISSIONS"));
    if !evaluate!(Command::new("chown").args(&["-R", "www-data:www-data", &web_dir]).status(), true) {
        return;
    }

    // Habilitar el sitio en Apache
    println!("{}", rust_i18n::t!("ENABLING_SITE_APACHE"));
    if !evaluate!(Command::new("a2ensite").arg(format!("{}.conf", sitio)).stdout(Stdio::null()).status(), true) {
        return;
    }
    
    // Actualizar el archivo /etc/hosts de forma segura y evaluada
    println!("{}", rust_i18n::t!("UPDATING_HOSTS_FILE"));
    let hosts_result = OpenOptions::new()
        .append(true)
        .open("/etc/hosts")
        .and_then(|mut file| writeln!(file, "{}   {}.lan", ip, sitio));

    if !evaluate!(hosts_result, true) {
        return;
    }

    // Reiniciar Apache para levantar el sitio
    println!("{}", rust_i18n::t!("RESTARTING_APACHE"));
    if !evaluate!(Command::new("systemctl").args(&["restart", "apache2"]).status(), true) {
        return;
    }

    // ¡Éxito absoluto!
    println!("{} {}", OK, rust_i18n::t!("SITE_CREATED_SUCCESS", site = sitio, version = ver_elegida));
}

fn obtener_versiones_fpm_nativas(ruta: &str) -> Vec<String> {
    let mut versiones = Vec::new();

    // 1. Leemos el directorio de forma nativa
    if let Ok(entries) = fs::read_dir(ruta) {
        for entry in entries.flatten() {
            // Convertimos el nombre del archivo a String
            if let Ok(file_name) = entry.file_name().into_string() {
                // Filtramos: debe empezar con "php" y terminar con "-fpm.conf"
                if file_name.starts_with("php") && file_name.ends_with("-fpm.conf") {
                    // Extraemos la versión cortando "php" (3 chars) y "-fpm.conf" (9 chars)
                    // Ejemplo: "php8.2-fpm.conf" -> "8.2"
                    if file_name.len() > 12 {
                        let version = &file_name[3..file_name.len() - 9];
                        
                        // Validamos que lo que quedó sean sólo números y puntos
                        if version.chars().all(|c| c.is_ascii_digit() || c == '.') {
                            versiones.push(version.to_string());
                        }
                    }
                }
            }
        }
    }

    // 2. Réplica exacta de `sort -V` (Ordenamiento Semántico/Natural)
    // Evita que "8.10" se posicione antes que "8.2" al ordenar cadenas
    versiones.sort_by(|a, b| {
        let parse = |s: &str| s.split('.').map(|x| x.parse::<u32>().unwrap_or(0)).collect::<Vec<u32>>();
        parse(a).cmp(&parse(b))
    });

    // 3. Réplica exacta de `sort -u` (Elimina duplicados consecutivos tras ordenar)
    versiones.dedup();

    versiones
}

pub fn disable_php_apache() {

    let versiones_activas = obtener_versiones_fpm_nativas("/etc/apache2/conf-available");

    if versiones_activas.is_empty() {
        println!("   [!] {}", rust_i18n::t!("NO_ACTIVE_PHP_FPM"));
        return;
    }

    limpiar_pantalla();

    // 2. Mostrar menú de configuraciones activas
    println!("=========================================");
    println!("     {}       ", rust_i18n::t!("ACTIVE_FPM_TITLE"));
    println!("=========================================");
    for (i, ver) in versiones_activas.iter().enumerate() {
        println!("{}) php{}-fpm", i + 1, ver);
    }
    println!("=========================================");

    // 3. Lectura de la selección
    let prompt = rust_i18n::t!("PROMPT_DISABLE_FPM", max = versiones_activas.len());
    let seleccion_raw = leer_linea(&prompt);
    let seleccion: usize = seleccion_raw.trim().parse().unwrap_or(0);

    if seleccion < 1 || seleccion > versiones_activas.len() {
        println!("[X] {}", rust_i18n::t!("INVALID_OPTION_CANCELLED"));
        return;
    }

    // 4. Preparar variables de ejecución
    let version_seleccionada = &versiones_activas[seleccion - 1];
    let nombre_fpm = format!("php{}-fpm", version_seleccionada);

    // 5. PASO 1: Deshabilitar la configuración en Apache
    println!("\n{}", rust_i18n::t!("DISABLING_APACHE_CONF", name = &nombre_fpm));
    
    let comando_apache = Command::new("a2disconf")
        .arg(&nombre_fpm)
        .stdout(Stdio::null()) 
        .stderr(registrar_log_error())
        .status();

    if evaluate!(comando_apache, true) {
        // Si se deshabilitó con éxito, pintamos el recordatorio debajo del [✓]
        println!("       {}", rust_i18n::t!("REMINDER_RESTART_APACHE"));
    }

    // 6. PASO 2: Detener el servicio de FPM en el sistema
    println!("\n{}", rust_i18n::t!("STOPPING_FPM_SERVICE", name = &nombre_fpm));
    
    let comando_sys = Command::new("systemctl")
        .args(&["stop", &nombre_fpm])
        .stdout(Stdio::null()) 
        .stderr(registrar_log_error())
        .status();

    evaluate!(comando_sys, true);
}


pub fn enable_php_apache() {
    let versiones_disponibles = obtener_versiones_fpm_nativas("/etc/apache2/conf-enabled");

    if versiones_disponibles.is_empty() {
        println!("   [!] {}", rust_i18n::t!("NO_AVAILABLE_PHP_FPM"));
        return;
    }

    limpiar_pantalla();

    // 2. Mostrar menú de configuraciones disponibles
    println!("=========================================");
    println!("    {}      ", rust_i18n::t!("AVAILABLE_FPM_TITLE"));
    println!("=========================================");
    for (i, ver) in versiones_disponibles.iter().enumerate() {
        println!("{}) php{}-fpm", i + 1, ver);
    }
    println!("=========================================");

    // 3. Lectura de la selección del usuario
    let prompt = rust_i18n::t!("PROMPT_ENABLE_FPM", max = versiones_disponibles.len());
    let seleccion_raw = leer_linea(&prompt);
    let seleccion: usize = seleccion_raw.trim().parse().unwrap_or(0);

    if seleccion < 1 || seleccion > versiones_disponibles.len() {
        println!("[X] {}", rust_i18n::t!("INVALID_OPTION_CANCELLED"));
        return;
    }

    // 4. Extraemos la versión seleccionada
    let version_seleccionada = &versiones_disponibles[seleccion - 1];
    let nombre_fpm = format!("php{}-fpm", version_seleccionada);

    // 5. PASO 1: Iniciar y habilitar el servicio FPM en el sistema operativo
    println!("\n{}", rust_i18n::t!("STARTING_FPM_SERVICE", name = &nombre_fpm));
    
    let status_fpm = Command::new("systemctl")
        .args(&["start", &nombre_fpm])
        .stdout(Stdio::null()) 
        .stderr(registrar_log_error())
        .status();

    // Si la macro detecta que el servicio NO inició, hacemos un 'return' temprano seguro
    if !evaluate!(status_fpm, true) {
        return; 
    }

    // Como el servicio inició bien, lo habilitamos para que arranque con el sistema de forma silenciosa
    let _ = Command::new("systemctl")
        .args(&["enable", &nombre_fpm])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();


    // 6. PASO 2: Enlazar la configuración en Apache usando a2enconf
    println!("\n{}", rust_i18n::t!("ENABLING_APACHE_CONF", name = &nombre_fpm));
    
    let status_apache = Command::new("a2enconf")
        .arg(&nombre_fpm)
        .stdout(Stdio::null()) 
        .stderr(registrar_log_error())
        .status();

    if evaluate!(status_apache, true) {
        // Si se enlazó con éxito, pintamos el recordatorio de reinicio debajo del [✓]
        println!("       {}", rust_i18n::t!("REMINDER_RESTART_APACHE"));
    }
}

pub struct SitioActivo {
    nombre: String,
    ruta: String,
}

struct CmsPaquete {
    nombre: &'static str,
    url: &'static str,
    archivo_cache: &'static str,
    es_zip: bool,
}

pub fn instalar_cms() {
    println!("{}", rust_i18n::t!("SEARCHING_ACTIVE_SITES"));

    let mut sitios = Vec::new();

    if let Ok(entradas) = fs::read_dir("/etc/apache2/sites-enabled") {
        for entrada in entradas.flatten() {
            let path = entrada.path();
            if path.is_file() && path.extension().unwrap_or_default() == "conf" {
                let nombre_sitio = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    for linea in reader.lines().flatten() {
                        let linea_trim = linea.trim();
                        if linea_trim.to_lowercase().starts_with("documentroot") {
                            let partes: Vec<&str> = linea_trim.split_whitespace().collect();
                            if partes.len() >= 2 {
                                let ruta_limpia = partes[1].replace("\"", "").replace("'", "");
                                sitios.push(SitioActivo {
                                    nombre: nombre_sitio.clone(),
                                    ruta: ruta_limpia,
                                });
                            }
                            break; 
                        }
                    }
                }
            }
        }
    }

    if sitios.is_empty() {
        println!("   {}", rust_i18n::t!("NO_ACTIVE_SITES"));
        return;
    }

    println!("=========================================");
    println!("      {}         ", rust_i18n::t!("ACTIVE_SITES_TITLE"));
    println!("=========================================");
    for (i, sitio) in sitios.iter().enumerate() {
        println!("{}) {} -> {}", i + 1, sitio.nombre, sitio.ruta);
    }
    println!("=========================================");

    let seleccion_sitio_raw = leer_linea(&rust_i18n::t!("PROMPT_SELECT_SITE", max = sitios.len()));
    let seleccion_sitio: usize = seleccion_sitio_raw.trim().parse().unwrap_or(0);

    if seleccion_sitio < 1 || seleccion_sitio > sitios.len() {
        println!("[X] {}", rust_i18n::t!("INVALID_OPTION"));
        return;
    }

    let sitio_elegido = &sitios[seleccion_sitio - 1];

    limpiar_pantalla();
    println!("=========================================");
    println!("     {}        ", rust_i18n::t!("CMS_GESTOR_TITLE"));
    println!(" {}: {}", rust_i18n::t!("SITE_DEST"), sitio_elegido.nombre);
    println!(" {}:  {}", rust_i18n::t!("PATH_DEST"), sitio_elegido.ruta);
    println!("=========================================");
    println!("1) WordPress (Última versión)");
    println!("2) Joomla (Versión 6.1.1)");
    println!("3) Drupal (Última versión)");
    println!("4) Moodle (Versión Estable 4.x)");
    println!("5) Añadir archivo info.php (Test de PHP)");
    println!("6) Vaciar sitio (Eliminar TODO el contenido)");
    println!("0) Cancelar");
    println!("=========================================");

    let cms_seleccionado = leer_linea("Selecciona una opción [0-6]: ");
    
    // Mapeamos la selección a nuestra estructura de paquetes
    let paquete_cms = match cms_seleccionado.trim() {
        "1" => Some(CmsPaquete { nombre: "WordPress", url: "https://wordpress.org/latest.tar.gz", archivo_cache: "wordpress_latest.tar.gz", es_zip: false }),
        "2" => Some(CmsPaquete { nombre: "Joomla", url: "https://downloads.joomla.org/cms/joomla6/6-1-1/Joomla_6.1.1-Stable-Full_Package.zip", archivo_cache: "joomla_6.1.1.zip", es_zip: true }),
        "3" => Some(CmsPaquete { nombre: "Drupal", url: "https://www.drupal.org/download-latest/tar.gz", archivo_cache: "drupal_latest.tar.gz", es_zip: false }),
        "4" => Some(CmsPaquete { nombre: "Moodle", url: "https://download.moodle.org/download.php/direct/stable404/moodle-latest-404.tgz", archivo_cache: "moodle_4.x.tgz", es_zip: false }),
        "5" => {
            let ruta_info = format!("{}/info.php", sitio_elegido.ruta);
            if evaluate!(fs::write(&ruta_info, "<?php phpinfo(); ?>\n"), true) {
                let _ = Command::new("chown").args(&["www-data:www-data", &ruta_info]).status();
                let dominio = sitio_elegido.nombre.replace(".conf", "");
                println!("[i] Puedes verlo en: http://{}/info.php", dominio);
            }
            return;
        }
        "6" => {
            println!("[!] ADVERTENCIA: Se eliminará TODO el contenido de {}", sitio_elegido.ruta);
            if leer_linea("¿Estás seguro? (s/n): ").trim().to_lowercase() == "s" {
                let cmd_limpiar = format!("find {} -mindepth 1 -delete", sitio_elegido.ruta);
                evaluate!(Command::new("bash").args(&["-c", &cmd_limpiar]).status(), true);
            }
            return;
        }
        _ => { println!("[X] {}", rust_i18n::t!("INVALID_OPTION")); return; }
    };

    // Si elegimos un CMS válido (Opciones 1 al 4)
    if let Some(cms) = paquete_cms {
        // 1. Configurar directorio de caché local en Documentos del usuario root/actual
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        let cache_dir = format!("{}/Documentos/cms_cache", home);
        let _ = fs::create_dir_all(&cache_dir); // Nos aseguramos nativamente de que la carpeta exista

        let ruta_completa_cache = format!("{}/{}", cache_dir, cms.archivo_cache);

        // 2. COMPROBACIÓN DE CACHÉ INTERNA
        if Path::new(&ruta_completa_cache).exists() {
            println!("{}", rust_i18n::t!("CACHE_FOUND", file = cms.archivo_cache));
        } else {
            // Si no existe, se descarga directamente a la caché
            println!("{}", rust_i18n::t!("DOWNLOADING_CMS", name = cms.nombre));
            if !evaluate!(Command::new("wget").args(&["-qO", &ruta_completa_cache, cms.url]).status(), true) {
                return;
            }
        }

        // 3. Validación de directorio de destino vacío
        let mut directorio_vacio = true;
        if let Ok(mut entradas) = fs::read_dir(&sitio_elegido.ruta) {
            if entradas.next().is_some() {
                directorio_vacio = false;
            }
        }

        if !directorio_vacio {
            println!("\n{}", rust_i18n::t!("DIRECTORY_NOT_EMPTY", dir = &sitio_elegido.ruta));
            println!("{}", rust_i18n::t!("WHAT_TO_DO"));
            println!("{}", rust_i18n::t!("CLEAN_BEFORE_INSTALL"));
            println!("{}", rust_i18n::t!("KEEP_EXISTING_FILES"));
            println!("{}", rust_i18n::t!("CANCEL_INSTALL"));
            
            let accion = leer_linea("Selecciona una opción [0-2]: ");
            if accion.trim() == "1" {
                let cmd_limpiar = format!("find {} -mindepth 1 -delete", sitio_elegido.ruta);
                let _ = Command::new("bash").args(&["-c", &cmd_limpiar]).status();
            } else if accion.trim() != "2" {
                println!("Operación cancelada.");
                return;
            }
        }

        // 4. EXTRACCIÓN DIRECTA DESDE LA CACHÉ
        println!("{}", rust_i18n::t!("EXTRACTING_CMS", name = cms.nombre));
        
        let estatus_extraccion = if cms.es_zip {
            // Asegurar que unzip esté en el sistema silenciosamente
            let _ = Command::new("apt-get").args(&["install", "unzip", "-y"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
            Command::new("unzip").args(&["-q", "-o", &ruta_completa_cache, "-d", &sitio_elegido.ruta]).status()
        } else {
            // Usa --strip-components=1 para volcar todo omitiendo la carpeta raíz interna del tar.gz (ej: wordpress/)
            Command::new("tar").args(&["-xzf", &ruta_completa_cache, "--strip-components=1", "-C", &sitio_elegido.ruta]).status()
        };

        if evaluate!(estatus_extraccion, true) {
            // 5. Permisos Finales
            println!("{}", rust_i18n::t!("APPLYING_PERMISSIONS"));
            let _ = Command::new("chown").args(&["-R", "www-data:www-data", &sitio_elegido.ruta]).status();

            println!("\n{} {}", OK, rust_i18n::t!("INSTALL_SUCCESS"));
            let dominio_limpio = sitio_elegido.nombre.replace(".conf", "");
            println!("{}", rust_i18n::t!("OPEN_BROWSER_TIP", site = dominio_limpio));
        }
    }
}

// 1. Estática para compilar las Regex una sola vez en la vida del binario
struct SitioRegex {
    docroot: Regex,
    dir: Regex,
    php: Regex,
    servername: Regex,
}

fn obtener_regex_sitio() -> &'static SitioRegex {
    static REGEX_LOCK: OnceLock<SitioRegex> = OnceLock::new();
    REGEX_LOCK.get_or_init(|| SitioRegex {
        docroot: Regex::new(r"DocumentRoot\s+(?P<path>[\S]+)").unwrap(),
        dir: Regex::new(r"<Directory\s+(?P<path>[\S]+)>").unwrap(),
        php: Regex::new(r"php(?P<ver>\d+\.\d+)-fpm\.sock").unwrap(),
        servername: Regex::new(r"ServerName\s+(?P<name>[\S]+)").unwrap(),
    })
}

pub fn editar_sitio_apache() {
    let sitios_path = "/etc/apache2/sites-available";
    
    // 2. Listar archivos de forma segura sin .expect()
    let entries = match fs::read_dir(sitios_path) {
        Ok(e) => e,
        Err(_) => {
            println!("[X] {}", rust_i18n::t!("READ_DIR_ERROR"));
            return;
        }
    };

    let archivos: Vec<_> = entries
        .flatten()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "conf"))
        .collect();

    if archivos.is_empty() { 
        println!("{}", rust_i18n::t!("NO_CONFIG_FILES_FOUND", path = sitios_path)); 
        return; 
    }

    // 3. Menú de selección
    println!("{}", rust_i18n::t!("SELECT_SITE_TO_EDIT"));
    for (i, entry) in archivos.iter().enumerate() {
        println!("{}) {}", i + 1, entry.file_name().to_string_lossy());
    }

    let sel = leer_linea(&rust_i18n::t!("PROMPT_OPTION")).parse::<usize>().unwrap_or(0);
    if sel == 0 || sel > archivos.len() { 
        println!("[X] {}", rust_i18n::t!("INVALID_OPTION")); 
        return; 
    }
    
    let path = archivos[sel - 1].path();
    let nombre_archivo = path.file_name().unwrap_or_default().to_string_lossy();
    
    // 4. Crear backup evaluado (Si falla, se detiene por seguridad)
    let backup_path = format!("{}.bak", path.display());
    if !evaluate!(fs::copy(&path, &backup_path), true) {
        return;
    }
    
    // Leer contenido de forma segura
    let mut contenido = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            println!("[X] Error al leer el archivo.");
            return;
        }
    };

    // Obtener las Regex optimizadas de la memoria estática
    let re = obtener_regex_sitio();

    println!("{}", rust_i18n::t!("EDITING_SITE", name = nombre_archivo));
    
    // 5. Modificaciones interactivas
    let nuevo_root = leer_linea(&rust_i18n::t!("PROMPT_NEW_DOCROOT"));
    if !nuevo_root.is_empty() {
        contenido = re.docroot.replace(&contenido, format!("DocumentRoot {}", nuevo_root)).to_string();
        contenido = re.dir.replace(&contenido, format!("<Directory {}>", nuevo_root)).to_string();
    }

    versiones_instaladas_php(); // Tu función existente que lista las versiones de PHP en el sistema

    let nueva_ver = leer_linea(&rust_i18n::t!("PROMPT_NEW_PHP_VER"));
    if !nueva_ver.is_empty() {
        contenido = re.php.replace(&contenido, format!("php{}-fpm.sock", nueva_ver)).to_string();
    }

    let nuevo_name = leer_linea(&rust_i18n::t!("PROMPT_NEW_SERVERNAME"));
    if !nuevo_name.is_empty() {
        contenido = re.servername.replace(&contenido, format!("ServerName {}", nuevo_name)).to_string();
    }

    // 6. Guardar cambios usando evaluate!
    if !evaluate!(fs::write(&path, contenido), true) {
        return;
    }
    println!("{}", rust_i18n::t!("FILE_UPDATED_BACKUP", backup = &backup_path));
    
    // 7. Aplicar y validar cambios en Apache con la macro
    println!("\n{}", rust_i18n::t!("CONFIG_TEST_RUNNING"));
    
    let test_status = Command::new("apache2ctl")
        .arg("configtest")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if test_status.is_ok() && test_status.unwrap().success() {
        // Si el test pasa, reiniciamos el servicio de forma limpia
        let restart_status = Command::new("systemctl").args(&["restart", "apache2"]).status();
        if evaluate!(restart_status, true) {
            println!("{}", rust_i18n::t!("CONFIG_VALID_RESTARTED"));
        }
    } else {
        // Si el test falla, pintamos el error y le recordamos dónde está su .bak
        println!("{}", rust_i18n::t!("CONFIG_ERROR_BACKUP_TIP", backup = backup_path));
    }
}
