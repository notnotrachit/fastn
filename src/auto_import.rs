#[derive(serde::Deserialize, Debug, Clone)]
pub struct AutoImport {
    pub path: String,
    pub alias: Option<String>,
}

impl AutoImport {
    pub fn from_string(name: &str) -> AutoImport {
        match name.split_once(" as ") {
            Some((package, alias)) => fpm::AutoImport {
                path: package.trim().to_string(),
                alias: Some(alias.trim().to_string()),
            },
            None => fpm::AutoImport {
                path: name.trim().to_string(),
                alias: None,
            },
        }
    }
}
