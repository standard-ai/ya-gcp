use futures::{pin_mut, StreamExt, TryFutureExt, TryStreamExt};
use std::{
    collections::BTreeMap,
    future::Future,
    task::{Context, Poll},
    time::Duration,
};
use ya_gcp::pubsub::{
    self, api::PubsubMessage, emulator::Emulator, ProjectSubscriptionName, ProjectTopicName,
    PublisherClient, SinkError, StreamSubscriptionConfig,
};

/// Helper to create a new topic request.
async fn create_dummy_topic<C>(
    client: &mut PublisherClient<C>,
    project_name: &str,
    topic: &str,
) -> Result<tonic::Response<pubsub::api::Topic>, tonic::Status>
where
    C: tower::Service<http::Uri> + Clone + Send + Sync + 'static,
    C::Response: hyper::client::connect::Connection
        + tokio::io::AsyncRead
        + tokio::io::AsyncWrite
        + Send
        + Unpin
        + 'static,
    C::Future: Send + Unpin + 'static,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let request = pubsub::api::Topic {
        name: format!("projects/{}/topics/{}", project_name, topic),
        ..pubsub::api::Topic::default()
    };

    client.create_topic(request).await
}

async fn create_dummy_subscription(
    client: &mut pubsub::api::subscriber_client::SubscriberClient<
        impl tonic::client::GrpcService<
            tonic::body::BoxBody,
            Error = ya_gcp::auth::grpc::AuthGrpcError<tonic::transport::Error, yup_oauth2::Error>,
            ResponseBody = tonic::transport::Body,
        >,
    >,
    project_name: &str,
    subscription_name: &str,
    topic_name: &str,
) -> Result<tonic::Response<pubsub::api::Subscription>, tonic::Status> {
    let request = pubsub::api::Subscription {
        name: ProjectSubscriptionName::new(project_name, subscription_name).into(),
        topic: ProjectTopicName::new(project_name, topic_name).into(),
        ..pubsub::api::Subscription::default()
    };

    client.create_subscription(request).await
}

async fn publish_data<C>(
    client: &mut PublisherClient<C>,
    project_name: &str,
    topic_name: &str,
    messages: impl IntoIterator<Item = (Vec<u8>, BTreeMap<String, String>)>,
) -> Result<Vec<pubsub::api::PubsubMessage>, tonic::Status>
where
    C: tower::Service<http::Uri> + Clone + Send + Sync + 'static,
    C::Response: hyper::client::connect::Connection
        + tokio::io::AsyncRead
        + tokio::io::AsyncWrite
        + Send
        + Unpin
        + 'static,
    C::Future: Send + Unpin + 'static,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let messages = messages
        .into_iter()
        .map(|(data, attributes)| pubsub::api::PubsubMessage {
            data: ::prost::bytes::Bytes::from(data),
            attributes,
            message_id: Default::default(),
            publish_time: None,
            ordering_key: Default::default(),
        });

    let (tx, rx) = futures::channel::mpsc::unbounded();
    let message_sink = client
        .publish_topic_sink(ProjectTopicName::new(project_name, topic_name))
        .with_response_sink(tx);

    futures::stream::iter(messages.map(Ok))
        .forward(message_sink)
        .await
        .map_err(|err| match err {
            SinkError::Publish(err) => tonic::Status::from(err),
            SinkError::Response(_) => panic!("channel send shouldn't fail"),
        })?;

    Ok(rx.collect().await)
}

#[tokio::test]
async fn build_emulator() {
    Emulator::new().project("test-project").await.unwrap();
}

#[tokio::test]
async fn build_publisher_client() {
    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());
    emulator
        .builder()
        .build_pubsub_publisher(config)
        .await
        .unwrap();
}

#[tokio::test]
async fn build_subscriber_client() {
    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());
    emulator
        .builder()
        .build_pubsub_subscriber(config)
        .await
        .unwrap();
}

#[tokio::test]
async fn create_topic() {
    let topic_name = "test-topic";
    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());
    let mut client = emulator
        .builder()
        .build_pubsub_publisher(config)
        .await
        .unwrap();

    assert_eq!(
        pubsub::api::Topic {
            name: format!("projects/{}/topics/{}", emulator.project(), topic_name),
            ..Default::default()
        },
        create_dummy_topic(&mut client, emulator.project(), topic_name)
            .await
            .unwrap()
            .into_inner()
    );
}

#[tokio::test]
async fn get_topic() {
    let topic_name = "test-topic";
    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());
    let mut client = emulator
        .builder()
        .build_pubsub_publisher(config)
        .await
        .unwrap();

    create_dummy_topic(&mut client, emulator.project(), topic_name)
        .await
        .unwrap();

    let request = pubsub::api::GetTopicRequest {
        topic: format!("projects/{}/topics/{}", emulator.project(), topic_name),
    };

    assert_eq!(
        pubsub::api::Topic {
            name: format!("projects/{}/topics/{}", emulator.project(), topic_name),
            ..Default::default()
        },
        client.get_topic(request).await.unwrap().into_inner()
    );
}

#[tokio::test]
// Publish a single list of messages.
async fn publish() {
    let topic_name = "test-topic";
    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());

    let mut client = emulator
        .builder()
        .build_pubsub_publisher(config)
        .await
        .unwrap();

    create_dummy_topic(&mut client, emulator.project(), topic_name)
        .await
        .unwrap();

    publish_data(
        &mut client,
        emulator.project(),
        topic_name,
        vec![(vec![0u8, 100], BTreeMap::default()); 100],
    )
    .await
    .unwrap();

    let mut attributes = BTreeMap::default();
    attributes.insert("test".into(), "test".into());
    publish_data(
        &mut client,
        emulator.project(),
        topic_name,
        vec![(vec![], attributes); 100],
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn list_topic_subscriptions() {
    let topic_name = "test-topic";
    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());
    let mut client = emulator
        .builder()
        .build_pubsub_publisher(config)
        .await
        .unwrap();

    create_dummy_topic(&mut client, emulator.project(), topic_name)
        .await
        .unwrap();

    let request = pubsub::api::ListTopicSubscriptionsRequest {
        topic: format!("projects/{}/topics/{}", emulator.project(), topic_name),
        page_size: 10,
        page_token: Default::default(),
    };

    let response = client.list_topic_subscriptions(request).await;

    assert_eq!(
        pubsub::api::ListTopicSubscriptionsResponse {
            subscriptions: vec![],
            next_page_token: String::new(),
        },
        response.unwrap().into_inner()
    );
}

#[tokio::test]
async fn create_subscription() {
    let topic_name = "test-topic";
    let subscription_name = "test-subscription";
    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());

    let mut client = emulator
        .builder()
        .build_pubsub_publisher(config.clone())
        .await
        .unwrap();

    // Create a topic to query.
    create_dummy_topic(&mut client, emulator.project(), topic_name)
        .await
        .unwrap();

    let mut client = emulator
        .builder()
        .build_pubsub_subscriber(config.clone())
        .await
        .unwrap();

    create_dummy_subscription(
        &mut client,
        emulator.project(),
        subscription_name,
        topic_name,
    )
    .await
    .unwrap();
}

#[tokio::test]
// Pull a single value from a subscription using none-streaming api. If this fails, the
// streaming api would also fail.
async fn create_subscription_pull() {
    let topic_name = "test-topic";
    let subscription_name = "test-subscription";
    let num_messages = 100;

    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());
    let mut publish_client = emulator
        .builder()
        .build_pubsub_publisher(config.clone())
        .await
        .unwrap();

    // Create a topic to query.
    create_dummy_topic(&mut publish_client, emulator.project(), topic_name)
        .await
        .unwrap();

    let mut subscription_client = emulator
        .builder()
        .build_pubsub_subscriber(config.clone())
        .await
        .unwrap();
    // Create a subscription to query.
    let response = create_dummy_subscription(
        &mut subscription_client,
        emulator.project(),
        subscription_name,
        topic_name,
    )
    .await
    .unwrap()
    .into_inner();

    #[allow(deprecated)]
    let request = pubsub::api::PullRequest {
        subscription: response.name,
        return_immediately: false,
        max_messages: num_messages as i32,
    };

    let mut attributes = BTreeMap::default();
    attributes.insert("test".into(), "test".into());
    let message_ids = publish_data(
        &mut publish_client,
        emulator.project(),
        topic_name,
        vec![(vec![], attributes); num_messages],
    )
    .await
    .unwrap()
    .into_iter()
    .map(|message| message.message_id)
    .collect::<Vec<_>>();

    assert_eq!(message_ids.len(), num_messages);

    let response = subscription_client.pull(request).await;
    let mut received_messages = response.unwrap().into_inner().received_messages;

    let message = received_messages.pop().unwrap();
    let message_inner = message;

    assert!(message_ids.contains(&message_inner.message.unwrap().message_id));
}

#[tokio::test]
// Demonstrate that a stream will fail with an error if we have not created a subscription yet
// for the stream.
async fn create_subscription_stream_none_exists() {
    let subscription_name = "test-subscription";
    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());

    let mut client = emulator
        .builder()
        .build_pubsub_subscriber(config)
        .await
        .unwrap();

    let stream = client.stream_subscription(
        ProjectSubscriptionName::new(emulator.project(), subscription_name),
        StreamSubscriptionConfig::default(),
    );
    fn assert_stream<S: futures::Stream>(s: S) -> S {
        s
    }
    let stream = assert_stream(stream);
    pin_mut!(stream);

    // Should fail b/c we have not create a subscription with this name.
    let result = stream.as_mut().next().await.unwrap();
    assert!(matches!(result, Err(tonic::Status { .. })));
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
// Demonstrate that a stream will hang (not yield a value) if there are is no new data to pull
// from the stream.
async fn create_subscription_stream_empty() {
    let topic_name = "test-topic";
    let subscription_name = "test-subscription";
    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());
    let mut client = emulator
        .builder()
        .build_pubsub_publisher(config.clone())
        .await
        .unwrap();

    // Create a topic to query.
    create_dummy_topic(&mut client, emulator.project(), topic_name)
        .await
        .unwrap();

    let mut client = emulator
        .builder()
        .build_pubsub_subscriber(config.clone())
        .await
        .unwrap();
    // Create a subscription to query.
    create_dummy_subscription(
        &mut client,
        emulator.project(),
        subscription_name,
        topic_name,
    )
    .await
    .unwrap();

    let stream = client.stream_subscription(
        ProjectSubscriptionName::new(emulator.project(), subscription_name),
        StreamSubscriptionConfig::default(),
    );
    pin_mut!(stream);

    let result = tokio::time::timeout(std::time::Duration::from_millis(100), stream.next()).await;

    assert!(result.is_err());
    assert!(matches!(result, Err(tokio::time::error::Elapsed { .. })));
}

#[tokio::test]
// Demonstrate pulling one response from a stream.
async fn create_subscription_stream_one() {
    let topic_name = "test-topic";
    let subscription_name = "test-subscription";
    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());
    let mut publish_client = emulator
        .builder()
        .build_pubsub_publisher(config.clone())
        .await
        .unwrap();

    // Create a topic to query.
    create_dummy_topic(&mut publish_client, emulator.project(), topic_name)
        .await
        .unwrap();

    let mut subscription_client = emulator
        .builder()
        .build_pubsub_subscriber(config.clone())
        .await
        .unwrap();
    // Create a subscription to query.
    create_dummy_subscription(
        &mut subscription_client,
        emulator.project(),
        subscription_name,
        topic_name,
    )
    .await
    .unwrap();

    let mut attributes = BTreeMap::default();
    let data = (0..100).into_iter().collect::<Vec<u8>>();
    attributes.insert("test".into(), "test".into());
    let num_messages = 100;
    let write_messages = publish_data(
        &mut publish_client,
        emulator.project(),
        topic_name,
        vec![(data.clone(), attributes); num_messages],
    )
    .await
    .unwrap();

    let stream = subscription_client.stream_subscription(
        ProjectSubscriptionName::new(emulator.project(), subscription_name),
        StreamSubscriptionConfig::default(),
    );

    pin_mut!(stream);

    let read_messages = stream
        .take(num_messages)
        .map_ok(|(_ack_token, msg)| pubsub::api::PubsubMessage {
            publish_time: None,
            ..msg
        })
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    assert_eq!(read_messages, write_messages);
}

#[tokio::test]
// Demonstrate reading many responses from a stream.
async fn create_subscription_stream_many() {
    let topic_name = "test-topic";
    let subscription_name = "test-subscription";
    let num_messages = 100;
    let num_message_batches = 20;
    let total_messages = num_messages * num_message_batches;

    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());
    let mut publish_client = emulator
        .builder()
        .build_pubsub_publisher(config.clone())
        .await
        .unwrap();

    // Create a topic to query.
    create_dummy_topic(&mut publish_client, emulator.project(), topic_name)
        .await
        .unwrap();

    let mut subscription_client = emulator
        .builder()
        .build_pubsub_subscriber(config.clone())
        .await
        .unwrap();
    // Create a subscription to query.
    create_dummy_subscription(
        &mut subscription_client,
        emulator.project(),
        subscription_name,
        topic_name,
    )
    .await
    .unwrap();

    let mut published_message_ids = Vec::new();
    for _ in 0..num_message_batches {
        let mut attributes = BTreeMap::default();
        attributes.insert("test".into(), "test".into());
        let message_ids = publish_data(
            &mut publish_client,
            emulator.project(),
            topic_name,
            vec![(vec![], attributes); num_messages],
        )
        .await
        .unwrap()
        .into_iter()
        .map(|message| message.message_id)
        .collect::<Vec<_>>();
        published_message_ids.extend(message_ids);
    }

    let stream = subscription_client
        .stream_subscription(
            ProjectSubscriptionName::new(emulator.project(), subscription_name),
            StreamSubscriptionConfig::default(),
        )
        .take(total_messages);

    let collected_messages = stream.try_collect().await;
    let collected_messages: Vec<(_, PubsubMessage)> = collected_messages.unwrap();

    for (_ack_token, response_message) in collected_messages {
        assert!(published_message_ids.contains(&response_message.message_id));
    }
}

#[tokio::test]
// Demonstrate that if we end a stream and recreate it with the same client ID, the state is
// preserved and we pick up right where we left off.
async fn create_subscription_stream_restart_stream() {
    let topic_name = "test-topic";
    let subscription_name = "test-subscription";
    let num_messages = 100;
    let num_message_batches = 20;
    // We will drop the stream after consuming 1 message. Meaning num_messages will be
    // lost. Since we ack these right away, we can't get them back.
    let total_messages = num_messages * (num_message_batches - 1);

    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());
    let mut publish_client = emulator
        .builder()
        .build_pubsub_publisher(config.clone())
        .await
        .unwrap();

    // Create a topic to query.
    create_dummy_topic(&mut publish_client, emulator.project(), topic_name)
        .await
        .unwrap();

    let mut subscription_client = emulator
        .builder()
        .build_pubsub_subscriber(config.clone())
        .await
        .unwrap();
    // Create a subscription to query.
    create_dummy_subscription(
        &mut subscription_client,
        emulator.project(),
        subscription_name,
        topic_name,
    )
    .await
    .unwrap();

    let mut published_message_ids = Vec::new();
    for _ in 0..num_message_batches {
        let mut attributes = BTreeMap::default();
        attributes.insert("test".into(), "test".into());
        let message_ids = publish_data(
            &mut publish_client,
            emulator.project(),
            topic_name,
            vec![(vec![], attributes); num_messages],
        )
        .await
        .unwrap()
        .into_iter()
        .map(|message| message.message_id)
        .collect::<Vec<_>>();
        published_message_ids.extend(message_ids);
    }

    // Create stream and consume 1 message from it before closing.
    {
        let stream = subscription_client.stream_subscription(
            ProjectSubscriptionName::new(emulator.project(), subscription_name),
            StreamSubscriptionConfig::default(),
        );

        pin_mut!(stream);

        stream.next().await.unwrap().unwrap();
    }

    let stream = subscription_client
        .stream_subscription(
            ProjectSubscriptionName::new(emulator.project(), subscription_name),
            StreamSubscriptionConfig::default(),
        )
        .take(total_messages);

    let collected_messages = stream.try_collect().await;
    let collected_messages: Vec<(_, PubsubMessage)> = collected_messages.unwrap();

    for (_ack_token, response_message) in collected_messages {
        assert!(published_message_ids.contains(&response_message.message_id));
    }
}

#[tokio::test]
// Demonstrate publishing in one stream to pubsub and subscribing in another.
async fn async_publish_subscribe() {
    let topic_name = "test-topic";
    let subscription_name = "test-subscription";
    let project_name = "test-project";
    let num_messages = 100;
    let outer_step: usize = 5;
    let inner_step: usize = 10;
    let batches = inner_step * outer_step;
    let total_messages = num_messages * batches;

    let emulator = Emulator::new().project(project_name).await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());

    let mut publish_client = emulator
        .builder()
        .build_pubsub_publisher(config.clone())
        .await
        .unwrap();

    // Create a topic to query.
    create_dummy_topic(&mut publish_client, project_name, topic_name)
        .await
        .unwrap();

    let mut subscription_client = emulator
        .builder()
        .build_pubsub_subscriber(config.clone())
        .await
        .unwrap();
    // Create a subscription to query. Must be created before messages are published.
    create_dummy_subscription(
        &mut subscription_client,
        project_name,
        subscription_name,
        topic_name,
    )
    .await
    .unwrap();

    let subscription_stream = subscription_client
        .stream_subscription(
            ProjectSubscriptionName::new(emulator.project(), subscription_name),
            StreamSubscriptionConfig::default(),
        )
        .take(total_messages)
        .map_err(Box::<dyn std::error::Error>::from)
        .and_then(|(ack_token, message)| async move {
            ack_token.ack().await?;
            Ok(message.message_id)
        });

    let publish_stream = async_stream::stream! {
        for _ in 0..outer_step {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            for _ in 0..inner_step {
                let mut attributes = BTreeMap::default();
                attributes.insert("test".into(), "test".into());
                let message_ids = publish_data(
                        &mut publish_client,
                        project_name,
                        topic_name,
                        vec![(vec![], attributes); num_messages]
                    ).map_ok(|result| {
                        result
                            .into_iter()
                            .map(|message| message.message_id)
                            .collect::<Vec<_>>()
                    }).map_err(Box::<dyn std::error::Error>::from).await;

                yield message_ids;
            }
        }
    };

    // Convert streams to Future<Vec<_>>. Now we can start both streams and wait for them both
    // to finish and collect the results.
    let (publish_results, subscription_results) = futures::future::join(
        publish_stream.try_collect::<Vec<_>>(),
        subscription_stream.try_collect::<Vec<_>>(),
    )
    .await;

    // Flatten the results.
    let publish_results: Vec<String> = publish_results.unwrap().into_iter().flatten().collect();
    let subscription_results: Vec<String> = subscription_results.unwrap().into_iter().collect();

    assert_eq!(publish_results.len(), total_messages);
    assert_eq!(subscription_results.len(), total_messages);

    // The message_ids returned from publishing the messages should also be returned when the
    // subscriber receives the messages.
    for subscription_result in subscription_results {
        assert!(publish_results.contains(&subscription_result));
    }
}

/// A nack'ed message should be redelivered
#[tokio::test]
async fn nack_redeliver() {
    let topic_name = "test-topic";
    let subscription_name = "test-subscription";

    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());

    let mut publish_client = emulator
        .builder()
        .build_pubsub_publisher(config.clone())
        .await
        .unwrap();

    // Create a topic to query.
    let topic = create_dummy_topic(&mut publish_client, emulator.project(), topic_name)
        .await
        .unwrap()
        .into_inner();

    let mut subscription_client = emulator
        .builder()
        .build_pubsub_subscriber(config.clone())
        .await
        .unwrap();

    // Create a subscription to query. Must be created before messages are published.
    let subscription = create_dummy_subscription(
        &mut subscription_client,
        emulator.project(),
        subscription_name,
        topic_name,
    )
    .await
    .unwrap()
    .into_inner();

    // update the subscription with a DeadLetterPolicy so that the delivery counter engages
    subscription_client
        .update_subscription(pubsub::api::UpdateSubscriptionRequest {
            subscription: Some(pubsub::api::Subscription {
                dead_letter_policy: Some(pubsub::api::DeadLetterPolicy {
                    dead_letter_topic: format!(
                        "projects/{}/topics/{}",
                        emulator.project(),
                        topic_name
                    ),
                    max_delivery_attempts: 5,
                }),
                ..subscription
            }),
            update_mask: Some(pubsub::api::FieldMask {
                paths: vec!["dead_letter_policy".into()],
            }),
        })
        .await
        .unwrap();

    // send 1 message to the publisher. this should be delivered again after a nack
    publish_client
        .publish(pubsub::api::PublishRequest {
            topic: topic.name,
            messages: vec![pubsub::api::PubsubMessage {
                data: "foobar".into(),
                ..pubsub::api::PubsubMessage::default()
            }],
        })
        .await
        .unwrap();

    let mut subscription_stream = subscription_client.stream_subscription(
        ProjectSubscriptionName::new(emulator.project(), subscription_name),
        StreamSubscriptionConfig {
            // set a large general deadline to prevent inadvertent delivery
            stream_ack_deadline: Duration::from_secs(600),
            ..StreamSubscriptionConfig::default()
        },
    );

    let (ack_token, message) = subscription_stream.next().await.unwrap().unwrap();

    assert_eq!(message.data, "foobar");
    assert_eq!(ack_token.delivery_attempt(), 1);

    // there should be no other messages available
    assert!(futures::poll!(subscription_stream.next()).is_pending());

    // nack the message to permit re-delivery
    assert_eq!(Ok(()), ack_token.nack().await);

    let (ack_token, message) = subscription_stream.next().await.unwrap().unwrap();
    // get the same message again
    assert_eq!(message.data, "foobar");
    assert_eq!(ack_token.delivery_attempt(), 2);
}

/// An extended message should be redelivered after its modified deadline
#[tokio::test]
async fn modify_deadline_redeliver() {
    let topic_name = "test-topic";
    let subscription_name = "test-subscription";

    let emulator = Emulator::new().project("test-project").await.unwrap();
    let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());

    let mut publish_client = emulator
        .builder()
        .build_pubsub_publisher(config.clone())
        .await
        .unwrap();

    // Create a topic to query.
    let topic = create_dummy_topic(&mut publish_client, emulator.project(), topic_name)
        .await
        .unwrap()
        .into_inner();

    let mut subscription_client = emulator
        .builder()
        .build_pubsub_subscriber(config.clone())
        .await
        .unwrap();

    // Create a subscription to query. Must be created before messages are published.
    let subscription = create_dummy_subscription(
        &mut subscription_client,
        emulator.project(),
        subscription_name,
        topic_name,
    )
    .await
    .unwrap()
    .into_inner();

    // update the subscription with a DeadLetterPolicy so that the delivery counter engages
    subscription_client
        .update_subscription(pubsub::api::UpdateSubscriptionRequest {
            subscription: Some(pubsub::api::Subscription {
                dead_letter_policy: Some(pubsub::api::DeadLetterPolicy {
                    dead_letter_topic: format!(
                        "projects/{}/topics/{}",
                        emulator.project(),
                        topic_name
                    ),
                    max_delivery_attempts: 5,
                }),
                ..subscription
            }),
            update_mask: Some(pubsub::api::FieldMask {
                paths: vec!["dead_letter_policy".into()],
            }),
        })
        .await
        .unwrap();

    // send 1 message to the publisher. this should be delivered again after its deadline
    publish_client
        .publish(pubsub::api::PublishRequest {
            topic: topic.name,
            messages: vec![pubsub::api::PubsubMessage {
                data: "foobar".into(),
                ..pubsub::api::PubsubMessage::default()
            }],
        })
        .await
        .unwrap();

    let mut subscription_stream = subscription_client.stream_subscription(
        ProjectSubscriptionName::new(emulator.project(), subscription_name),
        StreamSubscriptionConfig {
            // set a large general deadline to prevent inadvertent delivery
            stream_ack_deadline: Duration::from_secs(600),
            ..StreamSubscriptionConfig::default()
        },
    );

    let (mut ack_token, message) = subscription_stream.next().await.unwrap().unwrap();

    assert_eq!(message.data, "foobar");
    assert_eq!(ack_token.delivery_attempt(), 1);

    // there should be no other messages available
    assert!(futures::poll!(subscription_stream.next()).is_pending());

    // modify the deadline to permit re-delivery
    assert_eq!(Ok(()), ack_token.modify_deadline(1).await);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ack_token, message) = subscription_stream.next().await.unwrap().unwrap();
    // get the same message again
    assert_eq!(message.data, "foobar");
    assert_eq!(ack_token.delivery_attempt(), 2);
}

/// 1) Modifying a deadline with too great a deadline should be an error
/// 2) Ack'ing a message should error after the request stream is dropped
#[test]
fn ack_errors() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Interestingly, it appears that tonic spawns the request stream onto the runtime in some
    // way, and that dropping the output stream or even the subscriber client does not drop
    // the request stream. Instead, we have to drop the entire runtime in order to drop the
    // background task which holds the ack request stream

    let mut ack_token = runtime.block_on(async {
        let topic_name = "test-topic";
        let subscription_name = "test-subscription";

        let emulator = Emulator::new().project("test-project").await.unwrap();
        let config = pubsub::PubSubConfig::new().endpoint(emulator.endpoint());

        let mut publish_client = emulator
            .builder()
            .build_pubsub_publisher(config.clone())
            .await
            .unwrap();

        // Create a topic to query.
        let topic = create_dummy_topic(&mut publish_client, emulator.project(), topic_name)
            .await
            .unwrap()
            .into_inner();

        let mut subscription_client = emulator
            .builder()
            .build_pubsub_subscriber(config.clone())
            .await
            .unwrap();

        // Create a subscription to query. Must be created before messages are published.
        let _subscription = create_dummy_subscription(
            &mut subscription_client,
            emulator.project(),
            subscription_name,
            topic_name,
        )
        .await
        .unwrap()
        .into_inner();

        // send 1 message to the publisher to get an ack token
        publish_client
            .publish(pubsub::api::PublishRequest {
                topic: topic.name,
                messages: vec![pubsub::api::PubsubMessage {
                    data: "foobar".into(),
                    ..pubsub::api::PubsubMessage::default()
                }],
            })
            .await
            .unwrap();

        let subscription_stream = subscription_client.stream_subscription(
            ProjectSubscriptionName::new(emulator.project(), subscription_name),
            StreamSubscriptionConfig::default(),
        );

        pin_mut!(subscription_stream);

        let (ack_token, _message) = subscription_stream.next().await.unwrap().unwrap();
        ack_token
    });

    let mut cx = Context::from_waker(futures::task::noop_waker_ref());

    // just a parameter test while we have an ack token
    assert_eq!(
        Box::pin(ack_token.modify_deadline(601))
            .as_mut()
            .poll(&mut cx),
        Poll::Ready(Err(pubsub::ModifyAcknowledgeError::InvalidDeadline {
            seconds: 601
        }))
    );

    // invariant check that ack tokens still work while the runtime is alive
    assert_eq!(
        Box::pin(ack_token.modify_deadline(599))
            .as_mut()
            .poll(&mut cx),
        Poll::Ready(Ok(()))
    );

    std::mem::drop(runtime);

    // now modifications or acks/nacks will fail

    assert!(matches!(
        Box::pin(ack_token.modify_deadline(599))
            .as_mut()
            .poll(&mut cx),
        Poll::Ready(Err(pubsub::ModifyAcknowledgeError::Modify(
            pubsub::AcknowledgeError { .. }
        )))
    ));

    assert!(matches!(
        Box::pin(ack_token.ack()).as_mut().poll(&mut cx),
        Poll::Ready(Err(pubsub::AcknowledgeError { .. }))
    ));
}
