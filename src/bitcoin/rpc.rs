use hyper::{Body, Client, Error, Method, Request};
use hyper::header::{Authorization, Basic};
use tokio_core::reactor::Core;
use futures::{Future, Stream};
use serde_json;
use std::str;
use bitcoin::Config;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeaderRpcResponse {
    pub result: BlockHeaderRpc,
    pub id: String,
    pub error: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeaderRpc {
    pub hash: String,
    pub height: u32,
    pub version: u32,
    pub nonce: u32,
    pub versionHex: String,
    pub merkleroot: String,
    pub time: u32,
    pub mediantime: u32,
    pub bits: String,
    pub difficulty: f64,
    pub chainwork: String,
    pub nextblockhash: Option<String>,
    pub previousblockhash: Option<String>,
}

pub fn get_block_header(
    block_hash: String,
    config: &Config,
) -> Result<BlockHeaderRpcResponse, Error> {
    let auth = Authorization(Basic {
        username: config.username.clone(),
        password: config.password.clone(),
    });
    let mut core = Core::new()?;
    let client = Client::new(&core.handle());
    let request_body_string: String = format!(
        "{{\"jsonrpc\":\"1.0\",\"id\":\"{}\",\"method\":\"{}\",\"params\":[\"{}\"]}}",
        0,
        "getblockheader",
        block_hash
    );
    let mut req: Request = Request::new(Method::Post, config.host.parse().unwrap());
    req.set_body(Body::from(request_body_string));
    req.headers_mut().set(auth);

    let future_res = client.request(req);

    let work = future_res.and_then(|res| {
        //println!("Response: {}", res.status());
        // read into a String, so that you don't need to do the conversion.
        res.body().concat2()
    });

    let work_result = core.run(work)?; //this throw on mac

    //println!("work_result {:?}", work_result);
    let utf8 = str::from_utf8(&work_result)?;

    //println!("GET: {}", utf8);
    let block_header_rpc_response: BlockHeaderRpcResponse = match serde_json::from_str(utf8) {
        Err(e) => return Err(Error::Io(e.into())),
        Ok(f) => f,
    };

    //let block_header_rpc_response = serde_json::from_str(utf8)?;

    Ok(block_header_rpc_response)
}
