use std::io::{self, Read};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(author)]
struct Opt {
    #[structopt(name = "DUMMY")]
    dummy: String,
    #[structopt(name = "PAYLOAD")]
    payload: Option<String>,
    #[structopt(short = "c", long = "compression-level", name = "LEVEL")]
    compression_level: Option<i32>,
    #[structopt(short = "k", long = "key", name = "KEY")]
    key: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    let payload = opt.payload.map(|p| p.into_bytes()).unwrap_or_else(|| {
        let mut p = Vec::new();
        io::stdin().lock().read_to_end(&mut p).unwrap();
        p
    });

    match zwc_encode::camouflage(
        payload,
        opt.dummy.as_ref(),
        opt.compression_level,
        opt.key.as_ref().map(AsRef::as_ref),
    ) {
        Ok(camouflaged) => println!("{}", camouflaged),
        Err(e) => eprintln!("{}", e),
    }
}
