//! Deteção de plataforma e adaptação de recursos

use std::env;
use std::path::PathBuf;

/// Plataforma alvo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Windows,
    MacOS,
    Android,
    IOs,
    Unknown,
}

impl Platform {
    /// Detecta a plataforma em tempo de compilação
    pub fn current() -> Self {
        if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOS
        } else if cfg!(target_os = "android") {
            Self::Android
        } else if cfg!(target_os = "ios") {
            Self::IOs
        } else {
            Self::Unknown
        }
    }

    /// Retorna o nome da plataforma para exibição
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Windows => "Windows",
            Self::MacOS => "macOS",
            Self::Android => "Android",
            Self::IOs => "iOS",
            Self::Unknown => "Unknown",
        }
    }

    /// Verifica se a plataforma suporta aceleração GPU nativa
    pub fn has_gpu_acceleration(&self) -> bool {
        match self {
            Self::Linux | Self::Windows | Self::MacOS => true,
            Self::Android | Self::IOs => true,  // via NNAPI/Metal
            _ => false,
        }
    }

    /// Caminho padrão para modelos
    pub fn default_model_path(&self) -> PathBuf {
        match self {
            Self::Android | Self::IOs => {
                // Em dispositivos móveis, o modelo fica no diretório de assets
                PathBuf::from("assets/model.qt")
            }
            Self::Linux | Self::Windows | Self::MacOS => {
                PathBuf::from("/models/cathedral_33t.qt")
            }
            _ => PathBuf::from("model.qt"),
        }
    }
}

/// Inicializa recursos específicos da plataforma
pub fn init_platform() {
    match Platform::current() {
        Platform::Android => {
            // Configurar JNI, permissões
            #[cfg(target_os = "android")]
            android_logger::init_once(
                android_logger::Config::default()
                    .with_min_level(log::Level::Info)
            );
        }
        Platform::IOs => {
            // Configurar logging para iOS
            #[cfg(target_os = "ios")]
            unsafe {
                // Inicializar sistema de logging
            }
        }
        _ => {
            tracing_subscriber::fmt::init();
        }
    }
}
