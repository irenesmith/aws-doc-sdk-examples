/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use dynamodb::model::{
    AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
};

/// Lists your Amazon DynamoDB tables and creates the table __test-table__.
#[tokio::main]
async fn main() -> Result<(), dynamodb::Error> {
    let client = dynamodb::Client::from_env();
    let tables = client.list_tables().send().await?;

    println!("Current DynamoDB tables: {:?}", tables);

    let new_table = client
        .create_table()
        .table_name("test-table")
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("k")
                .key_type(KeyType::Hash)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("k")
                .attribute_type(ScalarAttributeType::S)
                .build(),
        )
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .write_capacity_units(10)
                .read_capacity_units(10)
                .build(),
        )
        .send()
        .await?;
    println!(
        "New table: {:#?}",
        &new_table.table_description.unwrap().table_arn.unwrap()
    );
    Ok(())
}
