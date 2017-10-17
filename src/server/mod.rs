use std::sync::{Arc, Mutex};
use hyper;
use futures;
use hyper::header::{AcceptRanges, ByteRangeSpec, ContentLength, ContentType, Headers, Range,
                    RangeUnit};
use hyper::server::{Http, Request, Response, Service};
use std::net::SocketAddr;
use hyper::StatusCode;

#[derive(Clone)]
struct HeaderServices {
    block_headers_bytes: Arc<Mutex<Vec<u8>>>,
}

pub fn start(block_headers_bytes: Arc<Mutex<Vec<u8>>>) {
    let x = "127.0.0.1:3000";
    println!("server starting at http://{}", x);
    let addr: SocketAddr = x.parse().unwrap();
    let server = Http::new()
        .bind(&addr, move || {
            Ok(HeaderServices {
                block_headers_bytes: block_headers_bytes.clone(),
            })
        })
        .unwrap();
    server.run().unwrap();
}

impl Service for HeaderServices {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, _req: Request) -> Self::Future {
        let response = match validate_req(_req) {
            Err(e) => Response::new().with_status(e),
            Ok(r) => build_range_response(self.block_headers_bytes.clone(), r),
        };
        futures::future::ok(response)
    }
}

fn validate_req(_req: Request) -> Result<Option<Range>, StatusCode> {
    let uri_path = _req.uri().path();

    match uri_path.eq("/bitcoin-headers") {
        true => match _req.headers().get::<Range>() {
            Some(r) => Ok(Some(r.clone())),
            None => Ok(None),
        },
        false => Err(StatusCode::NotFound),
    }
}


fn build_range_response(
    block_headers_bytes_arc: Arc<Mutex<Vec<u8>>>,
    range: Option<Range>,
) -> Response {
    let block_headers_bytes = block_headers_bytes_arc.lock().unwrap();
    match range {
        Some(range) => match range {
            Range::Bytes(r) => {
                let (start, end) = match r[0] {
                    ByteRangeSpec::AllFrom(start) => (start as usize, block_headers_bytes.len()),
                    ByteRangeSpec::FromTo(start, end) => (start as usize, end as usize),
                    ByteRangeSpec::Last(x) => {
                        let end = block_headers_bytes.len();
                        (end - (x as usize), end)
                    }
                };
                println!("Range request {}-{}", start, end);

                let mut reply = Vec::with_capacity(end - start);
                reply.extend(&block_headers_bytes[start..end]);

                Response::new()
                    .with_header(ContentType::octet_stream())
                    .with_header(ContentLength(reply.len() as u64))
                    .with_body(reply)
            }
            Range::Unregistered(_, _) => Response::new().with_status(StatusCode::NotFound),
        },
        None => {
            let mut headers = Headers::new();
            headers.set(AcceptRanges(vec![RangeUnit::Bytes]));
            headers.set(ContentLength(block_headers_bytes.len() as u64));

            Response::new()
                .with_headers(headers)
                .with_status(StatusCode::Ok)
        }
    }
}
