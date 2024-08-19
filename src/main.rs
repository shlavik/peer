use clap::Parser;

/// A simple P2P CLI demo application
#[derive(Debug, Parser)]
#[command(name = "Peer", version, about)]
struct Args {
    /// Sets the messaging period
    #[arg(short = 'r', long, value_name = "SECONDS", default_value = "1")]
    period: u64,

    /// Sets the peer port
    #[arg(short, long)]
    port: u16,

    /// Sets the address of the remote peer
    #[arg(short, long, value_name = "HOST:PORT")]
    connect: Option<String>,
}

fn main() {
    let args = Args::parse();

    println!("Period: {:?}", args.period);
    println!("Port: {}", args.port);
    println!("Connect: {:?}", args.connect);
}
