//! An example of publishing data to pubsub and streaming back the same data

use futures::StreamExt;
use structopt::StructOpt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

use ya_gcp::{pubsub, AuthFlow, ClientBuilder, ClientBuilderConfig, ServiceAccountAuth};

#[derive(Debug, StructOpt)]
struct Args {
    /// A path to the oauth service account key json file
    #[structopt(long)]
    service_account_key: std::path::PathBuf,

    /// The name of the pubsub project
    #[structopt(long)]
    project_name: String,

    /// The topic to create
    #[structopt(long)]
    topic: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .init();

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
    let topic_name = pubsub::ProjectTopicName::new(&args.project_name, &args.topic);
    let subscription_name = pubsub::ProjectSubscriptionName::new(&args.project_name, &args.topic);

    println!("Creating topic {}", &topic_name);

    publisher
        .raw_api_mut()
        .create_topic({
            let mut t = pubsub::api::Topic::default();
            t.name = topic_name.clone().into();
            t
        })
        .await?;

    println!("Creating subscription {}", &subscription_name);

    subscriber
        .raw_api_mut()
        .create_subscription({
            let mut s = pubsub::api::Subscription::default();
            s.name = subscription_name.clone().into();
            s.topic = topic_name.clone().into();
            s
        })
        .await?;

    println!("Publishing messages to topic");

    futures::stream::iter(0u32..15)
        .map(|i| {
            let mut m = pubsub::api::PubsubMessage::default();
            let payload = format!("message-{:02}", i);
            println!("Sending `{payload}`");
            m.data = payload.into();
            m
        })
        .map(Ok)
        .forward(publisher.publish_topic_sink(topic_name.clone(), pubsub::PublishConfig::default()))
        .await?;

    println!("Reading back published messages");

    let read_stream = subscriber.stream_subscription(
        subscription_name.clone(),
        pubsub::StreamSubscriptionConfig::default(),
    );
    futures::pin_mut!(read_stream);

    let mut messages = Vec::new();
    for _ in 0u32..15 {
        let (ack_token, message) = read_stream
            .next()
            .await
            .ok_or("unexpected end of stream")??;

        let payload = std::str::from_utf8(&message.data[..]).unwrap();
        println!("Received `{payload}`");
        messages.push(payload.to_owned());

        ack_token.ack().await?;
    }

    messages.sort();
    assert_eq!(
        messages,
        (0..15)
            .map(|i| format!("message-{:02}", i))
            .collect::<Vec<_>>()
    );

    println!("All messages matched!");

    println!("Deleting subscription {}", &subscription_name);

    subscriber
        .raw_api_mut()
        .delete_subscription({
            let mut d = pubsub::api::DeleteSubscriptionRequest::default();
            d.subscription = subscription_name.into();
            d
        })
        .await?;

    println!("Deleting topic {}", &topic_name);

    publisher
        .raw_api_mut()
        .delete_topic({
            let mut d = pubsub::api::DeleteTopicRequest::default();
            d.topic = topic_name.into();
            d
        })
        .await?;

    println!("Done");

    Ok(())
}
