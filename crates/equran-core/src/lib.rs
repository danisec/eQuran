pub mod api;
pub mod audio;
pub mod display;
pub mod playback;

pub mod domain {
    use std::str::FromStr;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Qari {
        Juhany,
        Qasim,
        Sudais,
        Dossari,
        Misyari,
        Yasser,
    }

    impl Qari {
        pub fn key(self) -> &'static str {
            match self {
                Self::Juhany => "01",
                Self::Qasim => "02",
                Self::Sudais => "03",
                Self::Dossari => "04",
                Self::Misyari => "05",
                Self::Yasser => "06",
            }
        }

        pub fn label(self) -> &'static str {
            match self {
                Self::Juhany => "Abdullah Al-Juhany",
                Self::Qasim => "Abdul Muhsin Al-Qasim",
                Self::Sudais => "Abdurrahman As-Sudais",
                Self::Dossari => "Ibrahim Al-Dossari",
                Self::Misyari => "Misyari Rasyid Al-Afasy",
                Self::Yasser => "Yasser Al-Dosari",
            }
        }
    }

    impl FromStr for Qari {
        type Err = String;

        fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
            match value.to_ascii_lowercase().as_str() {
                "juhany" | "abdullah" | "01" => Ok(Self::Juhany),
                "qasim" | "02" => Ok(Self::Qasim),
                "sudais" | "03" => Ok(Self::Sudais),
                "dossari" | "ibrahim" | "04" => Ok(Self::Dossari),
                "misyari" | "mishary" | "afasy" | "05" => Ok(Self::Misyari),
                "yasser" | "yasir" | "06" => Ok(Self::Yasser),
                other => Err(format!(
                    "unknown qari '{other}'. Use juhany, qasim, sudais, dossari, misyari, or yasser"
                )),
            }
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Lang {
        Id,
        En,
    }

    impl Lang {
        pub fn code(self) -> &'static str {
            match self {
                Self::Id => "id",
                Self::En => "en",
            }
        }
    }

    impl FromStr for Lang {
        type Err = String;

        fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
            match value.to_ascii_lowercase().as_str() {
                "id" | "indonesian" | "indonesia" => Ok(Self::Id),
                "en" | "english" => Ok(Self::En),
                other => Err(format!("unknown language '{other}'. Use id or en")),
            }
        }
    }
}
