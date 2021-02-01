extern crate pretty_env_logger;
#[macro_use] extern crate log;

use tokio::signal;
use std::convert::Infallible;
use hyper::service::{make_service_fn, service_fn, };
use hyper::{Body, Request, Response, Server, Client, StatusCode};
use rand::random;


async fn proxy(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let host = req.headers().get("host").unwrap().to_str().unwrap();

    let hostnames = match host {
        "localhost:3000" => vec!["localhost:3001", "localhost:3002"],
        "localhost:4000" => vec!["localhost:4001", "localhost:4002"],
        &_ => vec![],
    };

    if hostnames.len() == 0 {
        return Ok(Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .body(Body::empty()).unwrap())
    }

    let hostname = hostnames[random::<usize>() % hostnames.len()];

    let mut request = Request::builder()
        .method(req.method())
        .uri(format!("http://{}", hostname));

    for (name, value) in req.headers().iter() {
        request = request.header(name, value);
    }

    let request = request.body(req.into_body()).unwrap();

    let client = Client::new();

    let response = client.request(request).await.unwrap();

    Ok(response)
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    let addr = ([0, 0, 0, 0], 8080).into();

    let server = Server::bind(&addr).serve(make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(proxy)) }
    }));

    let graceful = server.with_graceful_shutdown(async {
        signal::ctrl_c().await.ok();
        info!("shutdown");
    });

    info!("server started");

    graceful.await?;

    Ok(())
}
