use ferrous_opencc::config::BuiltinConfig;
use ferrous_opencc::OpenCC;

use crate::error::AppError;

/// Map a mode string like "s2t" to the corresponding [`BuiltinConfig`].
pub fn mode_to_config(mode: &str) -> Result<BuiltinConfig, AppError> {
    match mode {
        "s2t" => Ok(BuiltinConfig::S2t),
        "t2s" => Ok(BuiltinConfig::T2s),
        "s2tw" => Ok(BuiltinConfig::S2tw),
        "tw2s" => Ok(BuiltinConfig::Tw2s),
        "s2hk" => Ok(BuiltinConfig::S2hk),
        "hk2s" => Ok(BuiltinConfig::Hk2s),
        "s2twp" => Ok(BuiltinConfig::S2twp),
        "tw2sp" => Ok(BuiltinConfig::Tw2sp),
        "t2tw" => Ok(BuiltinConfig::T2tw),
        "tw2t" => Ok(BuiltinConfig::Tw2t),
        "t2hk" => Ok(BuiltinConfig::T2hk),
        "hk2t" => Ok(BuiltinConfig::Hk2t),
        "t2jp" => Ok(BuiltinConfig::T2jp),
        "jp2t" => Ok(BuiltinConfig::Jp2t),
        _ => Err(AppError::UnsupportedMode(mode.to_string())),
    }
}

/// Create an [`OpenCC`] converter for the given mode.
pub fn create_converter(mode: &str) -> Result<OpenCC, AppError> {
    let config = mode_to_config(mode)?;
    OpenCC::from_config(config).map_err(|e| AppError::OpenCC(e.to_string()))
}

/// Convert a single string.
pub fn convert_text(converter: &OpenCC, text: &str) -> String {
    converter.convert(text)
}
