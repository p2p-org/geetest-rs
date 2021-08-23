extern crate geetest;
extern crate tokio;

use geetest::prelude::*;
use hyper::{
    body::Bytes,
    header,
    service::{make_service_fn, Service},
    Body, Request, Response, StatusCode,
};
use std::{
    convert::Infallible,
    future::Future,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{fs::File, io::AsyncReadExt};
use futures_util::Stream;

#[derive(Debug, Clone)]
struct Static<T> {
    root: PathBuf,
    fallback: T,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let captcha_id = env!("GEETEST_CAPTCHA_ID");
    let captcha_secret = env!("GEETEST_CAPTCHA_SECRET");

    let client = Client::new(captcha_id.to_owned(), DigestMod::Md5);
    let server = Server::new(client, captcha_secret.to_owned());

    let static_server = Static {
        root: PathBuf::from("static"),
        fallback: server,
    };

    let addr: SocketAddr = ("127.0.0.1".parse::<IpAddr>().unwrap(), 8000).into();
    log::info!("Running server at http://{}", addr);

    hyper::Server::bind(&addr)
        .serve(make_service_fn(move |_| {
            let svc = static_server.clone();
            async { Ok::<_, Infallible>(svc) }
        }))
        .await?;

    Ok(())
}

type BoxFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;

impl<T> Static<T> {
    async fn serve_file(path: PathBuf) -> Result<Response<Body>, Error> {
        let file = File::open(&path).await?;
        let file_size = file.metadata().await?.len();
        let file_type = match path.extension().and_then(|ext| ext.to_str()) {
            Some("js") => "text/javascript",
            Some("json") => "application/json",
            Some("html") => "text/html",
            Some("css") => "text/css",
            _ => "text/plain",
        };

        use async_stream::try_stream;

        fn read_file(mut file: File) -> impl Stream<Item = Result<Bytes, std::io::Error>> {
            let mut buf = [0u8; 1024];
            try_stream! {
                loop {
                    let n = file.read(&mut buf[..]).await?;
                    if n == 0 {
                        yield Bytes::new();
                        return;
                    } else {
                        yield Bytes::from(buf.to_vec());
                    }
                }
            }
        }

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_LENGTH, file_size)
            .header(header::CONTENT_TYPE, file_type)
            .body(Body::wrap_stream(read_file(file)))?)
    }
}

impl<T> Service<Request<Body>> for Static<T>
where
    T: Service<Request<Body>, Response = Response<Body>, Error = Error, Future = BoxFuture<Response<Body>, Error>>,
    T: 'static,
{
    type Response = Response<Body>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let path = req.uri().path();
        if path.starts_with("/static/") {
            let file_path = self.root.join(&path[8..]);
            Box::pin(Self::serve_file(file_path))
        } else {
            self.fallback.call(req)
        }
    }
}
