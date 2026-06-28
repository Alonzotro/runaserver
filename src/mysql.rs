// ==========================================
// MYSQL MANAGEMENT
// ==========================================
use crate::public::{execute, evaluate, output, error_log, clear_screen, print_header, read_in, OK, INFO, WARNING, ERROR_YOU, ERROR_PC, ARROW, LOG_ERRORES};
use std::io::{self, Write};
use std::process::{Command, Stdio};
use mysql::{Conn, Opts};
use mysql::prelude::*;

pub fn ajustar_politicas_password() {
    // 1. Intentamos instalar de ambas formas de manera silenciosa. 
    // Ignoramos el resultado porque si ya existen, saltará un error inocuo.
    execute("mysql", &["-u", "root", "-e", "INSTALL COMPONENT 'file://component_validate_password';"], true, true);

    execute("mysql", &["-u", "root", "-e", "INSTALL PLUGIN validate_password SONAME 'validate_password.so';"], true, true);

    // 2. Intentar ajuste con sintaxis de Componente (Usamos 0 que es LOW universalmente)
    let query_componente = "SET GLOBAL validate_password.policy=0; SET GLOBAL validate_password.length=4;";
    if execute("mysql", &["-u", "root", "-e", query_componente], true, false) {
        println!("{} {}", OK, rust_i18n::t!("DB_POLICIES_SUCCESS"));
        return;
    }

    // 3. Fallback: Sintaxis de Plugin antiguo
    let query_plugin = "SET GLOBAL validate_password_policy=0; SET GLOBAL validate_password_length=4;";
    if execute("mysql", &["-u", "root", "-e", query_plugin], true, false) {
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

    // B. Inicio del servicio usando nuestro framework command
    println!("{}", rust_i18n::t!("STARTING_MYSQL"));
    if !execute("systemctl", &["start", "mysql"]) { return; }

    // C. Modificación de la configuración bind-address
    println!("{}", rust_i18n::t!("UPDATING_BIND_ADDRESS"));
    if !execute("sed", &["-i", "s/bind-address.*/bind-address = 0.0.0.0/", "/etc/mysql/mysql.conf.d/mysqld.cnf"]) { return; }

    // D. Ajustar políticas de contraseña previa instalación de la clave
    ajustar_politicas_password();

    // E. Inyectar contraseña inicial vía CLI
    println!("{}", rust_i18n::t!("SETTING_ROOT_PASS_OS"));
    let query_pass = format!("ALTER USER 'root'@'localhost' IDENTIFIED WITH caching_sha2_password BY '{}';", mysql_root_pass);
    if !execute("mysql", &["-u", "root", "-e", &query_pass]) { return; }

    // F. Conexión limpia ESTILO UNIX SOCKET (Soluciona el error de conexión)
    println!("{}", rust_i18n::t!("CONNECTING_RUST_MYSQL"));
    let opts = mysql::OptsBuilder::new()
        .user(Some("root"))
        .pass(Some(&mysql_root_pass))
        .socket(Some("/var/run/mysqld/mysqld.sock")); // 👈 Conexión local directa garantizada

    let mut conn = match mysql::Conn::new(opts) {
        Ok(c) => c,
        Err(e) => {
            println!("{}", rust_i18n::t!("ERR_DB_CONNECT", err = e.to_string()));
            return;
        }
    };

    // G. Securización atómica de la Base de Datos
    println!("{}", rust_i18n::t!("APPLYING_SECURITY_QUERIES"));
    
    // Asignamos la contraseña también al root remoto '%' para que no quede expuesto sin clave
    let query_root_remoto = format!("CREATE USER IF NOT EXISTS 'root'@'%' IDENTIFIED WITH caching_sha2_password BY '{}';", mysql_root_pass);
    let query_alter_remoto = format!("ALTER USER 'root'@'%' IDENTIFIED WITH caching_sha2_password BY '{}';", mysql_root_pass);

    let consultas_securizacion = [
        "DELETE FROM mysql.user WHERE User='';",
        "DROP DATABASE IF EXISTS test;",
        "DELETE FROM mysql.db WHERE Db='test' OR Db='test\\_%';",
        &query_root_remoto,
        &query_alter_remoto,
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
    if !execute("systemctl", &["restart", "mysql"]) { return; }

    println!("{}", rust_i18n::t!("OPENING_FIREWALL_3306"));
    if !execute("ufw", &["allow", "3306/tcp"]) { return; }

    println!("{} {}", OK, rust_i18n::t!("MYSQL_SUCCESS_FINAL"));
}

pub fn instalar_phpmyadmin() {
    println!("[!] Iniciando instalación de phpMyAdmin...");

    // 1. Agrupamos todas las respuestas en un solo bloque de texto
    let configs = [
        "phpmyadmin phpmyadmin/reconfigure-webserver multiselect apache2",
        "phpmyadmin phpmyadmin/dbconfig-install boolean true",
        "phpmyadmin phpmyadmin/mysql/admin-pass password root", // Cambia 'root'
        "phpmyadmin phpmyadmin/app-password-confirm password root",
        "phpmyadmin phpmyadmin/mysql/app-pass password root",
    ].join("\n");

    // 2. Inyectamos las respuestas a debconf vía STDIN usando un pipe en Bash.
    // Usamos ExecMode::Silent porque esto es pura carpintería interna.
    let debconf_cmd = format!("echo '{}' | debconf-set-selections", configs);
    if !execute("bash", &["-c", &debconf_cmd]) {
        println!("[X] Error al precargar las respuestas de debconf.");
        return;
    }

    // 3. Ejecutamos la instalación pasando el FRONTEND de Debian en la misma línea.
    // Usamos ExecMode::Interactive por si apt necesita mostrar barras de progreso en la TTY.
    let apt_args = ["-c", "DEBIAN_FRONTEND=noninteractive apt-get install -y phpmyadmin"];
    
    if execute("bash", &apt_args) {
        println!("[✓] phpMyAdmin instalado con éxito.");
        println!("Puedes acceder en: http://tu-ip/phpmyadmin");
    } else {
        println!("[X] Hubo un error al instalar phpMyAdmin.");
    }
}
// sudo mysql -uroot -proot
