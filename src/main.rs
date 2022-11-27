pub mod client;
use chrono::Local;
use clap::Parser;
use client::{Client, ClientError, QueryType};
use env_logger::Env;
use log;
use std::io::Write;
use tokio::task::JoinHandle;
use futures::future::join_all;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Hostname to resolve
    hosts: Vec<String>,
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
    let mut tasks: Vec<JoinHandle<Result<String, ClientError>>> =
        Vec::with_capacity(cli.hosts.len());
    for host in cli.hosts {
        let h = host.clone();
        tasks.push(tokio::spawn(async move {
            let client = match Client::new("192.168.15.1:53".to_string()).await {
                Ok(client) => client,
                Err(err) => return Err(err),
            };
            match client
                .query(host, QueryType::AAAA)
                .await
            {
                Ok(res) => Ok(res),
                Err(err) => return Err(err),
            }
        }));
        tasks.push(tokio::spawn(async move {
            let client = match Client::new("192.168.15.1:53".to_string()).await {
                Ok(client) => client,
                Err(err) => return Err(err),
            };
            match client
                .query(h, QueryType::A)
                .await
            {
                Ok(res) => Ok(res),
                Err(err) => return Err(err),
            }
        }));
    }
    let joined = join_all(tasks).await;
    println!("Joined {:?}", joined);

    Ok(())
}
