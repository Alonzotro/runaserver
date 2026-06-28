use rust_i18n::t;
use std::io::{self, Read, Write};
use local_ip_address::local_ip;

mod php;
mod apache;
mod mysql;
mod servicios;
mod public;

use crate::php::{install_php, desinstalacion_php, versiones_instaladas_php, modulos_php, cambiar_php, php_activo};

use crate::apache::{version_apache, config_apache, reiniciar_apache, add_site_apache, instalar_cms, disable_php_apache, enable_php_apache, editar_sitio_apache};

use crate::mysql::{instalar_phpmyadmin, configurar_mysql_seguro};

use crate::servicios::{upgrade_server, passwd_root, install_needed_software, drivers_needed, permisos, reboot, auto_start,select_language};

use crate::public::{Settings, error_log, clear_screen, print_header, read_in, line, OK, INFO, WARNING, ERROR_YOU, ERROR_PC, ARROW, LOG_ERRORES};

rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    // Validamos si es root antes de poder entrar al menu principal
    if !is_root() {
        eprintln!("{}: {}", ERROR_YOU, rust_i18n::t!("MUST_BE_ROOT"));
        std::process::exit(1);
    }

    //Limpiamos la pantalla para que no se vea los comandos anteriores
    clear_screen();
    //load();
    //Obtenemos la IP para futuros comandos
    let ip = get_ip();

    loop {
        //Imprimimos el menu principal
        menu();
        //Esperamos una opcion
        let opcion = read_in(&format!("{} [1-10] {}: ", rust_i18n::t!("SELECT_OPTION"), rust_i18n::t!("ZERO2BACK")));
        match opcion.trim() {
            "1" => {
                clear_screen();
                upgrade_server();
                pause();
                clear_screen();
            }
            "2" => {
                clear_screen();
                passwd_root();
                pause();
                clear_screen();
            }
            "3" => {
                clear_screen();
                install_needed_software();
                pause();
                clear_screen();
            }
            "4" => {
                clear_screen();
                drivers_needed();
                pause();
                clear_screen();
            }
            "5" => {
                clear_screen();
                permisos();
                pause();
                clear_screen();
            }
            "6" => {
                clear_screen();
                instalar_cms();
                pause();
                clear_screen();
            }
            "7" => {
                clear_screen();
                auto_start();
                pause();
                clear_screen();
            }
            "8" | "PHP" | "php" => {
                // Iniciamos un bucle exclusivo para el menú PHP
                loop {
                    clear_screen();
                    menu_php();
                    
                    // Te sugiero agregar una opción (ej. 0) para regresar al menú principal
                    let opcion_php = read_in(&format!("{} [1-6] {}: ", rust_i18n::t!("SELECT_OPTION"), rust_i18n::t!("ZERO2BACK")));
                    
                    match opcion_php.trim() {
                        "1" => {
                            clear_screen();
                            versiones_instaladas_php();
                            
                            pause();
                        }
                        "2" => {
                            clear_screen();
                            install_php();
                            pause();
                        }
                        "3" => {
                            clear_screen();
                            desinstalacion_php();
                            pause();
                        }
                        "4" => {
                            clear_screen();
                            modulos_php();
                            pause();
                        }
                        "5" => {
                            clear_screen();
                            php_activo();
                            pause();
                        }
                        "6" => {
                            clear_screen();
                            cambiar_php();
                            pause();
                        }
                        "" | "0" | "exit"  => {
                            break; 
                        }
                        _ => {
                            clear_screen();
                        }
                    }
                }
                clear_screen();
            }
            "9" | "Apache" | "APACHE" | "apache" => {
                // Iniciamos el sub-bucle exclusivo para el menú de Apache
                loop {
                    clear_screen();
                    let _ = version_apache(); // Muestra el estado/versión
                    menu_apache();            // Muestra las opciones
                    
                    // Actualizamos el rango a [1-4] y añadimos la opción 0
                    let opcion_apache = read_in(&format!("{} [1-4] {}: ", rust_i18n::t!("SELECT_OPTION"), rust_i18n::t!("ZERO2BACK"))); 
                    
                    match opcion_apache.trim() {
                        "1" => {
                            clear_screen();
                            reiniciar_apache();
                            
                            pause();
                        }
                        "2" => {
                            clear_screen();
                            config_apache();
                            pause();
                        }
                        "3" => {
                            clear_screen();
                            add_site_apache(ip.as_deref().unwrap_or("127.0.0.1"));
                            pause();
                        }
                        "4" => {
                            clear_screen();
                            editar_sitio_apache();
                            pause();
                        }
                        "5" => {
                            clear_screen();
                            enable_php_apache(); 
                            pause();
                        }
                        "6" => {
                            clear_screen();
                            disable_php_apache(); 
                            pause();
                        }
                        "" | "0" | "exit" => {
                            break;
                        }
                        _ => {
                            clear_screen();
                        }
                    }
                }
                // Limpiamos la pantalla justo antes de devolver el control al menú principal
                clear_screen();
            }
            "10" | "MySQL" | "mysql" | "MYSQL" => {
                // Iniciamos el sub-bucle exclusivo para el menú de Apache
                loop {
                    clear_screen();
                    menu_mysql();            // Muestra las opciones
                    
                    let opcion_mysql = read_in(&format!("{} [1-2] {}: ", rust_i18n::t!("SELECT_OPTION"), rust_i18n::t!("ZERO2BACK")));
                    
                    match opcion_mysql.trim() {
                        "1" => {
                            clear_screen();
                            configurar_mysql_seguro();
                            pause();
                        }
                        "2" => {
                            clear_screen();
                            instalar_phpmyadmin();
                            pause();
                        }
                        "" | "0" | "exit" => {
                            // Rompe SOLO este bucle interno de Apache y vuelve al menú principal
                            break;
                        }
                        _ => {
                            clear_screen();                            

                        }
                    }
                }
                // Limpiamos la pantalla justo antes de devolver el control al menú principal
                clear_screen();
            }
            "11" => {
                clear_screen();
                know_ip();
                pause();
                clear_screen();
            }
            "12" => {
                clear_screen();
                select_language();
                pause();
                clear_screen();
            }
            "0" | "exit" => {
                clear_screen();
                std::process::exit(0);
            }
            "?" | "reboot" | "00" => {
                clear_screen();
                reboot();
            }
            _ => {
            clear_screen();
            }
        }
    }
}

// ==========================================
// FUNCIONES AUXILIARES Y DE CONTROL
// ==========================================

fn pause() {
    println!("");
    let _ = read_in!(&rust_i18n::t!("PAUSE"));
}

fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn get_ip() -> Option<String> {
    match local_ip() {
        Ok(ip) => Some(ip.to_string()),
        Err(_) => None,
    }
}

fn get_public_ip() -> Option<String> {
    match ureq::get("https://api.ipify.org").call() {
        Ok(respuesta) => {
            // En ureq 3.3.0 no lleva argumentos y te da el String limpio de golpe
            match respuesta.into_body().read_to_string() {
                Ok(cuerpo_string) => Some(cuerpo_string.trim().to_string()),
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

fn know_ip() {
    print_header("MIS IPs");
    println!("{} Obteniendo información de red...", INFO);

    if let Some(ip_lan) = get_ip() {
        println!("{} IP Local:  {}",OK, ip_lan);
    } else {
        println!("{} IP Local:  No encontrada",ERROR_YOU);
    }

    if let Some(ip_wan) = get_public_ip() {
        println!("{} IP Pública: {}",OK ,ip_wan);
    } else {
        println!("{} IP Pública: Desconectado / Error", ERROR_PC);
    }
}

// ==========================================
// VISTAS / MENÚS
// ==========================================

fn menu() {
    print_header(&rust_i18n::t!("MAIN_MENU"));
    println!("[1] {}", rust_i18n::t!("UPDATE_SERVER"));
    println!("[2] {}", rust_i18n::t!("CHANGE_ROOT_PSWD"));
    println!("[3] {}", rust_i18n::t!("INSTALL_NEEDED"));
    println!("[4] {}", rust_i18n::t!("INSTALL_DRIVERS"));
    line();
    println!("[5] {}", rust_i18n::t!("SET_WWW"));
    println!("[6] {}", rust_i18n::t!("INSTALL_CMS"));
    println!("[7] {}", rust_i18n::t!("ADD2TERMINAL"));
    line();
    println!("[8] {} {}", rust_i18n::t!("PHP_CONFIG"), ARROW);
    println!("[9] {} {}", rust_i18n::t!("APACHE_CONFIG"), ARROW);
    println!("[10] {} {}", rust_i18n::t!("MYSQL_CONFIG"), ARROW);
    line();
    println!("[11] Conocer mis IP");
    println!("[12] {}", rust_i18n::t!("CHANGE_LENGUAGE"));
    println!("[0] {}", rust_i18n::t!("EXIT"));
    println!("[00] {}", rust_i18n::t!("REBOOT"));
    line();
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
