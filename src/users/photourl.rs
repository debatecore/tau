use serde::{Deserialize, Serialize};
use std::error::Error;
use url::Url;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Clone, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String")]
pub struct PhotoUrl {
    url: Url,
}

/// A type for storing links to photo URLs. When constructed, the link is automatically validated.
impl PhotoUrl {
    pub fn new(str: &str) -> Result<Self, PhotoUrlError> {
        let url = Url::parse(str).map_err(PhotoUrlError::InvalidUrl)?;

        if PhotoUrl::has_valid_extension(&url) {
            Ok(PhotoUrl { url })
        } else {
            Err(PhotoUrlError::InvalidUrlExtension)
        }
    }

    pub fn as_url(&self) -> &Url {
        &self.url
    }

    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }

    fn has_valid_extension(url: &Url) -> bool {
        let path = url.path();
        if let Some(filename) = path.split("/").last() {
            if let Some((name, ext)) = filename.rsplit_once(".") {
                if !name.is_empty() {
                    return match ext {
                        "png" | "jpg" | "jpeg" | "webp" => true,
                        _ => false,
                    };
                }
            }
        }

        false
    }
}

impl TryFrom<String> for PhotoUrl {
    type Error = PhotoUrlError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        PhotoUrl::new(&value)
    }
}

impl Into<String> for PhotoUrl {
    fn into(self) -> String {
        self.as_str().to_owned()
    }
}

#[derive(Debug)]
pub enum PhotoUrlError {
    InvalidUrl(url::ParseError),
    InvalidUrlExtension,
}

impl std::fmt::Display for PhotoUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhotoUrlError::InvalidUrl(e) => write!(f, "Invalid URL: {}", e),
            PhotoUrlError::InvalidUrlExtension => {
                write!(f, "URL must point to a valid image file.")
            }
        }
    }
}

impl Error for PhotoUrlError {}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::users::photourl::{PhotoUrl, PhotoUrlError};

    const EXPECT_FALSE: [&str; 10] = [
        "https://manczak.net",
        "unix://hello.net/apng",
        "unix://hello.net/ajpg",
        "unix://hello.net/ajpeg",
        "unix://hello.net/a.jpegg",
        "unix://hello.net/a.jpeg.jpe",
        "unix://hello.net/a.png.pnng",
        "unix://hello.net/a.jpg.jpge",
        "unix://hello.net/a/.jpg",
        "unix://hello.net/a./jpg",
    ];

    const EXPECT_TRUE: [&str; 7] = [
        "https://manczak.net/jmanczak.png",
        "https://manczak.net/jmanczak.jpg",
        "https://manczak.net/jmanczak.jpeg",
        "unix://hello.net/a.jpeg",
        "unix://hello.net/a.jpg",
        "unix://hello.net/a.png",
        "https://placehold.co/128x128.png",
    ];

    #[test]
    fn valid_extension_test() {
        for url in EXPECT_FALSE {
            let url = Url::parse(url).unwrap();
            assert!(PhotoUrl::has_valid_extension(&url) == false);
        }
        for url in EXPECT_TRUE {
            let url = Url::parse(url).unwrap();
            assert!(PhotoUrl::has_valid_extension(&url) == true);
        }
    }

    #[test]
    fn photo_url_deserialization() {
        for url in EXPECT_TRUE {
            let str = format!("\"{url}\"");
            let _json: PhotoUrl = serde_json::from_str(&str).unwrap();
        }
    }

    #[test]
    fn photo_bad_url_deserialization() {
        for url in EXPECT_FALSE {
            let json: Result<PhotoUrl, serde_json::Error> = serde_json::from_str(url);
            assert!(json.is_err());
        }
    }
}
