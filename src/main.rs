use clap::Parser;
use ethers_core::{types::*, utils::format_ether};
use ethers_providers::{Http, Middleware, Provider};
use serde::Serialize;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

/// Program arguments.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Network to fetch balance.
    #[arg(short, long, default_value_t = Chain::Mainnet)]
    network: Chain,

    /// Input file path.
    #[arg(short, long, default_value_t = String::from("data.inp"))]
    input: String,

    /// Output file path.
    #[arg(short, long, default_value_t = String::from("data.out"))]
    output: String,
}

#[derive(Debug, Serialize)]
struct AddrBal {
    address: Address,
    balance: f64,
}

trait ChainExt {
    fn get_provider(&self) -> Provider<Http>;
}

impl ChainExt for Chain {
    fn get_provider(&self) -> Provider<Http> {
        let rpc = match self {
            Chain::Mainnet => "https://rpc.ankr.com/eth",
            Chain::Mantle => "https://rpc.mantle.xyz",
            _ => unimplemented!("Network {self} is unsupported."),
        };
        Provider::<Http>::try_from(rpc).unwrap_or_else(|_| {
            panic!("Cannot create provider for this network ({self}) from this RPC endpoint {rpc}.")
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let node = args.network.get_provider();
    let input = File::open(args.input)?;
    let buf = BufReader::new(input);
    let mut wtr = csv::Writer::from_path(args.output)?;

    let mut total = U256::zero();

    for line in BufRead::lines(buf) {
        let addr: Address = line?.parse()?;
        let curr = node.get_balance(addr, None).await?;

        let record = AddrBal {
            address: addr,
            balance: format_ether(curr).parse::<f64>()?,
        };
        wtr.serialize(record)?;

        total += curr
    }

    println!("Total: {}", format_ether(total).parse::<f64>()?);

    wtr.flush()?;
    anyhow::Ok(())
}
