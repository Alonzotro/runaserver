// ==========================================
// SERVICIOS PRINCIPALES
// ==========================================
use crate::{evaluate, read_in};
use crate::public::{error_log, clear_screen, print_header, Evaluable, OK, INFO, WARNING, ERROR_YOU, ERROR_PC, ARROW, LOG_ERRORES};
use std::fs::{self, OpenOptions};
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
//use rust_i18n::t;


fn update() {
    println!("{} {}",WARNING, rust_i18n::t!("UPDATING"));
    let status = Command::new("apt-get")
        .args(&["update", "-y"])
        .stdout(Stdio::null())
        .stderr(error_log())
        .status();
    if !evaluate!(status, true) {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_UPDATING"));
    }
}

fn upgrade() {
    println!("{} {}",WARNING, rust_i18n::t!("UPGRADING"));
    let status = Command::new("apt-get")
        .args(&["upgrade", "-y"])
        .stdout(Stdio::null())
        .stderr(error_log())
        .status();
    if !evaluate!(status, true) {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_UPGRADING"));
    }
}

pub fn select_language() {
    println!("{}:",rust_i18n::t!("SELECT_LANGUAGE"));
    println!("[1] English");
    println!("[2] Español");
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

    println!("{} {}",WARNING, rust_i18n::t!("DELETE_PKG_OBS"));
    let status = Command::new("apt-get")
        .args(&["autoremove", "-y"])
        .stdout(Stdio::null())
        .stderr(error_log())
        .status();
    if !evaluate!(status, true) {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_DELETE_PKG_OBS"));
    }

    println!("{} {}",WARNING, rust_i18n::t!("clear_CACHE"));
    let status = Command::new("apt-get")
        .args(&["autoclear", "-y"])
        .stdout(Stdio::null())
        .stderr(error_log())
        .status();
    if !evaluate!(status, true) {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_clear_CACHE"));
    }
}

pub fn passwd_root() -> bool {
    println!("{} {}", WARNING, rust_i18n::t!("CHANGE_PASSWD"));
    let status = Command::new("passwd").arg("root").status();
    evaluate!(status, true)
}

pub fn install_needed_software() {
    update();

    //Instala los paquetes necesarios
    println!("{} {}", WARNING, rust_i18n::t!("INSTALL_NECESSARY"));
    let status = Command::new("apt-get")
        .args([
            "install", "-y", 
            "figlet", "software-properties-common", "wget", "tar", "libncurses6", "libnuma1", "openssl", "net-tools", "ufw", "ca-certificates", "apt-transport-https"
        ])
        // Establecer DEBIAN_FRONTEND evita que apt se quede bloqueado esperando interacción del usuario
        .env("DEBIAN_FRONTEND", "noninteractive") 
        .stdout(Stdio::null())
        .stderr(error_log())
        .status();
    if !evaluate!(status, true) {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_INSTALL_NECESSARY"));
    }

    println!("[1/2] {} {}", WARNING, rust_i18n::t!("ADD_REPOSITORY_PHP"));
    //Agrega los repositorios de PHP
    let status = Command::new("add-apt-repository")
        .args(["ppa:ondrej/php", "-y"])
        .stdout(Stdio::null())
        .stderr(error_log()) // Revisa este log si algo falla
        .status();
    if !evaluate!(status, true) {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_ADD_REPOSITORY_PHP"));
    }
    println!("[2/2] {} {}", WARNING, rust_i18n::t!("ADD_REPOSITORY_APACHE"));
    let status = Command::new("add-apt-repository")
        .args(["ppa:ondrej/apache2", "-y"])
        .stdout(Stdio::null())
        .stderr(error_log())
        .status();
    if !evaluate!(status, true) {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_ADD_REPOSITORY_APACHE"));
    }

    update();

    //Instala programas necesarios
    println!("{} {} (Apache, MySQL, etc)...",WARNING, rust_i18n::t!("INSTALL_ESSENTIAL"));
    let status = Command::new("apt-get")
        .args([
            "install", "-y", 
            "apache2", "apache2-suexec-pristine", "apache2-suexec-custom", "libapache2-mod-fcgid",
            "mysql-server", "mysql-client"
        ])
        .env("DEBIAN_FRONTEND", "noninteractive")
        .stdout(Stdio::null())
        .stderr(error_log())
        .status();
    if !evaluate!(status, true) {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_INSTALL_ESSENTIAL"));
    }

    upgrade();
}

pub fn drivers_needed() {
    
    // 1. Mostramos el menú
    println!("=========================================");
    println!("{}", rust_i18n::t!("MACHINE"));
    println!("=========================================");
    println!("1) Virtualbox");
    println!("2) VMWare");
    println!("3) {}", rust_i18n::t!("NO_VM"));
    println!("=========================================");
    let vm = read_in!("{} [1-3]: ", rust_i18n::t!("SELECT_OPTION"));
    
    let opcion = vm.trim();

    
    println!("{}", rust_i18n::t!("INSTALLING_DRIVERS"));


    // 2. Ejecutamos el match y recolectamos el status de forma limpia
    let resultado_comando = match opcion {
        "1" => {
            clear_screen();
            update();
            let status = Command::new("apt-get")
                .args(&["install", "build-essential", "dkms", "virtualbox-guest-x11", "virtualbox-guest-utils", "-y"])
                .stdout(Stdio::null())
                .stderr(error_log())
                .status();
            
            println!("{}", rust_i18n::t!("RECOMMENDED_VIRTUALBOX"));
            Some(status)
        }
        "2" => {
            clear_screen();
            update();
            let status = Command::new("apt-get")
                .args(&["install", "open-vm-tools", "open-vm-tools-desktop", "-y"])
                .stdout(Stdio::null())
                .stderr(error_log())
                .status();
            
            Some(status)
        }
        "3" => {
            clear_screen();
            update();
            upgrade();
            let status = Command::new("apt-get")
                .args(&["install", "linux-headers-generic", "firmware-linux-free", "-y"])
                .stdout(Stdio::null())
                .stderr(error_log()) 
                .status();

            Some(status)
        }
        _ => { //unreachable!(),
            println!("{} {}",ERROR_YOU,rust_i18n::t!("BAD_SELECTED"));
            None
        }
    };

    if let Some(status) = resultado_comando {
        evaluate!(status, true);
    }
}

pub fn permisos() {
    println!("{} {}",WARNING,rust_i18n::t!("CHANGING_PERMISSION"));
    let status = Command::new("chown").args(["-R", "www-data:www-data", "/var/www"]).status();
    evaluate!(status, true);
}

pub fn reboot() {
    println!("=========================================");
    println!("          {}           ", rust_i18n::t!("REBOOT_SYSTEM"));
    println!("=========================================");
    println!("{} {} {}",WARNING,rust_i18n::t!("CAUTION"),rust_i18n::t!("NOTICE"));
    println!("=========================================\n");

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
    println!("=========================================");
    println!("    {}         ", rust_i18n::t!("CONFIG_AUTO_START_TITLE"));
    println!("=========================================");

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