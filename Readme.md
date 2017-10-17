
# Compressed Bitcoin Headers

The Bitcoin headers are probably the most condensed and important piece of data in the world, their demand is expected to grow.
Compressed Bitcoin Headers provides the bitcoin headers in a compressed form, saving about 45% of space.


## How it works

It works on chunks of 2016 headers (the difficulty adjustment period) by removing in all headers but the first, the previous header hash (32 bytes) and the difficulty (4 bytes).

When processing the headers stream the client:

 * Compute the hash of the first header (which is fully transmitted)
 * Save the difficulty bits which are constant for the next 2015 blocks.
 * Use this data to rebuild the missing information of the second header of the stream. Then repeat for the next headers.

Thus the headers byte stream is composed of one full header when `modulo(height,2016)==0` followed by 2015 headers stripped down of the previous hash and the difficulty bytes.

### Getting the headers

To get the headers information it connects to the RPC of a bitcoin full node. To retrieve connection information (rpcuser, rpcpassword and optionally rpchost) it scans the local machine for the bitcoin.conf file, looking in the following default paths:

* $HOME/Library/Application Support/Bitcoin/bitcoin.conf
* $HOME/.bitcoin/bitcoin.conf
* $HOME\AppData\Roaming\Bitcoin\bitcoin.conf

As of October 2017 it takes about 20 minutes to sync, then it stay on sync by asking the node for new headers every minute.

### Serving the compressed headers

To serve the headers the software starts an HTTP server and answer HTTP Range Request at the endpoint: _http://localhost:3000/bitcoin-headers_

 * _GET_ will return no data and an `Accept-Ranges: bytes` header
 * _HEAD_ request will return the content length of the stream
 * _GET_ with a header param `range: bytes=0-` will return the stream from the beginning to the end, `range: bytes=20000000-` will return from byte 20000000 to the end

The content type is `application/octet-stream`
The served headers are up to the connected node height less 6 to statistically avoid serving headers which could be reorged.

#### Public testing endpoint

https://finney.calendar.eternitywall.com/bitcoin-headers is a public available endpoint, you can donwload the header stream with: 

```
curl -v -X GET -H "range: bytes=0-" https://finney.calendar.eternitywall.com/bitcoin-headers >headers
```

## Building & launching

You need [rust](https://www.rust-lang.org/it-IT/) 

```
git clone https://github.com/RCasatta/rust-bitcoin-headers
cd rust-bitcoin-headers
cargo build --release
./target/release/rust-bitcoin-headers

```

The output will be something like this:
```
Found config file at /root/.bitcoin/bitcoin.conf
server starting at 127.0.0.1:3000
V4(127.0.0.1:3000)
Block #0 with hash 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f elapsed 0 seconds
Block #1000 with hash 00000000c937983704a73af28acdec37b049d214adbda81d7e2a3dd146f6ed09 elapsed 1 seconds
Block #1430 with hash 000000000009606d829b157912edb060c406b519fb2bfcc1078c196b69c67e49 is the min!
Block #2000 with hash 00000000dfd5d65c9d8561b4b8f60a63018fe3933ecb131fb37f905f87da951a elapsed 2 seconds
```

## Thanks

Thanks to Peter Todd and Gregory Maxwell 


  