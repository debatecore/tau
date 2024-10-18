use std::error::Error;

use url::Url;

#[derive(Debug)]
pub struct PhotoUrl {
    url: Url,
}

impl PhotoUrl {
    pub fn new(str: &str) -> Result<Self, PhotoUrlError> {
        let url = Url::parse(str).map_err(PhotoUrlError::InvalidUrl)?;

        if PhotoUrl::has_valid_extension(&url) {
            Ok(Self { url })
        } else {
            Err(PhotoUrlError::InvalidUrlExtension)
        }
    }

    pub fn as_url(&self) -> &Url {
        &self.url
    }

    fn has_valid_extension(url: &Url) -> bool {
        let path = url.path();
        if let Some(filename) = path.split("/").last() {
            if let Some((name, ext)) = filename.rsplit_once(".") {
                if !name.is_empty() {
                    return match ext {
                        "png" | "jpg" | "jpeg" => true,
                        _ => false,
                    };
                }
            }
        }

        false
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

#[test]
fn valid_extension_test() {
    let expect_false = vec![
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
    for url in expect_false {
        let url = Url::parse(url).unwrap();
        assert!(PhotoUrl::has_valid_extension(&url) == false);
    }
    let expect_true = vec![
        "https://manczak.net/jmanczak.png",
        "https://manczak.net/jmanczak.jpg",
        "https://manczak.net/jmanczak.jpeg",
        "unix://hello.net/a.jpeg",
        "unix://hello.net/a.jpg",
        "unix://hello.net/a.png",
    ];
    for url in expect_true {
        let url = Url::parse(url).unwrap();
        assert!(PhotoUrl::has_valid_extension(&url) == true);
    }
}
