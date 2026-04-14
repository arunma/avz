mod cli;
mod commands;
mod convert;
mod error;
mod io;

use aws_sdk_s3::Client as S3Client;
use clap::Parser;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> error::Result<()> {
    let s3_client = init_s3_if_needed(&cli.command).await;

    match cli.command {
        Commands::Cat { files, pretty, head } => {
            commands::cat::execute(&files.files, &s3_client, pretty, head).await
        }
        Commands::Head { files, count, pretty } => {
            commands::head::execute(&files.files, &s3_client, count, pretty).await
        }
        Commands::Schema { files, .. } => {
            commands::schema::execute(&files.files, &s3_client).await
        }
        Commands::Count { files } => {
            commands::count::execute(&files.files, &s3_client).await
        }
        Commands::Meta { files } => {
            commands::meta::execute(&files.files, &s3_client).await
        }
        Commands::FromJson { schema, output, codec, input } => {
            commands::fromjson::execute(&schema, &output, &codec, input.as_deref()).await
        }
        Commands::Concat { files, output } => {
            commands::concat::execute(&files.files, &s3_client, &output).await
        }
        Commands::Recodec { files, codec, output } => {
            commands::recodec::execute(&files.files, &s3_client, &codec, &output).await
        }
        Commands::Fingerprint { files, algorithm } => {
            commands::fingerprint::execute(&files.files, &s3_client, &algorithm).await
        }
        Commands::Validate { files, reader_schema } => {
            commands::validate::execute(&files.files, &s3_client, reader_schema.as_deref()).await
        }
        Commands::Grep { pattern, files, pretty, ignore_case, invert, count, fixed_string } => {
            commands::grep::execute(&pattern, &files.files, &s3_client, pretty, ignore_case, invert, count, fixed_string).await
        }
        Commands::Random { schema, count, format, output, seed, pretty } => {
            commands::random::execute(&schema, count, &format, output.as_deref(), seed, pretty).await
        }
    }
}

async fn init_s3_if_needed(command: &Commands) -> Option<S3Client> {
    let has_s3 = match command {
        Commands::Cat { files, .. }
        | Commands::Head { files, .. }
        | Commands::Schema { files, .. }
        | Commands::Count { files }
        | Commands::Meta { files }
        | Commands::Concat { files, .. }
        | Commands::Recodec { files, .. }
        | Commands::Fingerprint { files, .. }
        | Commands::Validate { files, .. }
        | Commands::Grep { files, .. } => files.files.iter().any(|f| f.starts_with("s3://")),
        Commands::FromJson { .. } | Commands::Random { .. } => false,
    };

    if has_s3 {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        Some(S3Client::new(&config))
    } else {
        None
    }
}
