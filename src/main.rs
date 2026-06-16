mod php;
mod apache;
mod mysql;
mod servicios;
// Traes las funciones específicas que vas a usar en este archivo


use crate::php::{install_php, desinstalacion_php, versiones_instaladas_php, modulos_php, cambiar_php, php_activo};

use crate::apache::{version_apache, config_apache, reiniciar_apache, add_site_apache, instalar_cms, disable_php_apache, enable_php_apache, editar_sitio_apache};

use crate::mysql::{instalar_phpmyadmin, configurar_mysql_seguro};

use crate::servicios::{upgrade_server, passwd_root, install_needed_software, drivers_needed, permisos, reboot, auto_start,select_language};

use rust_i18n::t;
use std::fs::{OpenOptions};
use std::io::{self, Write};
use std::process::{Command, Stdio, ExitStatus};
use std::path::PathBuf;
use std::process::Output;

const OK: &str = "[OK]";
const WARNING: &str = "[!]";
const ERROR_YOU: &str = "[X]";
const ERROR_PC: &str = "[ERROR]";
const ARROW: &str = "-->";
const LOG_ERRORES: &str = "/var/log/errores_mantenimiento.log";

rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    // Validamos si es root antes de poder entrar al menu principal
    if !is_root() {
        eprintln!("{}: {}", ERROR_YOU, rust_i18n::t!("MUST_BE_ROOT"));
        std::process::exit(1);
    }

    //Limpiamos la pantalla para que no se vea los comandos anteriores
    limpiar_pantalla();

    //Obtenemos la IP para futuros comandos
    let ip = get_ip();

    loop {
        //Imprimimos el menu principal
        menu();
        //Esperamos una opcion
        let opcion = leer_linea(&format!("{} [1-10] {}: ", rust_i18n::t!("SELECT_OPTION"), rust_i18n::t!("ZERO2BACK")));
        match opcion.trim() {
            "1" => {
                limpiar_pantalla();
                upgrade_server();
                pausa();
                limpiar_pantalla();
            }
            "2" => {
                limpiar_pantalla();
                passwd_root();
                pausa();
                limpiar_pantalla();
            }
            "3" => {
                limpiar_pantalla();
                install_needed_software();
                pausa();
                limpiar_pantalla();
            }
            "4" => {
                limpiar_pantalla();
                drivers_needed();
                pausa();
                limpiar_pantalla();
            }
            "5" => {
                limpiar_pantalla();
                permisos();
                pausa();
                limpiar_pantalla();
            }
            "6" => {
                limpiar_pantalla();
                instalar_cms();
                pausa();
                limpiar_pantalla();
            }
            "7" => {
                limpiar_pantalla();
                auto_start();
                pausa();
                limpiar_pantalla();
            }
            "8" | "PHP" | "php" => {
                // Iniciamos un bucle exclusivo para el menú PHP
                loop {
                    limpiar_pantalla();
                    menu_php();
                    
                    // Te sugiero agregar una opción (ej. 0) para regresar al menú principal
                    let opcion_php = leer_linea(&format!("{} [1-6] {}: ", rust_i18n::t!("SELECT_OPTION"), rust_i18n::t!("ZERO2BACK")));
                    
                    match opcion_php.trim() {
                        "1" => {
                            limpiar_pantalla();
                            versiones_instaladas_php();
                            
                            pausa();
                        }
                        "2" => {
                            limpiar_pantalla();
                            install_php();
                            pausa();
                        }
                        "3" => {
                            limpiar_pantalla();
                            desinstalacion_php();
                            pausa();
                        }
                        "4" => {
                            limpiar_pantalla();
                            modulos_php();
                            pausa();
                        }
                        "5" => {
                            limpiar_pantalla();
                            php_activo();
                            pausa();
                        }
                        "6" => {
                            limpiar_pantalla();
                            cambiar_php();
                            pausa();
                        }
                        "" | "0" | "exit"  => {
                            break; 
                        }
                        _ => {
                            limpiar_pantalla();
                        }
                    }
                }
                limpiar_pantalla();
            }
            "9" | "Apache" | "APACHE" | "apache" => {
                // Iniciamos el sub-bucle exclusivo para el menú de Apache
                loop {
                    limpiar_pantalla();
                    let _ = version_apache(); // Muestra el estado/versión
                    menu_apache();            // Muestra las opciones
                    
                    // Actualizamos el rango a [1-4] y añadimos la opción 0
                    let opcion_apache = leer_linea(&format!("{} [1-4] {}: ", rust_i18n::t!("SELECT_OPTION"), rust_i18n::t!("ZERO2BACK"))); 
                    
                    match opcion_apache.trim() {
                        "1" => {
                            limpiar_pantalla();
                            reiniciar_apache();
                            
                            pausa();
                        }
                        "2" => {
                            limpiar_pantalla();
                            config_apache();
                            pausa();
                        }
                        "3" => {
                            limpiar_pantalla();
                            add_site_apache(&ip);
                            pausa();
                        }
                        "4" => {
                            limpiar_pantalla();
                            editar_sitio_apache();
                            pausa();
                        }
                        "5" => {
                            limpiar_pantalla();
                            enable_php_apache(); 
                            pausa();
                        }
                        "6" => {
                            limpiar_pantalla();
                            disable_php_apache(); 
                            pausa();
                        }
                        "" | "0" | "exit" => {
                            break;
                        }
                        _ => {
                            limpiar_pantalla();
                        }
                    }
                }
                // Limpiamos la pantalla justo antes de devolver el control al menú principal
                limpiar_pantalla();
            }
            "10" | "MySQL" | "mysql" | "MYSQL" => {
                // Iniciamos el sub-bucle exclusivo para el menú de Apache
                loop {
                    limpiar_pantalla();
                    menu_mysql();            // Muestra las opciones
                    
                    let opcion_mysql = leer_linea(&format!("{} [1-2] {}: ", rust_i18n::t!("SELECT_OPTION"), rust_i18n::t!("ZERO2BACK")));
                    
                    match opcion_mysql.trim() {
                        "1" => {
                            limpiar_pantalla();
                            configurar_mysql_seguro();
                            pausa();
                        }
                        "2" => {
                            limpiar_pantalla();
                            instalar_phpmyadmin();
                            pausa();
                        }
                        "" | "0" | "exit" => {
                            // Rompe SOLO este bucle interno de Apache y vuelve al menú principal
                            break;
                        }
                        _ => {
                            limpiar_pantalla();                            

                        }
                    }
                }
                // Limpiamos la pantalla justo antes de devolver el control al menú principal
                limpiar_pantalla();
            }
            "11" => {
                limpiar_pantalla();
                select_language();
                pausa();
                limpiar_pantalla();
            }
            "0" | "exit" => {
                limpiar_pantalla();
                std::process::exit(0);
            }
            "?" | "reboot" | "00" => {
                limpiar_pantalla();
                reboot();
            }
            _ => {
            limpiar_pantalla();
            }
        }
    }
}

// ==========================================
// FUNCIONES AUXILIARES Y DE CONTROL
// ==========================================

fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn get_ip() -> String {
    // Redirige errores al log
    let log_file = registrar_log_error();
    if let Ok(output) = Command::new("hostname").arg("-I").stderr(log_file).output() {
        let ips = String::from_utf8_lossy(&output.stdout);
        if let Some(primera_ip) = ips.split_whitespace().next() {
            return primera_ip.to_string();
        }
    }
    "127.0.0.1".to_string()
}

pub fn registrar_log_error() -> Stdio {
    //Abre el achivo de log, anade la informacion hasta abajo, lo crea si es necesario y verifica que todo esta bien
    if let Ok(file) = OpenOptions::new().create(true).append(true).open(LOG_ERRORES) {
        //Crea una tuveria en log files
        Stdio::from(file)
    } else {
        Stdio::null()
    }
}

fn limpiar_pantalla() {
    // \x1B[2J limpia la pantalla visible
    // \x1B[3J borra el historial de desplazamiento (scrollback buffer)
    // \x1B[1;1H reposiciona el cursor en la esquina superior izquierda
    print!("{}[2J{}[3J{}[1;1H", 27 as char, 27 as char, 27 as char);
    let _ = io::stdout().flush();
}

fn pausa() {
    println!("");
    let _ = leer_linea!(&rust_i18n::t!("PAUSE"));
}

fn leer_linea(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input
}

#[macro_export]
macro_rules! leer_linea {
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

// ==========================================
// VISTAS / MENÚS
// ==========================================

fn menu() {
    println!("=========================================");
    println!("      {}       ", rust_i18n::t!("MAIN_MENU"));
    println!("=========================================");
    println!("[1] {}", rust_i18n::t!("UPDATE_SERVER"));
    println!("[2] {}", rust_i18n::t!("CHANGE_ROOT_PSWD"));
    println!("[3] {}", rust_i18n::t!("INSTALL_NEEDED"));
    println!("[4] {}", rust_i18n::t!("INSTALL_DRIVERS"));
    println!("=========================================");
    println!("[5] {}", rust_i18n::t!("SET_WWW"));
    println!("[6] {}", rust_i18n::t!("INSTALL_CMS"));
    println!("[7] {}", rust_i18n::t!("ADD2TERMINAL"));
    println!("=========================================");
    println!("[8] {} {}", rust_i18n::t!("PHP_CONFIG"), ARROW);
    println!("[9] {} {}", rust_i18n::t!("APACHE_CONFIG"), ARROW);
    println!("[10] {} {}", rust_i18n::t!("MYSQL_CONFIG"), ARROW);
    println!("=========================================");
    println!("[11] {}", rust_i18n::t!("CHANGE_LENGUAGE"));
    println!("[0] {}", rust_i18n::t!("EXIT"));
    println!("[00] {}", rust_i18n::t!("REBOOT"));
    println!("=========================================");
}


fn menu_php() {
    println!("=========================================");
    println!("     {}       ", rust_i18n::t!("PHP_MENU"));
    println!("=========================================");
    println!("[1] {}", rust_i18n::t!("SHOW_INSTALLED_PHP"));
    println!("[2] {}", rust_i18n::t!("INSTALLED_PHP"));
    println!("[3] {}", rust_i18n::t!("UNINSTALL_PHP"));
    println!("[4] {}", rust_i18n::t!("MANAGE_MODULES_PHP"));
    println!("[5] {}", rust_i18n::t!("SHOW_ACTIVE_PHP"));
    println!("[6] {}", rust_i18n::t!("CHANGE_PHP_CLI"));
    println!("[0] {}", rust_i18n::t!("GOBACK"));
    println!("=========================================");
}


fn menu_apache() {
    println!("=========================================");
    println!("    {}    ", rust_i18n::t!("APACHE_MENU"));
    println!("=========================================");
    println!("[1] {}", rust_i18n::t!("RESTART_APACHE"));
    println!("[2] {}", rust_i18n::t!("CONFIG_APACHE"));
    println!("[3] {}", rust_i18n::t!("ADD_SITE"));
    println!("[4] {}", rust_i18n::t!("CONFIG_SITE"));
    println!("[5] {}", rust_i18n::t!("ENABLE_PHP_APACHE"));
    println!("[6] {}", rust_i18n::t!("DISABLE_PHP_APACHE"));
    println!("[0] {}", rust_i18n::t!("GOBACK"));
    println!("=========================================");
}


fn menu_mysql() {
    println!("=========================================");
    println!("    {}    ", rust_i18n::t!("MYSQL_MENU"));
    println!("=========================================");
    println!("[1] {}", rust_i18n::t!("CONFIG_MYSQL"));
    println!("[2] {}", rust_i18n::t!("INSTALL_PHPMYADMIN"));
    println!("[0] {}", rust_i18n::t!("GOBACK"));
    println!("=========================================");
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
                if show { println!("{} {} {} {}", WARNING, rust_i18n::t!("RESULT"), rust_i18n::t!("CODE"), status); }
                false
            }
            Err(e) => {
                if show { println!("{} {} {} {}", ERROR_PC, rust_i18n::t!("RESULT_ERROR"), rust_i18n::t!("CODE"), e); }
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
                if show { println!("{} {} {} {}", ERROR_PC, rust_i18n::t!("RESULT_ERROR"), rust_i18n::t!("CODE"), e); }
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
                if show { println!("{} {} {} {}", ERROR_PC, rust_i18n::t!("RESULT_ERROR"), rust_i18n::t!("CODE"), e); }
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
                // Como es un error crítico, ignoramos el 'show' y lo pintamos a fuerza
                println!("[X] No se pudo determinar la ruta del ejecutable: {}", e);
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
                if show { println!("{} El comando falló con código: {}", WARNING, out.status); }
                None
            }
            Err(e) => {
                if show { println!("{} Error al ejecutar comando: {}", ERROR_PC, e); }
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



