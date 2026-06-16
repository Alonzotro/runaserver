# Ubuntu Server Automator (Rust) 🚀

Este es un script desarrollado en **Rust** diseñado para automatizar la configuración y transformar cualquier máquina con **Ubuntu 24.04 LTS (Noble Numbat)** en un servidor listo para producción. 

Aunque fue desarrollado principalmente con entornos virtuales (VPS) en mente, funciona perfectamente en hardware real (máquinas físicas).

---

## 📌 Requisitos de Compatibilidad

Actualmente, el script está optimizado exclusivamente para:
* **Ubuntu 24.04 LTS (Noble Numbat)**
* *(Opcional)* Compatibilidad teórica con **Ubuntu 22.04 LTS (Jammy Jellyfish)** debido a las dependencias de terceros.

### ¿Por qué solo estas versiones?
El script añade repositorios externos que dependen directamente del soporte del mantenedor. Los repositorios de **Ondřej Surý** (esenciales para las últimas versiones de PHP y Apache2) solo están disponibles de forma oficial para las versiones LTS activas de Ubuntu:
* Ubuntu 24.04 (Noble)
* Ubuntu 22.04 (Jammy)

> 💡 **¿Conoces la disponibilidad en otras distribuciones?** > Si has probado este script o conoces la disponibilidad de estos repositorios para otras versiones/distribuciones de Linux, te invitamos a abrir un **Issue** para ayudarnos a expandir la compatibilidad.

---

## ✨ Características principales

El script ejecuta de forma automatizada las siguientes tareas de administración:

### 1. Actualización Completa del Sistema
Mantiene tu sistema al día ejecutando la limpieza de paquetes residuales:
```bash
apt update && apt upgrade -y
apt autoremove -y && apt autoclean -y