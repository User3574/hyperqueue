use clap::Parser;
use hyperqueue::server::server::create_server;
use hyperqueue::transfer::service::{init_tracing, WorldClient};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use tarpc::{client, context, tokio_serde::formats::Json};
use tokio::time::sleep;
use tracing::Instrument;

// Root CLI options
#[derive(Parser)]
#[clap(author, about, version(option_env!("HQ_BUILD_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[clap(global_setting = clap::AppSettings::DisableHelpSubcommand)]
#[clap(global_setting = clap::AppSettings::HelpExpected)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
struct HelloOpts {
    /// Sets the name to say hello to.
    #[clap(long)]
    name: String,
}

#[allow(clippy::large_enum_variant)]
#[derive(Parser)]
enum SubCommand {
    /// Create TarPC Server
    Create,
    /// Say Hello to TarPC Server
    Hello(HelloOpts),
    /// Say Bye to TarPC Server
    Bye,
}

async fn command_say_hello(flags: HelloOpts) -> anyhow::Result<()> {
    init_tracing("TARPC Client")?;

    // Localhost
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080);
    let transport = tarpc::serde_transport::tcp::connect(socket, Json::default);

    // World client comm
    let client = WorldClient::new(client::Config::default(), transport.await?).spawn();
    let hello = async move {
        tokio::select! {
            hello = client.hello(context::current(), format!("{}1", flags.name)) => { hello }
        }
    }
    .instrument(tracing::info_span!("Hello"))
    .await;

    // Show response
    println!("{:?}", hello);

    // Let the background span processor finish.
    sleep(Duration::from_micros(1)).await;
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

async fn command_say_bye() -> anyhow::Result<()> {
    init_tracing("TARPC Client")?;

    // Localhost
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080);
    let transport = tarpc::serde_transport::tcp::connect(socket, Json::default);

    // World client comm
    let client = WorldClient::new(client::Config::default(), transport.await?).spawn();
    let bye = async move {
        tokio::select! {
            bye = client.bye(context::current()) => { bye }
        }
    }
    .instrument(tracing::info_span!("Bye"))
    .await;

    // Show response
    println!("{:?}", bye);

    // Let the background span processor finish.
    sleep(Duration::from_micros(1)).await;
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

async fn command_create_server() -> anyhow::Result<()> {
    create_server().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let top_opts: Opts = Opts::parse();

    let result = match top_opts.subcmd {
        SubCommand::Create => command_create_server().await,
        SubCommand::Hello(opts) => command_say_hello(opts).await,
        SubCommand::Bye => command_say_bye().await,
    };

    if let Err(e) = result {
        println!("Error: {e}");
        std::process::exit(1);
    }
}
