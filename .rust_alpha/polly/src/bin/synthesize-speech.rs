/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */
use std::fs;
use std::process;

use polly::model::{OutputFormat, VoiceId};
use polly::{Client, Config, Region};

use aws_types::region::{ProvideRegion};

use bytes::Buf;
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// The file containing the text to synthesize.
    #[structopt(short, long)]
    filename: String,

    /// Whether to displaly additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Reads a text file and creates an MP3 file with the text synthesized into speech by Amazon Polly.
/// # Arguments
///
/// * `[-f FILENAME]` - The name of the file containing the text to synthesize.
///   The MP3 output is saved in a file with the same basename and a ".MP3" extension.
/// * `[-d DEFAULT-REGION]` - The AWS Region containing the voices.
///   If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() {
    let Opt {
        filename,
        default_region,
        verbose,
    } = Opt::from_args();

    let region = default_region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    if verbose {
        println!("polly client version: {}.\n", polly::PKG_VERSION);
        println!("AWS Region:           {:?}", &region);
        println!("Filename:             {}", filename);

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }

    let config = Config::builder().region(region).build();

    let client = Client::from_conf(config);

    let content = fs::read_to_string(&filename);

    let resp = match client
        .synthesize_speech()
        .output_format(OutputFormat::Mp3)
        .text(content.unwrap())
        .voice_id(VoiceId::Joanna)
        .send()
        .await
    {
        Ok(output) => output,
        Err(e) => {
            println!("Got an error synthesizing speech:");
            println!("{}", e);
            process::exit(1);
        }
    };

    // Get MP3 data from response and save it
    let mut blob = resp
        .audio_stream
        .collect()
        .await
        .expect("Failed to read data.");

    let parts: Vec<&str> = filename.split('.').collect();
    let out_file = format!("{}{}", String::from(parts[0]), ".mp3");

    let mut file = tokio::fs::File::create(out_file)
        .await
        .expect("Failed to create file.");
    while blob.has_remaining() {
        file.write_buf(&mut blob)
            .await
            .expect("Failed to write to file.");
    }
}
