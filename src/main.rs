use prometheus::{__register_gauge, register_gauge, Opts};
use prometheus_exporter::{FinishedUpdate, PrometheusExporter};
use std::error::Error;
use std::io::Read;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long = "bind-addr", default_value = "0.0.0.0:8010")]
    bind_addr: String,

    #[structopt(long = "socket-timeout", default_value = "5")]
    socket_timeout: u64,

    #[structopt(long = "server-addr")]
    server_addr: String,

    #[structopt(long = "refresh-interval", default_value = "30")]
    refresh_interval: u64,
}

fn to_socket_addr(addr: &str) -> Option<SocketAddr> {
    addr.to_socket_addrs().ok()?.next()
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args();
    env_logger::init();
    log::info!("Starting prometheus-teamspeak with {:?}", args);
    let bind_addr = to_socket_addr(&args.bind_addr)
        .expect(&format!("Invalid socket address: {}", args.bind_addr));

    let server_addr = to_socket_addr(&args.server_addr)
        .expect(&format!("Invalid socket address: {}", args.server_addr));

    let socket_timeout = Duration::from_secs(args.socket_timeout);
    let interval = Duration::from_secs(args.refresh_interval);

    let (request_receiver, finished_sender) =
        PrometheusExporter::run_and_repeat(bind_addr, interval);

    let opts = Opts::new("is_up", "whether the TeamSpeak server is up").namespace("teamspeak");
    let teamspeak_status_metric = register_gauge!(opts)?;

    let mut buffer = [0u8; 32];
    loop {
        request_receiver.recv().unwrap();

        log::info!("Updating metrics!");

        match TcpStream::connect_timeout(&server_addr, socket_timeout) {
            Ok(mut stream) => match stream.read_exact(&mut buffer) {
                Ok(_) => {
                    teamspeak_status_metric.set(1.0);
                }
                Err(e) => {
                    teamspeak_status_metric.set(0.0);
                    log::info!("Unable to reach server: {}", e);
                }
            },
            Err(e) => {
                teamspeak_status_metric.set(0.0);
                log::info!("Unable to reach server: {}", e);
            }
        }

        finished_sender.send(FinishedUpdate).unwrap();
    }
}
