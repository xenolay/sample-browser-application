extern crate alloc;
use alloc::string::String;
use noli::net::{lookup_host, SocketAddr, TcpStream};
use noli::print;
use saba_core::error::Error;
use saba_core::http::HttpResponse;
pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
        let ips = match lookup_host(&host) {
            Ok(ips) => ips,
            Err(_) => return Err(Error::Network(String::from("Failed to find IP addresses"))),
        };

        if ips.len() < 1 {
            return Err(Error::Network(String::from("Failed to find IP addresses")))
        }

        let socket_addr: SocketAddr = (ips[0], port).into();

        let mut stream = match TcpStream::connect(socket_addr) {
            Ok(stream) => stream,
            Err(_) => return Err(Error::Network(String::from("Failed to connect to TCP stream"))),
        };

        // 3. Request Line | RFC 9112 - HTTP/1.1
        // https://datatracker.ietf.org/doc/html/rfc9112#name-request-line
        // ----- Cited From Reference -----
        //   HTTP-message   = start-line CRLF
        //                    *( field-line CRLF )
        //                    CRLF
        //                    [ message-body ]
        // --------------------------------

        // 3. Request Line | RFC 9112 - HTTP/1.1
        // https://datatracker.ietf.org/doc/html/rfc9112#name-request-line
        // ----- Cited From Reference -----
        // request-line   = method SP request-target SP HTTP-version
        // --------------------------------
        
        let mut request = String::from("GET /");
        request.push_str(&path);
        request.push_str(" HTTP/1.1\n");

        // 7.2. Host and :authority | RFC 9110 - HTTP Semantics
        // https://datatracker.ietf.org/doc/html/rfc9110#name-host-and-authority
        // ----- Cited From Reference -----
        // The "Host" header field in a request provides the host and port information from the target URI, enabling the origin server to distinguish among resources while servicing requests for multiple host names.¶
        
        // In HTTP/2 [HTTP/2] and HTTP/3 [HTTP/3], the Host header field is, in some cases, supplanted by the ":authority" pseudo-header field of a request's control data.¶
        
        //   Host = uri-host [ ":" port ] ; Section 4
        // --------------------------------

        request.push_str("Host: ");
        request.push_str(&host);
        request.push_str("\n");

        // 12.5.1. Accept | RFC 9110 - HTTP Semantics
        // https://datatracker.ietf.org/doc/html/rfc9110#name-accept
        // ----- Cited From Reference -----
        // The "Accept" header field can be used by user agents to specify their preferences regarding response media types. For example, Accept header fields can be used to indicate that the request is specifically limited to a small set of desired types, as in the case of a request for an in-line image.
        // --------------------------------
        request.push_str("Accept: text/html\n");

        // 3. Request Line | RFC 9112 - HTTP/1.1
        // https://datatracker.ietf.org/doc/html/rfc9112#name-request-line
        // ----- Cited From Reference -----
        // 9.6. Tear-down
        // The "close" connection option is defined as a signal that the sender will close this connection after completion of the response. A sender SHOULD send a Connection header field (Section 7.6.1 of [HTTP]) containing the "close" connection option when it intends to close a connection. For example,¶
        
        // Connection: close
        // ¶
        // as a request header field indicates that this is the last request that the client will send on this connection, while in a response, the same field indicates that the server is going to close this connection after the response message is complete.¶
        // --------------------------------
        request.push_str("Connection: close\n");

        // ここ削ると408が見れる。確かに RFC で指定された CRLF が存在しない形になるので
        request.push_str("\r\n");

        let _bytes = match stream.write(request.as_bytes()) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::Network(String::from("Failed to send a request to TCP stream"))),
        };

        print!("write done!\n\n\n");

        let mut received = alloc::vec::Vec::new();

        loop {
            let mut buf = [0u8; 4096];
            let bytes_read = match stream.read(&mut buf) {
                Ok(bytes) => bytes,
                Err(_) => return Err(Error::Network(String::from("Failed to receive a request from TCP stream"))),
            };
            if bytes_read == 0 {
                break;
            }
            received.extend_from_slice(&buf[..bytes_read]);
        }

        print!("read done!\n\n\n");

        match String::from_utf8(received) {
            Ok(result) =>         HttpResponse::new(result),
            Err(e) => Err(Error::Network(alloc::format!("Invalid received response: {}", e)))
        }
    }
}
