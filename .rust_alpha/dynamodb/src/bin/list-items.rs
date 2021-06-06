/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::process;

use dynamodb::{Client, Config, Region};

use aws_types::region::{ProvideRegion};

use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// The name of the table.
    #[structopt(short, long)]
    table: String,

    /// Whether to display addtional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists the items in an Amazon DynamoDB table.
/// # Arguments
///
/// * `-t TABLE` - The name of the table.
/// * `[-d DEFAULT-REGION]` - The AWS Region containing the table.
///   If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() {
    let Opt {
        table,
        default_region,
        verbose,
    } = Opt::from_args();

    let region = default_region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    if verbose {
        println!("DynamoDB client version: {}\n", dynamodb::PKG_VERSION);
        println!("AWS Region:              {:?}", &region);
        println!("Table:                   {}", table);

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }

    let config = Config::builder().region(region).build();

    let client = Client::from_conf(config);

    let t = &table;

    match client.scan().table_name(t).send().await {
        Ok(resp) => {
            println!("Items in table {}:", table);

            let items = resp.items.unwrap_or_default();

            for item in items {
                println!("   {:?}", item);
            }
        }
        Err(e) => {
            println!("Got an error listing items:");
            println!("{}", e);
            process::exit(1);
        }
    };
}
