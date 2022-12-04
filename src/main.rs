pub mod client;
pub mod nsconfig;
use chrono::Local;
use clap::Parser;
use client::{Client, ClientError, QueryAnswer, QueryType};
use env_logger::Env;
use futures::future::join_all;
use log;
use std::io::Write;
use tokio::task::JoinHandle;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Hostname to resolve
    hosts: Vec<String>,

    #[arg(short, long, default_value_t = String::from(""))]
    server: String,
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .init();

    let cli = Cli::parse();
    log::debug!("It will resolve {:?}", cli.hosts);
    let mut tasks: Vec<JoinHandle<Result<Vec<QueryAnswer>, ClientError>>> =
        Vec::with_capacity(cli.hosts.len());

    let mut server: String = if cli.server.len() > 0 {
        cli.server
    } else {
        match nsconfig::read_nameservers("/etc/resolv.conf".to_string()) {
            Err(err) => return Err(ClientError::GenericError(err.to_string())),
            Ok(vec) => {
                if vec.len() > 0 {
                    vec.get(0).unwrap().clone()
                } else {
                    "8.8.8.8".to_string()
                }
            }
        }
    };
    if !server.ends_with(":53") {
        server.push_str(":53");
    }

    for host in cli.hosts {
        let h = host.clone();
        tasks.push(tokio::spawn(async move {
            let client = match Client::new("192.168.15.1:53".to_string()).await {
                Ok(client) => client,
                Err(err) => return Err(err),
            };
            match client.query(host, QueryType::AAAA).await {
                Ok(res) => Ok(res),
                Err(err) => return Err(err),
            }
        }));
        tasks.push(tokio::spawn(async move {
            let client = match Client::new("192.168.15.1:53".to_string()).await {
                Ok(client) => client,
                Err(err) => return Err(err),
            };
            match client.query(h, QueryType::A).await {
                Ok(res) => Ok(res),
                Err(err) => return Err(err),
            }
        }));
    }
    let joined = join_all(tasks).await;
    println!("Joined {:?}", joined);
    Ok(())
}
