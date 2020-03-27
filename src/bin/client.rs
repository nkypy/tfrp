use frp::error::Error;
use frp::Result;
use futures::TryFutureExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use warp::{reject, Filter, Rejection, Reply};

async fn index(req: Request<Body>) -> std::result::Result<Response<Body>, hyper::Error> {
    let url = req.uri().to_string();
    println!("url is {}", url);
    let e = Error {
        code: 404,
        message: "".to_string(),
    };
    Ok(e.into())
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8100));
    let srv = Server::bind(&addr).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(index))
    }));
    srv.await?;
    Ok(())
    // let hello_app = warp::path!("hello" / String);
    // let hello = hello_app //warp::path!("hello" / String)
    //     .map(|name| format!("Hello, {}!", name));
    // // let others = hello_app  // warp::path!("hello"/String)
    // //     .and(warp::post()).map(|name| format!("POST Hello, {}!",name));
    // let others = warp::any().and_then(handle_error);
    // let routes = warp::get().and(hello).or(others); // .recover(handle_rejection);
    //
    // warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    // Ok(())
}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_error() -> std::result::Result<Box<dyn Reply>, Infallible> {
    let e = Error {
        code: 0,
        message: "".to_string(),
    };
    Ok(Box::from(e))
}

async fn handle_rejection(err: Rejection) -> std::result::Result<Box<dyn Reply>, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }
    let e = Error {
        code: 0,
        message: "123".to_string(),
    };
    Ok(Box::from(e))

    // Ok(warp::reply::html(message))
}
