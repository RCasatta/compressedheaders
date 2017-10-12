use std::sync::{Arc, Mutex};
use hyper;
use futures;
use hyper::header::{Headers,ContentLength,ContentType,Range,RangeUnit,AcceptRanges,ByteRangeSpec};
use hyper::server::{Http, Request, Response, Service};
use std::net::SocketAddr;
use bitcoin::header::BlockHeader;
use hyper::StatusCode;

#[derive(Clone)]
struct HeaderServices {
    block_headers_bytes : Arc<Mutex<Vec<u8>>>,
}

pub fn start(block_headers_bytes : Arc<Mutex<Vec<u8>>>) {
    let x = "127.0.0.1:3000";
    println!("server starting at {}", x);
    let addr : SocketAddr = x.parse().unwrap();
    println!("{:?}",addr);
    let server = Http::new().bind(&addr,move || Ok(HeaderServices {
        block_headers_bytes : block_headers_bytes.clone(),
    })).unwrap();
    server.run().unwrap();
}

impl Service for HeaderServices {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, _req: Request) -> Self::Future {


        let response = match validate_req(_req) {
            Err(e) => Response::new().with_status(e) ,
            Ok(r) => build_range_response(self.block_headers_bytes.clone(), r) ,
        };

        futures::future::ok(response)
    }
}

fn validate_req(_req: Request ) -> Result<Option<Range>, StatusCode> {
    let uri_path = _req.uri().path();

    match uri_path.eq("/bitcoin-headers") {
        true  => {
            match _req.headers().get::<Range>() {
                Some(r) => Ok(Some(r.clone())),
                None => Ok(None)
            }
        },
        false => Err(StatusCode::NotFound),
    }
}


fn build_range_response( block_headers_bytes_arc : Arc<Mutex<Vec<u8>>>, range : Option<Range>) -> Response {
    let block_headers_bytes = block_headers_bytes_arc.lock().unwrap();
    match range {
        Some(range) => {
            println!("{:?}",range);

            match range {
                Range::Bytes(r) => {
                    let (start, end) = match r[0] {
                        ByteRangeSpec::AllFrom(start) => {
                            println!("AllFrom {}", start);
                            (start as usize, block_headers_bytes.len())
                        },
                        ByteRangeSpec::FromTo(start, end) => {
                            println!("FromTo {} {}", start, end);
                            (start as usize, end as usize)
                        },
                        ByteRangeSpec::Last(x) => {
                            println!("Last {}", x);
                            let end = block_headers_bytes.len();
                            (end - (x as usize), end)
                        }
                    };
                    println!("{}-{}",start,end);
                    let mut reply = Vec::with_capacity(end-start);
                    reply.extend(&block_headers_bytes[start..end]);

                    Response::new()
                        .with_header(ContentType::octet_stream())
                        .with_header(ContentLength(reply.len() as u64))
                        .with_body(reply)
                },
                Range::Unregistered(r,s) => {
                    Response::new().with_status(StatusCode::NotFound)
                }
            }
        },
        None => {
            let mut headers = Headers::new();
            headers.set(AcceptRanges(vec![RangeUnit::Bytes]));
            headers.set(ContentLength( block_headers_bytes.len() as u64 ));

            Response::new()
                .with_headers( headers )
                .with_status(StatusCode::Ok)
        }
     }

    // headers.set(AcceptRanges(vec![RangeUnit::Bytes]));
    //headers.set(Range::bytes(1, 100));
}
/*
fn build_response(parsed_request : ParsedRequest, block_headers : Arc<Mutex<Vec<Option<BlockHeader>>>>) -> Response {
    let chunk_number = parsed_request.chunk_number.unwrap();
    let (start,end) = match parsed_request.request_type {
        RequestType::_1    => (1*chunk_number, 1*chunk_number+1),
        RequestType::_144  => (144*chunk_number, 144*chunk_number+144),
        RequestType::_2016 => (2016*chunk_number, 2016*chunk_number+2016),
        _ => (0,0)
    };
    let locked_block_headers = block_headers.lock().unwrap();
    if end > locked_block_headers.len() {
        Response::new().with_status(StatusCode::NotFound)
    } else {
        match parsed_request.request_type {
            RequestType::_1 =>
                println!("Request type: _1 chunk: {} -> Returning header of block {}", chunk_number, start),
            _ =>
                println!("Request type: {:?} chunk: {} -> Returning compressed headers from {} to {}",parsed_request.request_type, chunk_number, start,end-1),
        }

        let mut vec : Vec<u8> = Vec::new();
        let first = locked_block_headers[start].unwrap();
        vec.extend(first.as_bytes().into_iter() );
        for i in start+1..end {
            let current = locked_block_headers[i].unwrap();
            let compressed = current.as_compressed_bytes();
            vec.extend(compressed.into_iter() );
        }
        Response::new()
            .with_header(ContentType::octet_stream())
            .with_header(ContentLength(vec.len() as u64))
            .with_body(vec)
    }
}

*/
