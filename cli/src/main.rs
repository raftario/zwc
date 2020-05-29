use std::io::Write;
use std::{
    io::{self, Read},
    process,
};
use structopt::StructOpt;

/// Hide and retrieve arbitrary data using zero-width characters
#[derive(StructOpt)]
#[structopt(author)]
enum Opt {
    /// Hides data inside a string
    #[structopt(alias = "c")]
    Camouflage {
        /// Dummy string to hide data in
        #[structopt(name = "DUMMY")]
        dummy: String,
        /// Payload to hide, read from standard input if not specified
        #[structopt(name = "PAYLOAD")]
        payload: Option<String>,
        /// Encryption key, data is not encrypted if not specified
        #[structopt(short = "k", long = "key", name = "KEY")]
        key: Option<String>,
        /// Brotli compression level of the payload, set to a sensible default if not specified
        #[structopt(short = "c", long = "compression-level", name = "LEVEL")]
        compression_level: Option<i32>,
    },
    /// Retrieves data from a string
    #[structopt(alias = "d")]
    Decamouflage {
        /// String containing hidden data, read from standard input if not specified
        #[structopt(name = "CAMOUFLAGED")]
        camouflaged: Option<String>,
        /// Decryption key, data is not decrypted if not specified
        #[structopt(short = "k", long = "key", name = "KEY")]
        key: Option<String>,
    },
}

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Camouflage {
            dummy,
            payload,
            compression_level,
            key,
        } => camouflage(dummy, payload, compression_level, key),
        Opt::Decamouflage { camouflaged, key } => decamouflage(camouflaged, key),
    }
}

fn camouflage(
    dummy: String,
    payload: Option<String>,
    compression_level: Option<i32>,
    key: Option<String>,
) {
    let payload = payload.map(|p| p.into_bytes()).unwrap_or_else(|| {
        let mut data = Vec::new();
        io::stdin()
            .lock()
            .read_to_end(&mut data)
            .unwrap_or_else(|e| {
                eprintln!("{}", e);
                process::exit(74)
            });
        data
    });

    match zwc::camouflage(
        payload,
        dummy.as_ref(),
        key.as_ref().map(AsRef::as_ref),
        compression_level,
    ) {
        Ok(camouflaged) => println!("{}", camouflaged),
        Err(e) => eprintln!("{}", e),
    }
}

fn decamouflage(camouflaged: Option<String>, key: Option<String>) {
    let camouflaged = camouflaged.unwrap_or_else(|| {
        let mut data = String::new();
        io::stdin()
            .lock()
            .read_to_string(&mut data)
            .unwrap_or_else(|e| {
                eprintln!("{}", e);
                process::exit(74)
            });
        data
    });

    match zwc::decamouflage(&camouflaged, key.as_ref().map(AsRef::as_ref)) {
        Ok(payload) => io::stdout().lock().write_all(&payload).unwrap(),
        Err(e) => eprintln!("{}", e),
    }
}
