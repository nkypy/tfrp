use crate::Result;
use hyper::{Body, Error as HyperError, Response};
use std::convert::{From, Into};
use warp::hyper::StatusCode;

const HTTP_NOT_FOUND_HTML: &'static str = r#"
<!DOCTYPE html>
<html>
<head>
<link rel="shortcut icon" href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACgAAAAoBAMAAAB+0KVeAAAAGFBMVEUAAAA7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozsmr7C1AAAAB3RSTlMA8o3HRiCjAZ7XVAAAAFRJREFUKM9jGAUMDE6GwiroYkHi5eWFqqhirOnlQFAWgCLIVA4GCiiCjhBBERRBdYhgEYqgOUSwGEVQHCJYSEgQoZ2QRQgnEXI8wpsEAgQRdKOAAQCvriuz6lBMNwAAAABJRU5ErkJggg==">
<title>Not Found</title>
<style>
    body {
        width: 35em;
        margin: 3em auto;
        font-family: Tahoma, Verdana, Arial, sans-serif;
    }
</style>
</head>
<body>
<h1>The page you requested was not found.</h1>
<p>Sorry, the page you are looking for is currently unavailable.<br/>
Please try again later.</p>
<p>The server is powered by <a href="https://github.com/nkypy/frp-rs" target="_blank">frp-rs</a>.</p>
<p><em>Faithfully yours, frp-rs.</em></p>
</body>
</html>
"#;

pub struct Error {
    pub code: u16,
    pub message: String,
}

impl From<Error> for Box<dyn warp::Reply> {
    fn from(_e: Error) -> Self {
        let body = warp::reply::html(HTTP_NOT_FOUND_HTML);
        Box::new(warp::reply::with_status(body, StatusCode::NOT_FOUND))
    }
}

impl Into<Error> for hyper::Error {
    fn into(self) -> Error {
        Error {
            code: 404,
            message: "NOT_FOUND".to_owned(),
        }
    }
}

impl From<Error> for Response<Body> {
    fn from(_e: Error) -> Self {
        let body = Body::from(HTTP_NOT_FOUND_HTML);
        Response::new(body)
    }
}
