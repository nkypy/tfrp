use hyper::{self, Body, Response};
use std::convert::{From, Into};

const HTTP_NOT_FOUND_HTML: &'static str = r#"
<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8" />
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
<p>The server is powered by <a href="https://github.com/nkypy/frp-rs" target="_blank">tfrp</a>.</p>
<p><em>Faithfully yours, tfrp.</em></p>
</body>
</html>
"#;

pub struct Error {}

impl Into<Error> for hyper::Error {
    fn into(self) -> Error {
        Error {}
    }
}

impl From<Error> for Response<Body> {
    fn from(_e: Error) -> Self {
        let body = Body::from(HTTP_NOT_FOUND_HTML);
        Response::builder()
            .status(404)
            .body(body)
            .unwrap()
    }
}
