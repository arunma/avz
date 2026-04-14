use thiserror::Error;

#[derive(Error, Debug)]
pub enum AvzError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Avro error: {0}")]
    Avro(#[from] apache_avro::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("S3 error: {0}")]
    S3(String),

    #[error("{0}")]
    User(String),
}

pub type Result<T> = std::result::Result<T, AvzError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_io() {
        let err = AvzError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "gone"));
        assert!(err.to_string().contains("gone"));
    }

    #[test]
    fn test_error_display_user() {
        let err = AvzError::User("bad input".into());
        assert_eq!(err.to_string(), "bad input");
    }

    #[test]
    fn test_error_display_s3() {
        let err = AvzError::S3("timeout".into());
        assert!(err.to_string().contains("timeout"));
    }
}
