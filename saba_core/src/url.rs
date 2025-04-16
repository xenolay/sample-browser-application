use alloc::string::{String, ToString};

#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    url: String,
    host: String,
    port: String,
    path: String,
    searchpart: String,
}

impl Url {
    pub fn new(url: &str) -> Self {
        Self { url: String::from(url),
            host: String::from(""),
            port: String::from(""),
            path: String::from(""),
            searchpart: String::from(""),
        }
    }

    pub fn parse(&self) -> Result<Self, String> {
        if self.is_not_http() {
            return Err(String::from("Only HTTP scheme is supported."))
        }

        let Some(host) = self.extract_host() else {
            return Err(String::from("Host parse failed"))
        };
        let port = self.extract_port();
        let path = self.extract_path();
        let searchpart = self.extract_searchpart();

        Ok(Url { url: self.url.clone(), host, port, path, searchpart })
    }

    // host が取れない場合だけは URL として不正とみなしたいので Option 型を返す
    fn extract_host(&self) -> Option<String> {
        self.url
            .trim_start_matches("http://")
            .split('/')
            .next()
            .and_then(|host_port| host_port.split(':').next())
            .and_then(|x| Some(x.to_string()))
    }

    fn extract_port(&self) -> String {
        self.url
            .trim_start_matches("http://")
            .split('/')
            .next()
            .and_then(|host_port| host_port.split(':').nth(1))
            .unwrap_or("80")
            .to_string()
    }

    fn extract_path(&self) -> String {
        self.url
            .trim_start_matches("http://")
            .splitn(2, "/")
            .nth(1)
            .and_then(|path_and_searchpart| path_and_searchpart.splitn(2, "?").nth(0))
            .unwrap_or("")
            .to_string()
    }

    fn extract_searchpart(&self) -> String {
        self.url
            .trim_start_matches("http://")
            .splitn(2, "/")
            .nth(1)
            .and_then(|path_and_searchpart| path_and_searchpart.splitn(2, "?").nth(1))
            .unwrap_or("")
            .to_string()
    }

    fn is_not_http(&self) -> bool {
        !self.url.starts_with("http://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url() {
        let url = "http://example.com".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        });
        assert_eq!(expected, Url::new(&url).parse());
    }

    #[test]
    fn test_url_with_port() {
        let url = "http://example.com:8888".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        });
        assert_eq!(expected, Url::new(&url).parse());
    }

    #[test]
    fn test_url_with_path() {
        let url = "http://example.com/index.html".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: "index.html".to_string(),
            searchpart: "".to_string(),
        });
        assert_eq!(expected, Url::new(&url).parse());
    }

    #[test]
    fn test_url_with_port_path() {
        let url = "http://example.com:8888/index.html".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            searchpart: "".to_string(),
        });
        assert_eq!(expected, Url::new(&url).parse());
    }

    #[test]
    fn test_url_with_port_path_searchpart() {
        let url = "http://example.com:8888/index.html?a=123&b=456".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            searchpart: "a=123&b=456".to_string(),
        });
        assert_eq!(expected, Url::new(&url).parse());
    }

    #[test]
    fn test_localhost() {
        let url = "http://localhost:8000".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "localhost".to_string(),
            port: "8000".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        });
        assert_eq!(expected, Url::new(&url).parse());
    }

    #[test]
    fn test_no_scheme() {
        let url = "example.com".to_string();
        let expected = Err("Only HTTP scheme is supported.".to_string());
        assert_eq!(expected, Url::new(&url).parse());
    }

    #[test]
    fn test_unsupported_scheme() {
        let url = "https://example.com:8888/index.html".to_string();
        let expected = Err("Only HTTP scheme is supported.".to_string());
        assert_eq!(expected, Url::new(&url).parse());
    }
}
