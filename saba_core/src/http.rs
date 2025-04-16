use alloc::{string::{String, ToString}, vec::Vec};
use super::error::Error;


#[derive(Debug, Clone)]
pub struct HttpResponse {
    version: String,
    status_code: u32,
    reason: String,
    headers: Vec<Header>,
    body: String,
}

impl HttpResponse {
    pub fn new(raw_response: String) -> Result<Self, Error> {
        // [] 3. Request Line | RFC 9112 - HTTP/1.1
        // https://datatracker.ietf.org/doc/html/rfc9112#name-request-line
        // ----- Cited From Reference -----
        //   HTTP-message   = start-line CRLF
        //                    *( field-line CRLF )
        //                    CRLF
        //                    [ message-body ]
        // --------------------------------

        // [] Although the line terminator for the start-line and fields is the sequence CRLF, a recipient MAY recognize a single LF as a line terminator and ignore any preceding CR.¶ | RFC 9112 - HTTP/1.1
        // https://datatracker.ietf.org/doc/html/rfc9112#section-2.2-3
        // ----- Cited From Reference -----
        // Although the line terminator for the start-line and fields is the sequence CRLF, a recipient MAY recognize a single LF as a line terminator and ignore any preceding CR.¶
        // --------------------------------        
        // ということで、CRLF を LF に置き換えて解釈してよいから、そうする
        let preprocessed_response = raw_response.trim_start().replace("\r\n", "\n");

        let (status_line, remaining_lines) = match preprocessed_response.split_once("\n") {
            Some((s, r)) => (s, r),
            None => return Err(Error::Network(alloc::format!("invalid http response: {}", preprocessed_response))),
        };

        let (headers, body) = match remaining_lines.split_once("\n\n") {
            Some((h, b)) => {
                let mut headers = Vec::new();
                for header in h.split("\n") {
                    let splitted_header: Vec<&str> = header.splitn(2, ":").collect();
                    headers.push(
                        Header::new(String::from(splitted_header[0].trim()), String::from(splitted_header[1].trim()))
                    )
                }
                (headers, b)
            }
            None => (Vec::new(), remaining_lines),
        };

        let statuses: Vec<&str> = status_line.split(" ").collect();

        Ok(HttpResponse { 
            version: statuses.get(0).unwrap_or(&"").to_string(),
            status_code: statuses.get(1).copied().and_then(|x| x.parse().ok()).unwrap_or(404),
            reason: statuses.get(2).unwrap_or(&"").to_string(),
            headers,
            body: body.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    name: String,
    value: String,
}

impl Header {
    fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}
