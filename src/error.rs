use hyper::{self, Body, Response};
use std::convert::{From, Into};

pub const HTTP_NOT_FOUND_HTML: &'static str = r#"
<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8" />
<link rel="shortcut icon" href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAACoUlEQVR4nO2bP276MBTHewSO0CP0CD1Cj9AjdK7i8JZEWaAsrbrwZ4qYqt8UMZGNbI3aCzBmQh7NgtyhJWoohPjFjm1+/kpvxHnvE3+/CZJzdaVJjwA3pA9T0ocpAFzr6qNzAUCP9OGJ+ECJD7ysPjzp7k2pAKBHCNwTH9aVwatFCYF73b1Kl+fBre9DXjN4pXwfcs+DW919t1bp84aD/ylb8+Gkz/Eg7MiHhj7Hltn5IOpzbP3kw53ueUu19rmt+SDd57bkg2Kfm50PXfkcW8ryQZvP8baQkw/G+BwPApcPhvocW2L5YLrPsdU4H1Q3QinljLGTFYSR0utrB8AY43VyABwAB8ABcAAcAAfAAXAAHAAH4PIAjCczvtvt/j8Ag+GIv+d57eB7fXx+8sFwdBkAgjDiqyw7u+2PaZVlSkB0BmA8mXFKqfDgx0BYBWA8mfGiKFoPrgqEMgBttntTMcb429s/swB0MfihKKVoEFIBtN3um82Gb7db9O+LohAGIQ1AEEboxtfrNR9PZuVaSbJAr7VfT8sOWKapUKOMMZ4ki5MvO6ssQwFIkoUeAEEYNXrUMcb4Mk0bveUFYdT4pYlzsbuvJATjeH62weeXV+Gwen55PQuCMSa8tnQAxIejQVgUBY/jufBaIiCWaSq8nhIAg+GoclfqfN4GxG/QlFLUNZQAIP53gL3nufJ/c3E855TSylPECAC2lAPgADgA9fI8uCOXcTKkUsIHqAiBB91NSypKCDwAQK/x8Hv9HJKaGjAEbnBZp8WsOyZHIH0EuGk9+KFMz4fODlIbmA94n2NlSD7I8zlW2vJBlc+x6iofjPtg4lAK86F7n2MlOR/0+xyr1vlgms+xEs0H432OVYN8sMfnWJ3IB3t9jlWZD5p9/gW9AKXEA7c0bgAAAABJRU5ErkJggg==">
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
<p>The server is powered by <a href="https://github.com/nkypy/tfrp" target="_blank">tfrp</a>.</p>
<p><em>Faithfully yours, tiny fast reverse proxy.</em></p>
</body>
</html>
"#;

pub struct Error;

impl Into<Error> for hyper::Error {
    fn into(self) -> Error {
        Error
    }
}

impl From<Error> for Response<Body> {
    fn from(_e: Error) -> Self {
        let body = Body::from(HTTP_NOT_FOUND_HTML);
        Response::builder().status(404).body(body).unwrap()
    }
}
