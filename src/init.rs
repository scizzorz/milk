use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
}


fn main() {
    let args = Cli::from_args();
    println!("milk-init");
}
