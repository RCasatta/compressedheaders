use std::sync::{Arc, Mutex};
use hyper;
use futures;
use hyper::header::{ContentLength,ContentType};
use hyper::server::{Http, Request, Response, Service};
use std::net::SocketAddr;
use bitcoin::header::BlockHeader;
use hyper::StatusCode;

#[derive(Clone)]
struct HeaderServices {
    block_headers : Arc<Mutex<Vec<Option<BlockHeader>>>>,
}

#[derive(Debug, Clone, Copy)]
enum RequestType {
    _2016,
    _144,
    _1,
    Invalid,
}

#[derive(Debug, Clone, Copy)]
struct ParsedRequest {
    request_type : RequestType,
    chunk_number : Option<usize>,
}

pub fn start(block_headers : Arc<Mutex<Vec<Option<BlockHeader>>>>) {

    let x = "127.0.0.1:3000";
    println!("server starting at {}", x);
    let addr : SocketAddr = x.parse().unwrap();
    println!("{:?}",addr);

    let server = Http::new().bind(&addr,move || Ok(HeaderServices {
        block_headers : block_headers.clone()
    })).unwrap();
    server.run().unwrap();
}

impl Service for HeaderServices {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = futures::future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, _req: Request) -> Self::Future {
        let parsed_request = validate_req(_req);
        let response = match (parsed_request.request_type, parsed_request.chunk_number) {
            (RequestType::Invalid,_) => Response::new().with_status(StatusCode::NotFound),
            (_,None) => Response::new().with_status(StatusCode::BadRequest),
            _ => build_response(parsed_request, self.block_headers.clone()),
        };

        futures::future::ok(response)
    }
}

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

fn validate_req(_req: Request ) -> ParsedRequest {
    let uri_path = _req.uri().path();

    let request_type = match (uri_path.starts_with("/headers/2016/"), uri_path.starts_with("/headers/144/"), uri_path.starts_with("/headers/1/")) {
        (true, false, false) => RequestType::_2016,
        (false, true, false) => RequestType::_144,
        (false, false, true) => RequestType::_1,
        _ => RequestType::Invalid,
    };

    let num : Option<usize> = match request_type {
        RequestType::_2016 => parse_uri(&uri_path[14..]),
        RequestType::_144  => parse_uri(&uri_path[13..]),
        RequestType::_1    => parse_uri(&uri_path[11..]),
        RequestType::Invalid => None,
    };

    ParsedRequest {
        request_type : request_type,
        chunk_number : num,
    }
}

#[allow(unused_variables)]
fn parse_uri(num : &str) -> Option<usize> {
    match num.parse::<usize>() {
        Ok(n) => Some(n),
        Err(e) => None,
    }
}
