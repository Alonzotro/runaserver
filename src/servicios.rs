// ==========================================
// SERVICIOS PRINCIPALES
// ==========================================
use crate::{read_in};
use crate::public::*;
use crate::data::*;
use crate::checker::*;
use std::fs::{self, OpenOptions};
use std::env;
use std::future::Ready;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use serde::{Deserialize, Serialize};

pub const TARGET_BIN: &str = "/usr/local/bin/runaserver-bin";
pub const TARGET_WRAPPER: &str = "/usr/local/bin/runaserver";


pub fn update() {
    println!("{} {}",INFO, rust_i18n::t!("UPDATING"));
    if !execute("apt-get", &["update"]) {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_UPDATING"));
    }
}

fn upgrade() {
    println!("{} {}",INFO, rust_i18n::t!("UPGRADING"));
    if !execute("apt-get", &["upgrade", "-y"]) {
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
    if !execute("apt-get", &["autoremove", "-y"]) {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_DELETE_PKG_OBS"));
    }

    println!("{} {}",INFO, rust_i18n::t!("clear_CACHE"));
    if !execute("apt-get", &["autoclean", "-y"]) {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_clear_CACHE"));
    }
}

pub fn passwd_root() {
    println!("{} {}", INFO, rust_i18n::t!("CHANGE_PASSWD"));
    let status = Command::new("passwd").arg("root").status();
    evaluate(status);
}

pub fn install_needed_software() {
    update();

    //Instala los paquetes necesarios
    println!("{INFO} {}", rust_i18n::t!("INSTALL_NECESSARY"));

    let packages_raw = search_json("packages.json", "paquetes_base");
    let ggwellplayed = search_json("packages.json", "apache");

    install(&packages_raw);

    install(&ggwellplayed);



    println!("[1/2] {} {}", INFO, rust_i18n::t!("ADD_REPOSITORY_PHP"));
    if execute("add-apt-repository", &["ppa:ondrej/php", "-y"]) {

    } else {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_ADD_REPOSITORY_PHP"));
        return;
    }

    println!("[2/2] {} {}", INFO, rust_i18n::t!("ADD_REPOSITORY_APACHE"));
    if execute("add-apt-repository", &["ppa:ondrej/apache2", "-y"]) {

    } else {
        println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_ADD_REPOSITORY_APACHE"));
        return;
    }

    update();

    upgrade();
}

pub fn install(pkg: &[String]) {
    let (packages,_) = findout_software(pkg);

    if !packages.is_empty() {
        let status= Command::new("apt-get")
        .args(&["install", "-y"])
        .args(&packages)
        .stderr(error_log())
        .stdout(Stdio::null())
        .status();

        if evaluate(status) {
            println!("{OK} {}", "Exito");
        } else {
            println!("{} {}", ERROR_PC, rust_i18n::t!("ERROR_INSTALL_NECESSARY"));
            return;
        }
    }
}

pub fn drivers_needed() {
    
    // 1. Mostramos el menú
    print_header(&rust_i18n::t!("MACHINE"));
    println!("[1] Virtualbox");
    println!("[2] VMWare");
    println!("[3] {}", rust_i18n::t!("NO_VM"));
    line();
    let vm = read_in!("{} [1-3]: ", rust_i18n::t!("SELECT_OPTION"));

    let vm_trimmed = vm.trim();
    let seleccion: u32 = vm_trimmed.parse().unwrap_or(0);
    if !matches!(seleccion, 1..=3) {
        return;
    }

    clear_screen();
    println!("{}", rust_i18n::t!("INSTALLING_DRIVERS"));
    update();
    // 2. Ejecutamos el match y recolectamos el status de forma limpia
    let list = match seleccion {
        1 => {
            println!("{}", rust_i18n::t!("RECOMMENDED_VIRTUALBOX"));
            "pkg_virtualbox"
        }
        2 => {
            "pkg_vmware"
        }
        3 => {
            upgrade_server();
            "pkg_no_vm"
        }
        _ => {
            unreachable!();
        }
    };

    let packages_raw = search_json("packages.json", &list);
    install(&packages_raw);
}

pub fn permisos() {
    println!("{} {}",INFO,rust_i18n::t!("CHANGING_PERMISSION"));
    execute("chown",&["-R", "www-data:www-data", "/var/www"]);
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
            if evaluate(status) {
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

    println!("{}", rust_i18n::t!("GETTING_CURRENT_PATH"));
    let exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            println!("{} [CRITICAL] Imposible resolver la ruta del binario: {}", ERROR_PC, e);
            return;
        }
    };

    // 1. BORRADO IDIOMÁTICO (Cero TOCTOU)
    match fs::remove_file(TARGET_BIN) {
        Ok(_) => {
            println!("{} {}", WARNING, rust_i18n::t!("PREVIOUS_VERSION_DETECTED", path = TARGET_BIN));
            println!("{} {}", WARNING, rust_i18n::t!("DELETING_OLD_FILE"));
        }
        Err(e) if e.kind() == ErrorKind::NotFound => { /* No existía, perfecto */ }
        Err(e) => println!("{} Aviso: No se pudo limpiar el binario previo: {}", WARNING, e),
    }

    // 2. COPIA LIMPIA
    println!("{}", rust_i18n::t!("COPYING_NEW_VERSION"));
    if let Err(e) = fs::copy(&exe_path, TARGET_BIN) {
        println!("{} Fallo crítico copiando el binario: {}", ERROR_PC, e);
        return;
    }

    // 3. PERMISOS NATIVOS 0o755 (rwxr-xr-x) -> ¡¡Sin invocar a Bash!!
    println!("{}", rust_i18n::t!("SETTING_BIN_PERMISSIONS"));
    let modo_755 = fs::Permissions::from_mode(0o755);
    if let Err(e) = fs::set_permissions(TARGET_BIN, modo_755.clone()) {
        println!("{} Error aplicando permisos de ejecución: {}", ERROR_PC, e);
        return;
    }

    // 4. CREACIÓN DEL WRAPPER (Optimizado con 'exec' y sh POSIX)
    println!("{}", rust_i18n::t!("CONFIGURING_PRIVILEGE_ELEVATION"));
    
    let wrapper_content = format!(
        "#!/bin/sh\n\
        if [ \"$EUID\" -eq 0 ]; then\n\
        \texec \"{}\" \"$@\"\n\
        else\n\
        \texec pkexec \"{}\" \"$@\"\n\
        fi\n",
        TARGET_BIN, TARGET_BIN
    );

    if let Err(e) = fs::write(TARGET_WRAPPER, wrapper_content) {
        println!("{} No se pudo escribir el wrapper en /usr/local/bin: {}", ERROR_PC, e);
        return;
    }

    println!("{}", rust_i18n::t!("SETTING_WRAPPER_PERMISSIONS"));
    let _ = fs::set_permissions(TARGET_WRAPPER, modo_755);

    println!("{} {}", OK, rust_i18n::t!("UPDATE_COMPLETE_PKEXEC"));
}