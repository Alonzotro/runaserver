// ==========================================
// SERVICIOS PRINCIPALES
// ==========================================
use crate::{evaluate, read_in, command};
use crate::public::{error_log, clear_screen, print_header, line, command, Evaluable, OK, INFO, WARNING, ERROR_YOU, ERROR_PC, ARROW, LOG_ERRORES};
use std::fs::{self, OpenOptions};
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
//use rust_i18n::t;


fn update() {
    println!("{} {}",INFO, rust_i18n::t!("UPDATING"));
    let status = command!("apt-get", &["update"], true);
    if !status {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_UPDATING"));
    }
}

fn upgrade() {
    println!("{} {}",INFO, rust_i18n::t!("UPGRADING"));
    let status = command!("apt-get", &["upgrade"], true);
    if !status {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_UPGRADING"));
    }
}

pub fn select_language() {
    print_header(&rust_i18n::t!("SELECT_LANGUAGE"));
    println!("[1] English");
    println!("[2] Español");
    line();
    let _ = io::stdout().flush();

    let opcion = read_in!("{} [1-2]: ", rust_i18n::t!("SELECT_OPTION"));
    match opcion.trim() {
        "1" | "" => {
            // Si elige 1, o escribe cualquier otra cosa, usamos inglés por defecto
            rust_i18n::set_locale("en");
            println!("{} {}",OK,rust_i18n::t!("LANGUAGE_SELECTED"));
        }
        "2" => {
            rust_i18n::set_locale("es");
            println!("{} {}",OK,rust_i18n::t!("LANGUAGE_SELECTED"));
        }
        _ => {
            println!("{} {}",ERROR_YOU,rust_i18n::t!("BAD_SELECTED"));
        }
    }
}

pub fn upgrade_server() {

    update();

    upgrade();

    println!("{} {}",INFO, rust_i18n::t!("DELETE_PKG_OBS"));
    let status = command!("apt-get", &["autoremove", "-y"], true);
    if !status {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_DELETE_PKG_OBS"));
    }

    println!("{} {}",INFO, rust_i18n::t!("clear_CACHE"));
    let status = command!("apt-get", &["autoclean", "-y"], true);
    if !status {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_clear_CACHE"));
    }
}

pub fn passwd_root() {
    println!("{} {}", INFO, rust_i18n::t!("CHANGE_PASSWD"));
    command!("passwd", &["root"], true, Stdio::inherit());
}

pub fn install_needed_software() {
    update();

    //Instala los paquetes necesarios
    println!("{} {}", INFO, rust_i18n::t!("INSTALL_NECESSARY"));
    let status = command!("apt-get", &["install", "-y", "software-properties-common", "wget", "tar", "libncurses6", "libnuma1", "openssl", "net-tools", "ufw", "ca-certificates", "apt-transport-https"], true);
    if !status {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_INSTALL_NECESSARY"));
    }

    println!("[1/2] {} {}", INFO, rust_i18n::t!("ADD_REPOSITORY_PHP"));
    let status = command!("add-apt-repository", &["ppa:ondrej/php", "-y"], true);
    if !status {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_ADD_REPOSITORY_PHP"));
    }

    println!("[2/2] {} {}", INFO, rust_i18n::t!("ADD_REPOSITORY_APACHE"));
    let status = command!("add-apt-repository", &["ppa:ondrej/apache2", "-y"], true);
    if !status {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_ADD_REPOSITORY_APACHE"));
    }

    update();

    //Instala programas necesarios
    println!("{} {} (Apache, MySQL, etc)...",INFO, rust_i18n::t!("INSTALL_ESSENTIAL"));;
    let status = command!("apt-get", &["install", "-y", "apache2", "apache2-suexec-pristine", "apache2-suexec-custom", "libapache2-mod-fcgid", "mysql-server", "mysql-client"], true);
    if !status {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_INSTALL_NECESSARY"));
    }

    upgrade();
}

pub fn drivers_needed() {
    
    // 1. Mostramos el menú
    print_header(&rust_i18n::t!("MACHINE"));
    println!("[1] Virtualbox");
    println!("[2] VMWare");
    println!("[3] {}", rust_i18n::t!("NO_VM"));
    line();
    let vm = read_in!("{} [1-3]: ", rust_i18n::t!("SELECT_OPTION"));
    
    let opcion = vm.trim();

    // 2. Ejecutamos el match y recolectamos el status de forma limpia
    match opcion {
        "1" => {
            clear_screen();
            println!("{}", rust_i18n::t!("INSTALLING_DRIVERS"));
            update();

            let status = command!("apt-get", &["install", "-y", "build-essential", "dkms", "virtualbox-guest-x11", "virtualbox-guest-utils"], true);
            if !status {
                println!("{} Error al instalar los drivers necesarios", ERROR_PC);
            }
            println!("{}", rust_i18n::t!("RECOMMENDED_VIRTUALBOX"));
        }
        "2" => {
            clear_screen();
            println!("{}", rust_i18n::t!("INSTALLING_DRIVERS"));
            update();

            let status = command!("apt-get", &["install", "-y", "open-vm-tools", "open-vm-tools-desktop"], true);
            if !status {
                println!("{} Error al instalar los drivers necesarios", ERROR_PC);
            }
        }
        "3" => {
            clear_screen();
            println!("{}", rust_i18n::t!("INSTALLING_DRIVERS"));
            update();

            let status = command!("apt-get", &["install", "-y", "linux-headers-generic", "firmware-linux-free"], true);
            if !status {
                println!("{} Error al instalar los drivers necesarios", ERROR_PC);
            }
        }
        _ => { //unreachable!(),
            println!("{} {}",ERROR_YOU,rust_i18n::t!("BAD_SELECTED"));
        }
    };

}

pub fn permisos() {
    println!("{} {}",INFO,rust_i18n::t!("CHANGING_PERMISSION"));
    command!("chown", &["-R", "www-data:www-data", "/var/www"], true, Stdio::inherit());
}

pub fn reboot() {
    print_header(&rust_i18n::t!("REBOOT_SYSTEM"));

    println!("{} {} {}",WARNING,rust_i18n::t!("CAUTION"),rust_i18n::t!("NOTICE"));
    line();

    let confirmacion = read_in!(&rust_i18n::t!("SURE_ACCION"));

    match confirmacion.trim().to_lowercase().as_str() {
        "s" | "si" | "y" | "yes" => {
            println!("{}",rust_i18n::t!("REBOOTING"));
            
            // Ejecutamos el comando de reinicio nativo de Linux
            let status = Command::new("reboot").status();
            if !evaluate!(status, true) {
                std::process::exit(0);
            }
        }
        _ => {
            clear_screen();
        }
    }
}


pub fn auto_start() {
    print_header(&rust_i18n::t!("CONFIG_AUTO_START_TITLE"));

    // 1. Obtener la ruta exacta del ejecutable actual
    println!("{}", rust_i18n::t!("GETTING_CURRENT_PATH"));
    let Some(exe_path) = evaluate!(env::current_exe(), false) else { return; };

    let target_bin = "/usr/local/bin/runaserver-bin";
    let target_wrapper = "/usr/local/bin/runaserver";

    // 2. DETECCIÓN Y BORRADO FORZADO:
    if Path::new(target_bin).exists() {
        println!("{} {}", WARNING, rust_i18n::t!("PREVIOUS_VERSION_DETECTED", path = target_bin));
        println!("{} {}", WARNING, rust_i18n::t!("DELETING_OLD_FILE"));
        
        if !evaluate!(fs::remove_file(target_bin), false) {
            println!("{} {}", ERROR_PC, rust_i18n::t!("COULD_NOT_DELETE_OLD"));
            println!("{}", rust_i18n::t!("TRYING_TO_CONTINUE"));
        }
    }

    // 3. COPIA LIMPIA DEL BINARIO REAL:
    println!("{}", rust_i18n::t!("COPYING_NEW_VERSION"));
    if !evaluate!(fs::copy(&exe_path, target_bin), false) {
        return;
    }

    println!("{}", rust_i18n::t!("SETTING_BIN_PERMISSIONS"));
    evaluate!(Command::new("chmod").args(&["+x", target_bin]).status(), false);

    // 4. CREACIÓN DEL WRAPPER SCRIPT (Magia de pkexec):
    println!("{}", rust_i18n::t!("CONFIGURING_PRIVILEGE_ELEVATION"));
    
    let wrapper_content = format!(
        "#!/bin/bash\n\
        if [ \"$EUID\" -eq 0 ]; then\n\
        \t{} \"$@\"\n\
        else\n\
        \tpkexec {} \"$@\"\n\
        fi\n",
        target_bin, target_bin
    );

    if !evaluate!(fs::write(target_wrapper, wrapper_content), true) {
        return;
    }

    println!("{}", rust_i18n::t!("SETTING_WRAPPER_PERMISSIONS"));
    evaluate!(Command::new("chmod").args(&["+x", target_wrapper]).status(), false);

    println!("{} {}", OK, rust_i18n::t!("UPDATE_COMPLETE_PKEXEC"));

    // 5. VERIFICACIÓN DEL BASHRC:
    let bashrc_path = "/root/.bashrc";
    
    println!("{}", rust_i18n::t!("VERIFYING_BASHRC"));
    if let Ok(contenido) = fs::read_to_string(bashrc_path) {
        if contenido.contains(target_wrapper) {
            println!("{} {}", OK, rust_i18n::t!("BASHRC_ALREADY_LINKED"));
            return;
        }
    }

    // Si no existía en el .bashrc, añadimos la línea al final
    println!("{}", rust_i18n::t!("ADDING_TO_BASHRC"));
    match OpenOptions::new().append(true).open(bashrc_path) {
        Ok(mut file) => {
            let linea_arranque = format!("\n# Iniciar gestor de servidor automáticamente\n{}\n", target_wrapper);
            if let Err(e) = file.write_all(linea_arranque.as_bytes()) {
                println!("[X] {}", rust_i18n::t!("BASHRC_WRITE_ERROR", error = e));
            } else {
                println!("{} {}", OK, rust_i18n::t!("INITIAL_BOOT_CONFIG_SUCCESS"));
            }
        }
        Err(e) => {
            println!("[X] {}", rust_i18n::t!("BASHRC_OPEN_ERROR", path = bashrc_path, error = e));
        }
    }
}