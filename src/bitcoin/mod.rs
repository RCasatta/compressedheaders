pub mod rpc;
pub mod header;

use std::env;
use std::fs::File;
use std::io::Read;


pub struct Config {
    host : String,
    username : String,
    password : Option<String>,
}

impl Config {

    fn new (host: String, username: String, password: Option<String>) -> Config {
        Config {
            host,
            username,
            password,
        }
    }
    pub fn read() -> Result<Config, &'static str> {
        let mut host: Option<String> = Some(String::from("http://localhost:8332"));
        let mut username: Option<String> = None;
        let mut password: Option<String> = None;

        match env::home_dir() {
            Some(path) => {
                let paths = vec![
                    "/Library/Application Support/Bitcoin/bitcoin.conf",
                    "/.bitcoin/bitcoin.conf",
                    "\\AppData\\Roaming\\Bitcoin\\bitcoin.conf",
                ];
                for filename in paths {
                    let full_path = format!("{}{}", path.display(), filename);
                    let f = File::open(full_path.clone());
                    match f {
                        Ok(mut f) => {
                            let mut contents = String::new();
                            f.read_to_string(&mut contents)
                                .expect("something went wrong reading the file");
                            println!("Found config file at {}", full_path);
                            let x = contents.split("\n");
                            for el in x {
                                let x = el.replace(" ", "");
                                if x.starts_with("rpcuser=") {
                                    username = Some(String::from(&x[8..]));
                                }
                                if x.starts_with("rpcpassword=") {
                                    password = Some(String::from(&x[12..]));
                                }
                                if x.starts_with("rpchost=") {
                                    host = Some(String::from(&x[8..]));
                                }
                            }
                        }
                        Err(_) => (),
                    }
                }
            }
            None => println!("Impossible to get your home dir!"),
        }

        match host.is_some() && username.is_some() {
            true => Ok( Config::new(host.unwrap(), username.unwrap(), password)),
            false => Err("Cannot find rpcuser and rpcpassword"),
        }
    }
}

