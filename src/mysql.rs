// ==========================================
// MYSQL MANAGEMENT
// ==========================================
//use crate::{registrar_log_error, leer_linea, limpiar_pantalla, LOG_ERRORES, OK, WARNING, ERROR};
use crate::{OK, WARNING, ERROR_YOU, ERROR_PC, registrar_log_error, Evaluable, evaluate};
use std::io::{self, Write};
use std::process::{Command, Stdio};
use mysql::{Conn, Opts};
use mysql::prelude::*;

pub fn ajustar_politicas_password() {
    // 1. Intentar activar como Componente moderno
    println!("{}", rust_i18n::t!("TRY_INSTALL_COMP"));
    let status_comp = Command::new("mysql")
        .args(["-u", "root", "-e", "INSTALL COMPONENT 'file://component_validate_password';"])
        .stdout(Stdio::null())
        .stderr(registrar_log_error())
        .status();
    evaluate!(status_comp, true);

    // 2. Intentar activar como Plugin clásico
    println!("{}", rust_i18n::t!("TRY_INSTALL_PLUGIN"));
    let status_plugin = Command::new("mysql")
        .args(["-u", "root", "-e", "INSTALL PLUGIN validate_password SONAME 'validate_password.so';"])
        .stdout(Stdio::null())
        .stderr(registrar_log_error())
        .status();
    evaluate!(status_plugin, true);

    // 3. Intentar ajuste con sintaxis de Componente (puntos ".")
    println!("{}", rust_i18n::t!("SETTING_POLICIES_COMP"));
    let query_componente = "SET GLOBAL validate_password.policy=LOW; SET GLOBAL validate_password.length=4;";
    let status_opt1 = Command::new("mysql")
        .args(["-u", "root", "-e", query_componente])
        .stdout(Stdio::null())
        .stderr(registrar_log_error())
        .status();

    if evaluate!(status_opt1, true) {
        println!("{} {}", OK, rust_i18n::t!("DB_POLICIES_SUCCESS"));
        return; // Si funcionó, salimos limpios
    }

    // 4. Fallback: Intentar ajuste con sintaxis de Plugin (guiones bajos "_")
    println!("{}", rust_i18n::t!("SETTING_POLICIES_PLUGIN"));
    let query_plugin = "SET GLOBAL validate_password_policy=LOW; SET GLOBAL validate_password_length=4;";
    let status_opt2 = Command::new("mysql")
        .args(["-u", "root", "-e", query_plugin])
        .stdout(Stdio::null())
        .stderr(registrar_log_error())
        .status();

    if evaluate!(status_opt2, true) {
        println!("{} {}", OK, rust_i18n::t!("DB_POLICIES_SUCCESS"));
    }
}

// 2. FUNCIÓN PRINCIPAL
pub fn configurar_mysql_seguro() {
    println!("=========================================");
    println!("       {}    ", rust_i18n::t!("MYSQL_CONFIG_TITLE"));
    println!("=========================================");

    // A. Obtención y validación de contraseña
    print!("{}", rust_i18n::t!("PROMPT_ROOT_PASS"));
    let _ = io::stdout().flush();
    let mysql_root_pass = rpassword::read_password().unwrap_or_default();

    print!("{}", rust_i18n::t!("PROMPT_CONFIRM_PASS"));
    let _ = io::stdout().flush();
    let mysql_root_pass_confirm = rpassword::read_password().unwrap_or_default();

    if mysql_root_pass.is_empty() || mysql_root_pass != mysql_root_pass_confirm {
        println!("{}", rust_i18n::t!("PASS_MISMATCH"));
        return;
    }

    // B. Inicio del servicio
    println!("{}", rust_i18n::t!("STARTING_MYSQL"));
    let status_start = Command::new("systemctl")
        .args(["start", "mysql"])
        .stdout(Stdio::null())
        .stderr(registrar_log_error())
        .status();
    if !evaluate!(status_start, true) { return; }

    // C. Modificación de la configuración bind-address
    println!("{}", rust_i18n::t!("UPDATING_BIND_ADDRESS"));
    let status_sed = Command::new("sed")
        .args(["-i", "s/bind-address.*/bind-address = 0.0.0.0/", "/etc/mysql/mysql.conf.d/mysqld.cnf"])
        .stdout(Stdio::null())
        .stderr(registrar_log_error())
        .status();
    if !evaluate!(status_sed, true) { return; }

    // D. Ajustar políticas (Llamada a tu función segura estandarizada)
    ajustar_politicas_password();

    // E. Inyectar contraseña inicial vía CLI del OS
    println!("{}", rust_i18n::t!("SETTING_ROOT_PASS_OS"));
    let query_pass = format!("ALTER USER 'root'@'localhost' IDENTIFIED WITH caching_sha2_password BY '{}';", mysql_root_pass);
    let status_alter = Command::new("mysql")
        .args(["-u", "root", "-e", &query_pass])
        .stdout(Stdio::null())
        .stderr(registrar_log_error())
        .status();
    if !evaluate!(status_alter, true) { return; }

    // F. Conexión limpia y segura estilo PDO usando OptsBuilder (Anti-Panics)
    println!("{}", rust_i18n::t!("CONNECTING_RUST_MYSQL"));
let opts = mysql::OptsBuilder::new()
    .ip_or_hostname(Some("127.0.0.1"))
    .user(Some("root"))
    .pass(Some(&mysql_root_pass))
    .tcp_port(3306);

// 2. Le pasamos 'opts' directamente a la conexión
let mut conn = match mysql::Conn::new(opts) {
    Ok(c) => c,
    Err(e) => {
        println!("{}", rust_i18n::t!("ERR_DB_CONNECT", err = e.to_string()));
        return;
    }
};

    // G. Securización atómica de la Base de Datos
    println!("{}", rust_i18n::t!("APPLYING_SECURITY_QUERIES"));
    
    // Separamos los comandos SQL. Ejecutar multi-queries en una sola cadena puede fallar según la config de red del driver
    let consultas_securizacion = [
        "DELETE FROM mysql.user WHERE User='';",
        "DROP DATABASE IF EXISTS test;",
        "DELETE FROM mysql.db WHERE Db='test' OR Db='test\\_%';",
        "CREATE USER IF NOT EXISTS 'root'@'%';",
        "GRANT ALL PRIVILEGES ON *.* TO 'root'@'%' WITH GRANT OPTION;",
        "FLUSH PRIVILEGES;"
    ];

    for query in consultas_securizacion {
        if let Err(e) = conn.query_drop(query) {
            println!("{}", rust_i18n::t!("ERR_DB_QUERY", err = e.to_string()));
            return;
        }
    }

    // H. Reinicio final y apertura de Firewall
    println!("{}", rust_i18n::t!("RESTARTING_MYSQL_FINAL"));
    let status_restart = Command::new("systemctl")
        .args(["restart", "mysql"])
        .stdout(Stdio::null())
        .stderr(registrar_log_error())
        .status();
    if !evaluate!(status_restart, true) { return; }

    println!("{}", rust_i18n::t!("OPENING_FIREWALL_3306"));
    let status_ufw = Command::new("ufw")
        .args(["allow", "3306/tcp"])
        .stdout(Stdio::null())
        .stderr(registrar_log_error())
        .status();
    if !evaluate!(status_ufw, true) { return; }

    // Éxito absoluto
    println!("{} {}", OK, rust_i18n::t!("MYSQL_SUCCESS_FINAL"));
}



pub fn instalar_phpmyadmin() {
    println!("[!] Iniciando instalación phpMyAdmin...");

    // 1. Pre-configuramos las respuestas del instalador (debconf)
    // - phpmyadmin/reconfigure-webserver: Elegimos apache2
    // - phpmyadmin/dbconfig-install: Decimos que sí queremos configurar la BD automáticamente
    // - phpmyadmin/mysql/admin-pass: Aquí deberías poner la pass de root de MySQL
    
    let preconfig = [
        "phpmyadmin phpmyadmin/reconfigure-webserver multiselect apache2",
        "phpmyadmin phpmyadmin/dbconfig-install boolean true",
        "phpmyadmin phpmyadmin/mysql/admin-pass password root", // Cambia 'root' por tu contraseña
        "phpmyadmin phpmyadmin/app-password-confirm password root",
        "phpmyadmin phpmyadmin/mysql/app-pass password root",
    ];

    for config in preconfig {
        let _ = Command::new("debconf-set-selections")
            .arg("-v")
            .arg(config)
            .status();
    }

    // 2. Instalación en modo "non-interactive"
    // Esto es clave: DEBIAN_FRONTEND=noninteractive evita que aparezcan ventanas
    let status = Command::new("apt-get")
        .env("DEBIAN_FRONTEND", "noninteractive")
        .args(&["install", "-y", "phpmyadmin"])
        .stdout(Stdio::inherit())
        .status();

    if status.is_ok() && status.unwrap().success() {
        println!("[✓] phpMyAdmin instalado con éxito.");
        println!("Puedes acceder en: http://tu-ip/phpmyadmin");
    } else {
        println!("[X] Hubo un error al instalar phpMyAdmin.");
    }
}

// sudo mysql -uroot -proot
