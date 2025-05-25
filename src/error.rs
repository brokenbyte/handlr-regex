/// Custom error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Xdg(#[from] xdg::BaseDirectoriesError),
    #[error(transparent)]
    Config(#[from] confy::ConfyError),
    #[error("No handlers found for '{0}'")]
    NotFound(String),
    #[error(
        "Could not find a mimetype associated with the file extension: '{0}'"
    )]
    AmbiguousExtension(String),
    #[error(transparent)]
    BadMimeType(#[from] mime::FromStrError),
    #[error("Bad mime: {0}")]
    InvalidMime(mime::Mime),
    #[error("Malformed desktop entry at {0}")]
    BadEntry(std::path::PathBuf),
    #[error(transparent)]
    BadRegex(#[from] regex::Error),
    #[error("Error spawning selector process '{0}'")]
    Selector(String),
    #[error("Selection cancelled")]
    Cancelled,
    #[error("Please specify the default terminal with handlr set x-scheme-handler/terminal")]
    NoTerminal,
    #[error("Bad path: {0}")]
    BadPath(String),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeIniDe(#[from] serde_ini::de::Error),
    #[error(transparent)]
    SerdeIniSer(#[from] serde_ini::ser::Error),
    #[error("Could not split exec command '{0}' in desktop file '{1}' into shell words")]
    BadExec(String, String),
    #[error("Could not split command '{0}' into shell words")]
    BadCmd(String),
    #[error(transparent)]
    TracingGlobalDefault(#[from] tracing::dispatcher::SetGlobalDefaultError),
    #[cfg(test)]
    #[error(transparent)]
    BadUrl(#[from] url::ParseError),
    #[cfg(test)]
    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
