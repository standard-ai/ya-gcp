//! An example of publishing data to pubsub and streaming back the same data

use futures::StreamExt;
use structopt::StructOpt;

use ya_gcp::{pubsub, AuthFlow, ClientBuilder, ClientBuilderConfig, ServiceAccountAuth};

#[derive(Debug, StructOpt)]
struct Args {
    /// A path to the oauth service account key json file
    #[structopt(long)]
    service_account_key: std::path::PathBuf,

    /// The name of the pubsub project
    #[structopt(long)]
    project_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    println!("Building PubSub clients");

    let builder = ClientBuilder::new(ClientBuilderConfig::new().auth_flow(
        AuthFlow::ServiceAccount(ServiceAccountAuth::Path(args.service_account_key)),
    ))
    .await?;

    let pubsub_config = pubsub::PubSubConfig::default();

    // Prepare the publishing and subscribing clients

    let mut publisher = builder
        .build_pubsub_publisher(pubsub_config.clone())
        .await?;
    let mut subscriber = builder.build_pubsub_subscriber(pubsub_config).await?;

    // Make up topic and subscription names
    let topic_name = pubsub::ProjectTopicName::new(&args.project_name, "pubsub_stream_example");
    let subscription_name =
        pubsub::ProjectSubscriptionName::new(&args.project_name, "pubsub_stream_example");

    println!("Creating topic {}", &topic_name);

    publisher
        .create_topic(pubsub::api::Topic {
            name: topic_name.clone().into(),
            ..pubsub::api::Topic::default()
        })
        .await?;

    println!("Creating subscription {}", &subscription_name);

    subscriber
        .create_subscription(pubsub::api::Subscription {
            name: subscription_name.clone().into(),
            topic: topic_name.clone().into(),
            ..pubsub::api::Subscription::default()
        })
        .await?;

    println!("Publishing messages to topic");

    futures::stream::iter(0u32..100)
        .map(|i| pubsub::api::PubsubMessage {
            data: format!("message-{}", i).into(),
            ..pubsub::api::PubsubMessage::default()
        })
        .map(Ok)
        .forward(publisher.publish_topic_sink(topic_name.clone()))
        .await?;

    println!("Reading back published messages");

    let read_stream = subscriber.stream_subscription(
        subscription_name.clone(),
        pubsub::StreamSubscriptionConfig::default(),
    );
    futures::pin_mut!(read_stream);

    for i in 0u32..100 {
        let (ack_token, message) = read_stream
            .next()
            .await
            .ok_or("unexpected end of stream")??;

        ack_token.ack().await?;

        assert_eq!(message.data, format!("message-{}", i));
    }

    println!("All messages matched!");

    println!("Deleting subscription {}", &subscription_name);

    subscriber
        .delete_subscription(pubsub::api::DeleteSubscriptionRequest {
            subscription: subscription_name.into(),
        })
        .await?;

    println!("Deleting topic {}", &topic_name);

    publisher
        .delete_topic(pubsub::api::DeleteTopicRequest {
            topic: topic_name.into(),
        })
        .await?;

    println!("Done");

    Ok(())
}
