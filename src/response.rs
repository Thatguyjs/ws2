// Build & send HTTP Responses

use std::{io::Write, path::Path};


// pub type Result = std::result::Result<Response, HttpError>;


#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Status {
    // 2xx
    Ok,

    // 3xx
    MovedPermanently,
    Found,
    SeeOther,
    TemporaryRedirect,
    PermanentRedirect,

    // 4xx
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    TooManyRequests,

    // 5xx
    InternalServerError,
    ServiceUnavailable
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Status::Ok => "200 Ok",

            Status::MovedPermanently => "301 Moved Permanently",
            Status::Found => "302 Found",
            Status::SeeOther => "303 See Other",
            Status::TemporaryRedirect => "307 Temporary Redirect",
            Status::PermanentRedirect => "308 Permanent Redirect",

            Status::BadRequest => "400 Bad Request",
            Status::Unauthorized => "401 Unauthorized",
            Status::Forbidden => "403 Forbidden",
            Status::NotFound => "404 Not Found",
            Status::MethodNotAllowed => "405 Method Not Allowed",
            Status::TooManyRequests => "429 Too Many Requests",

            Status::InternalServerError => "500 Internal Server Error",
            Status::ServiceUnavailable => "503 Service Unavailable"
        }
    }
}


// Returns a MIME type from a filepath
pub fn mime_from_path(path: &Path) -> Option<&'static str> {
    Some(match path.extension()?.to_str()? {
        "aac" => "audio/aac",
        "apng" => "image/apng",
        "avif" => "image/avif",
        "bin" => "application/octet-stream",
        "bmp" => "image/bmp",
        "css" => "text/css",
        "csv" => "text/csv",
        "gif" => "image/gif",
        "htm" | "html" => "text/html",
        "ico" => "image/x-icon",
        "jpg" | "jpeg" => "image/jpeg",
        "js" | "mjs" => "text/javascript",
        "json" => "application/json",
        "m4a" => "audio/mp4",
        "mp3" => "audio/mp3",
        "mp4" => "video/mp4",
        "mpeg" => "video/mpeg",
        "oga" => "audio/ogg",
        "ogv" => "video/ogg",
        "ogx" => "application/ogg",
        "opus" => "audio/ogg",
        "otf" => "font/otf",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "svg" => "image/svg+xml",
        "tif" | "tiff" => "image/tiff",
        "ttf" => "font/ttf",
        "txt" => "text/plain",
        "wav" => "audio/wav",
        "weba" => "audio/webm",
        "webm" => "video/webm",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "xml" => "text/xml",
        "zip" => "application/zip",

        _ => return None
    })
}


#[derive(Debug)]
pub struct Header(&'static str, Vec<u8>);


// A finalized HTTP Response, ready to be sent
#[derive(Debug)]
pub struct Response {
    pub status: Status,
    pub headers: Vec<Header>,
    pub body: Vec<u8>
}

impl Response {
    // `true` if a header is found, `false` otherwise
    fn contains_header(&self, name: &'static str) -> bool {
        let name = name.to_lowercase();
        self.headers.iter().any(|header| header.0.to_lowercase() == name)
    }

    // Add basic headers that all responses should have
    fn add_basic_headers(&mut self) {
        if !self.contains_header("Content-Length") {
            self.headers.push(Header("Content-Length", self.body.len().to_string().into_bytes()));
        }
    }

    // Calculate how many bytes this response will take up
    fn calculate_size(&self) -> usize {
        let status_len = self.status.as_str().as_bytes().len();
        let mut header_len = 0;

        for header in &self.headers {
            header_len += header.0.as_bytes().len() + header.1.len();
        }

        status_len + header_len + self.body.len() + 15 // +15 for "HTTP/1.1 " + the 3 newlines (\r\n)
    }

    // Combine all headers into a Vec of bytes
    fn collect_headers(&self) -> std::io::Result<Vec<u8>> {
        let mut headers = vec![];

        for header in &self.headers {
            headers.write(b"\r\n")?;
            headers.write(header.0.as_bytes())?;
            headers.write(b": ")?;
            headers.write(&header.1)?;
        }

        headers.flush()?;
        Ok(headers)
    }

    // Create a simple response with a status code and text for the body
    pub fn text<B: Into<Vec<u8>>>(status: Status, body: B) -> Self {
        Response {
            status,
            headers: vec![],
            body: body.into()
        }
    }

    pub fn try_into_bytes(mut self) -> std::io::Result<Vec<u8>> {
        self.add_basic_headers();

        let mut res = Vec::with_capacity(self.calculate_size());

        res.write(b"HTTP/1.1 ")?;
        res.write(self.status.as_str().as_bytes())?;
        res.write(&self.collect_headers()?)?;
        res.write(b"\r\n\r\n")?;
        res.write(&self.body)?;

        Ok(res)
    }
}


// Builds HTTP Responses step-by-step
// Use a builder pattern to avoid a bunch of methods on Response structs
pub struct Builder {
    status: Status,
    headers: Vec<Header>,
    body: Vec<u8>
}

impl Builder {
    pub fn with_status(status: Status) -> Builder {
        Builder {
            status,
            headers: vec![],
            body: vec![]
        }
    }

    pub fn add_header<V: Into<Vec<u8>>>(mut self, name: &'static str, value: V) -> Self {
        self.headers.push(Header(name, value.into()));
        self
    }

    pub fn set_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn build(self) -> Response {
        Response {
            status: self.status,
            headers: self.headers,
            body: self.body
        }
    }
}
