use std::collections::HashMap;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub files: Walk,
    pub default: EngineConfig,
}

impl Config {
    pub fn from_dir(cwd: &std::path::Path) -> Result<Option<Self>, anyhow::Error> {
        let config = if let Some(path) =
            find_project_file(cwd, &["typos.toml", "_typos.toml", ".typos.toml"])
        {
            Some(Self::from_file(&path)?)
        } else {
            None
        };
        Ok(config)
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, anyhow::Error> {
        let s = std::fs::read_to_string(path)?;
        Self::from_toml(&s)
    }

    pub fn from_toml(data: &str) -> Result<Self, anyhow::Error> {
        let content = toml::from_str(data)?;
        Ok(content)
    }

    pub fn from_defaults() -> Self {
        Self {
            files: Walk::from_defaults(),
            default: EngineConfig::from_defaults(),
        }
    }

    pub fn update(&mut self, source: &Config) {
        self.files.update(&source.files);
        self.default.update(&source.default);
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
#[serde(rename_all = "kebab-case")]
pub struct Walk {
    /// Skip hidden files and directories.
    pub ignore_hidden: Option<bool>,
    /// Respect ignore files.
    pub ignore_files: Option<bool>,
    /// Respect .ignore files.
    pub ignore_dot: Option<bool>,
    /// Respect ignore files in vcs directories.
    pub ignore_vcs: Option<bool>,
    /// Respect global ignore files.
    pub ignore_global: Option<bool>,
    /// Respect ignore files in parent directories.
    pub ignore_parent: Option<bool>,
}

impl Walk {
    pub fn from_defaults() -> Self {
        let empty = Self::default();
        Self {
            ignore_hidden: Some(empty.ignore_hidden()),
            ignore_files: Some(true),
            ignore_dot: Some(empty.ignore_dot()),
            ignore_vcs: Some(empty.ignore_vcs()),
            ignore_global: Some(empty.ignore_global()),
            ignore_parent: Some(empty.ignore_parent()),
        }
    }

    pub fn update(&mut self, source: &Walk) {
        if let Some(source) = source.ignore_hidden {
            self.ignore_hidden = Some(source);
        }
        if let Some(source) = source.ignore_files {
            self.ignore_files = Some(source);
            self.ignore_dot = None;
            self.ignore_vcs = None;
            self.ignore_global = None;
            self.ignore_parent = None;
        }
        if let Some(source) = source.ignore_dot {
            self.ignore_dot = Some(source);
        }
        if let Some(source) = source.ignore_vcs {
            self.ignore_vcs = Some(source);
            self.ignore_global = None;
        }
        if let Some(source) = source.ignore_global {
            self.ignore_global = Some(source);
        }
        if let Some(source) = source.ignore_parent {
            self.ignore_parent = Some(source);
        }
    }

    pub fn ignore_hidden(&self) -> bool {
        self.ignore_hidden.unwrap_or(true)
    }

    pub fn ignore_dot(&self) -> bool {
        self.ignore_dot.or(self.ignore_files).unwrap_or(true)
    }

    pub fn ignore_vcs(&self) -> bool {
        self.ignore_vcs.or(self.ignore_files).unwrap_or(true)
    }

    pub fn ignore_global(&self) -> bool {
        self.ignore_global
            .or(self.ignore_vcs)
            .or(self.ignore_files)
            .unwrap_or(true)
    }

    pub fn ignore_parent(&self) -> bool {
        self.ignore_parent.or(self.ignore_files).unwrap_or(true)
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
#[serde(rename_all = "kebab-case")]
pub struct EngineConfig {
    /// Check binary files.
    pub binary: Option<bool>,
    /// Verifying spelling in file names.
    pub check_filename: Option<bool>,
    /// Verifying spelling in files.
    pub check_file: Option<bool>,
    #[serde(flatten)]
    pub tokenizer: Option<TokenizerConfig>,
    #[serde(flatten)]
    pub dict: Option<DictConfig>,
}

impl EngineConfig {
    pub fn from_defaults() -> Self {
        let empty = Self::default();
        EngineConfig {
            binary: Some(empty.binary()),
            check_filename: Some(empty.check_filename()),
            check_file: Some(empty.check_file()),
            tokenizer: Some(
                empty
                    .tokenizer
                    .unwrap_or_else(TokenizerConfig::from_defaults),
            ),
            dict: Some(empty.dict.unwrap_or_else(DictConfig::from_defaults)),
        }
    }

    pub fn update(&mut self, source: &EngineConfig) {
        if let Some(source) = source.binary {
            self.binary = Some(source);
        }
        if let Some(source) = source.check_filename {
            self.check_filename = Some(source);
        }
        if let Some(source) = source.check_file {
            self.check_file = Some(source);
        }
        if let Some(source) = source.tokenizer.as_ref() {
            let mut tokenizer = None;
            std::mem::swap(&mut tokenizer, &mut self.tokenizer);
            let mut tokenizer = tokenizer.unwrap_or_default();
            tokenizer.update(source);
            let mut tokenizer = Some(tokenizer);
            std::mem::swap(&mut tokenizer, &mut self.tokenizer);
        }
        if let Some(source) = source.dict.as_ref() {
            let mut dict = None;
            std::mem::swap(&mut dict, &mut self.dict);
            let mut dict = dict.unwrap_or_default();
            dict.update(source);
            let mut dict = Some(dict);
            std::mem::swap(&mut dict, &mut self.dict);
        }
    }

    pub fn binary(&self) -> bool {
        self.binary.unwrap_or(false)
    }

    pub fn check_filename(&self) -> bool {
        self.check_filename.unwrap_or(true)
    }

    pub fn check_file(&self) -> bool {
        self.check_file.unwrap_or(true)
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
#[serde(rename_all = "kebab-case")]
pub struct TokenizerConfig {
    /// Do not check identifiers that appear to be hexadecimal values.
    pub ignore_hex: Option<bool>,
    /// Allow identifiers to start with digits, in addition to letters.
    pub identifier_leading_digits: Option<bool>,
    /// Allow identifiers to start with one of these characters.
    pub identifier_leading_chars: Option<kstring::KString>,
    /// Allow identifiers to include digits, in addition to letters.
    pub identifier_include_digits: Option<bool>,
    /// Allow identifiers to include these characters.
    pub identifier_include_chars: Option<kstring::KString>,
}

impl TokenizerConfig {
    pub fn from_defaults() -> Self {
        let empty = Self::default();
        Self {
            ignore_hex: Some(empty.ignore_hex()),
            identifier_leading_digits: Some(empty.identifier_leading_digits()),
            identifier_leading_chars: Some(kstring::KString::from_ref(
                empty.identifier_leading_chars(),
            )),
            identifier_include_digits: Some(empty.identifier_include_digits()),
            identifier_include_chars: Some(kstring::KString::from_ref(
                empty.identifier_include_chars(),
            )),
        }
    }

    pub fn update(&mut self, source: &TokenizerConfig) {
        if let Some(source) = source.ignore_hex {
            self.ignore_hex = Some(source);
        }
        if let Some(source) = source.identifier_leading_digits {
            self.identifier_leading_digits = Some(source);
        }
        if let Some(source) = source.identifier_leading_chars.as_ref() {
            self.identifier_leading_chars = Some(source.clone());
        }
        if let Some(source) = source.identifier_include_digits {
            self.identifier_include_digits = Some(source);
        }
        if let Some(source) = source.identifier_include_chars.as_ref() {
            self.identifier_include_chars = Some(source.clone());
        }
    }

    pub fn ignore_hex(&self) -> bool {
        self.ignore_hex.unwrap_or(true)
    }

    pub fn identifier_leading_digits(&self) -> bool {
        self.identifier_leading_digits.unwrap_or(false)
    }

    pub fn identifier_leading_chars(&self) -> &str {
        self.identifier_leading_chars.as_deref().unwrap_or("_")
    }

    pub fn identifier_include_digits(&self) -> bool {
        self.identifier_include_digits.unwrap_or(true)
    }

    pub fn identifier_include_chars(&self) -> &str {
        self.identifier_include_chars.as_deref().unwrap_or("_'")
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
#[serde(rename_all = "kebab-case")]
pub struct DictConfig {
    pub locale: Option<Locale>,
    pub extend_identifiers: HashMap<kstring::KString, kstring::KString>,
    pub extend_words: HashMap<kstring::KString, kstring::KString>,
}

impl DictConfig {
    pub fn from_defaults() -> Self {
        let empty = Self::default();
        Self {
            locale: Some(empty.locale()),
            extend_identifiers: Default::default(),
            extend_words: Default::default(),
        }
    }

    pub fn update(&mut self, source: &DictConfig) {
        if let Some(source) = source.locale {
            self.locale = Some(source);
        }
        self.extend_identifiers.extend(
            source
                .extend_identifiers
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
        self.extend_words.extend(
            source
                .extend_words
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }

    pub fn locale(&self) -> Locale {
        self.locale.unwrap_or_default()
    }

    pub fn extend_identifiers(&self) -> Box<dyn Iterator<Item = (&str, &str)> + '_> {
        Box::new(
            self.extend_identifiers
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str())),
        )
    }

    pub fn extend_words(&self) -> Box<dyn Iterator<Item = (&str, &str)> + '_> {
        Box::new(
            self.extend_words
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str())),
        )
    }
}

fn find_project_file(dir: &std::path::Path, names: &[&str]) -> Option<std::path::PathBuf> {
    let mut file_path = dir.join("placeholder");
    for name in names {
        file_path.set_file_name(name);
        if file_path.exists() {
            return Some(file_path);
        }
    }
    None
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Locale {
    En,
    EnUs,
    EnGb,
    EnCa,
    EnAu,
}

impl Locale {
    pub const fn category(self) -> Option<typos_vars::Category> {
        match self {
            Locale::En => None,
            Locale::EnUs => Some(typos_vars::Category::American),
            Locale::EnGb => Some(typos_vars::Category::BritishIse),
            Locale::EnCa => Some(typos_vars::Category::Canadian),
            Locale::EnAu => Some(typos_vars::Category::Australian),
        }
    }

    pub const fn variants() -> [&'static str; 5] {
        ["en", "en-us", "en-gb", "en-ca", "en-au"]
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::En
    }
}

impl std::str::FromStr for Locale {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "en" => Ok(Locale::En),
            "en-us" => Ok(Locale::EnUs),
            "en-gb" => Ok(Locale::EnGb),
            "en-ca" => Ok(Locale::EnCa),
            "en-au" => Ok(Locale::EnAu),
            _ => Err("valid values: en, en-us, en-gb, en-ca, en-au".to_owned()),
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Locale::En => write!(f, "en"),
            Locale::EnUs => write!(f, "en-us"),
            Locale::EnGb => write!(f, "en-gb"),
            Locale::EnCa => write!(f, "en-ca"),
            Locale::EnAu => write!(f, "en-au"),
        }
    }
}
