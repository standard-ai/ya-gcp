use std::{
    convert::TryFrom,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use async_channel as mpmc;
use futures::{
    pin_mut,
    stream::{self, Stream},
    StreamExt,
};
use pin_project::pin_project;
use tonic::metadata::MetadataValue;
use tracing::{debug, trace_span, Instrument};

use crate::{
    auth::grpc::{AuthGrpcService, OAuthTokenSource},
    pubsub::{api, PubSubRetryCheck},
    retry_policy::{exponential_backoff, ExponentialBackoff, RetryOperation, RetryPolicy},
};

config_default! {
    /// Configuration for a [streaming subscription](super::SubscriberClient::stream_subscription)
    /// request
    #[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize)]
    pub struct StreamSubscriptionConfig {
        /// See [`StreamingPullRequest.stream_ack_deadline_seconds`](api::StreamingPullRequest::stream_ack_deadline_seconds)
        #[serde(with = "humantime_serde")]
        @default(Duration::from_secs(10), "StreamSubscriptionConfig::default_stream_ack_deadline")
        pub stream_ack_deadline: Duration,

        /// See [`StreamingPullRequest.max_outstanding_messages`](api::StreamingPullRequest::max_outstanding_messages)
        @default(1000, "StreamSubscriptionConfig::default_max_outstanding_messages")
        pub max_outstanding_messages: i64,

        /// See [`StreamingPullRequest.max_outstanding_bytes`](api::StreamingPullRequest::max_outstanding_bytes)
        @default(0, "StreamSubscriptionConfig::default_max_outstanding_bytes")
        pub max_outstanding_bytes: i64,

        /// Deprecated, subsumed by `max_outstanding_messages`
        #[deprecated]
        @default(0, "StreamSubscriptionConfig::default_ack_channel_capacity")
        pub ack_channel_capacity: usize,
    }
}

/// A user-initiated request to acknowledge, reject, or modify the deadline of some message
#[derive(Debug, PartialEq)]
enum UserAck {
    Ack { id: String },
    Modify { id: String, seconds: i32 },
}

/// An error encountered when issuing acks, nacks, or modifications from an
/// [`AcknowledgeToken`](AcknowledgeToken)
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
#[error("cannot ack/nack/modify because the stream was dropped")]
pub struct AcknowledgeError {
    _private: (),
}

impl AcknowledgeError {
    // SendErrors should only happen when the receiver is disconnected, which isn't called manually
    // by the stream so should only happen on drop
    fn from_send_err<T>(_err: mpmc::SendError<T>) -> Self {
        AcknowledgeError { _private: () }
    }
}

/// The maximum deadline supported by the RPC service
const MAX_DEADLINE_SEC: u32 = 600;

/// An error encountered when issuing deadline modifications with
/// [`AcknowledgeToken::modify_deadline`](AcknowledgeToken::modify_deadline)
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum ModifyAcknowledgeError {
    /// An error occurred in delivering the modification request
    #[error(transparent)]
    Modify(#[from] AcknowledgeError),

    /// The requested deadline was not within the permitted range
    #[error("deadline must be between 0 and {MAX_DEADLINE_SEC} seconds, given {seconds}")]
    InvalidDeadline {
        /// The number of seconds requested for the deadline
        seconds: u32,
    },
}

/// A token with an associated message produced by the [`StreamSubscription`](StreamSubscription)
/// stream, used to control that message's re-delivery within the message queue.
#[derive(Debug)]
pub struct AcknowledgeToken {
    id: String,
    channel: mpmc::Sender<UserAck>,
    delivery_attempt: i32,
}

impl AcknowledgeToken {
    /// Acknowledge the corresponding message as received, so that the message service will stop
    /// attempting to deliver it to subscribers.
    ///
    /// Note that acknowledgements may not arrive to the service within the deadline (or at all),
    /// so this is only a best-effort means of preventing re-delivery.
    pub async fn ack(self) -> Result<(), AcknowledgeError> {
        self.channel
            .send(UserAck::Ack { id: self.id })
            .await
            .map_err(AcknowledgeError::from_send_err)?;
        Ok(())
    }

    /// Negatively acknowledge the corresponding message, requesting that the message service
    /// re-deliver it to subscribers.
    ///
    /// This may be useful if the message consumer encounters an error while processing the
    /// message.
    pub async fn nack(self) -> Result<(), AcknowledgeError> {
        self.channel
            .send(UserAck::Modify {
                id: self.id,
                // "A NACK is any call to ModifyAckDeadline with a 0 deadline"
                // see ReceivedMessage or ModifyAckDeadlineRequest rpc docs
                seconds: 0,
            })
            .await
            .map_err(AcknowledgeError::from_send_err)?;

        Ok(())
    }

    /// Modify the acknowledgement deadline of the corresponding message, so that re-delivery is
    /// not attempted until the given time elapses (unless an ack/nack is sent).
    ///
    /// The message's deadline will be set to the given number of seconds after the service
    /// receives the modify request. Note that this could be shorter than the message's prior
    /// deadline, if for example the [subscription-level
    /// deadline](StreamSubscriptionConfig::stream_ack_deadline) is longer. A deadline of
    /// 0 seconds is equivalent to a [`nack`](AcknowledgeToken::nack) and will make the message
    /// immediately available for re-delivery.
    ///
    /// The maximum deadline accepted by the service is 600 seconds.
    pub async fn modify_deadline(&mut self, seconds: u32) -> Result<(), ModifyAcknowledgeError> {
        if seconds > MAX_DEADLINE_SEC {
            return Err(ModifyAcknowledgeError::InvalidDeadline { seconds });
        }

        self.channel
            .send(UserAck::Modify {
                id: self.id.clone(),
                seconds: i32::try_from(seconds).expect("deadline must fit in i32"),
            })
            .await
            .map_err(|err| ModifyAcknowledgeError::Modify(AcknowledgeError::from_send_err(err)))?;
        Ok(())
    }

    /// The approximate number of times that Cloud Pub/Sub has attempted to deliver the associated
    /// message to a subscriber.
    ///
    /// See [`delivery_attempt`](api::ReceivedMessage::delivery_attempt)
    pub fn delivery_attempt(&self) -> i32 {
        self.delivery_attempt
    }
}

/// Create the initial StreamingPullRequest which must include a Subscription string and a unique
/// client_id.
fn create_initial_streaming_pull_request(
    subscription: String,
    client_id: String,
    config: &StreamSubscriptionConfig,
) -> api::StreamingPullRequest {
    api::StreamingPullRequest {
        subscription,
        client_id,
        stream_ack_deadline_seconds: i32::try_from(config.stream_ack_deadline.as_secs())
            .expect("ack deadline seconds should fit in i32"),
        max_outstanding_messages: config.max_outstanding_messages,
        modify_deadline_seconds: Vec::default(),
        modify_deadline_ack_ids: Vec::default(),
        max_outstanding_bytes: config.max_outstanding_bytes,
        ack_ids: Vec::default(),
    }
}

/// Create the Nth StreamingPullRequest message (i.e. 2nd or later).
///
/// Message should contain the previous message ids we want to acknowledge, and message ids for
/// messages whose deadlines should be modified. Nack'ing a message is accomplished by setting its
/// deadline to 0.
fn create_subsequent_streaming_pull_request(
    ack_ids: Vec<String>,
    modify_deadline_seconds: Vec<i32>,
    modify_deadline_ack_ids: Vec<String>,
    config: &StreamSubscriptionConfig,
) -> api::StreamingPullRequest {
    api::StreamingPullRequest {
        ack_ids,
        // subscription was set on the first request and is not set again.
        subscription: String::default(),
        // client_id was set on the first request and is not set again.
        client_id: String::default(),
        // Even though this was set on the first request, it is reset on every subsequent request.
        // Here we "reset" it to the same value.
        stream_ack_deadline_seconds: i32::try_from(config.stream_ack_deadline.as_secs())
            .expect("ack deadline seconds should fit in i32"),
        // max_outstanding_messages was set on the first request and is not set again.
        max_outstanding_messages: 0,
        modify_deadline_seconds,
        modify_deadline_ack_ids,
        // max_outstanding_bytes was set on the first request and is not set again.
        max_outstanding_bytes: 0,
    }
}

/// Create never ending request stream of StreamingPullRequests. The first request initializes a new
/// gRPC stream and subsequent messages will acknowledge previous messages and request additional
/// ones.
fn create_streaming_pull_request_stream(
    subscription: String,
    client_id: String,
    user_acks: impl Stream<Item = UserAck>,
    config: StreamSubscriptionConfig,
) -> impl Stream<Item = api::StreamingPullRequest> {
    async_stream::stream! {
        // start by issuing the first request
        yield create_initial_streaming_pull_request(subscription, client_id, &config);

        // Subsequent requests come from user acknowledgements.
        // Pull multiple acks at once in order to batch acks in the request.
        // The size of batches is chosen to not exceed API restrictions
        const MAX_PER_REQUEST_CHANGES: usize = 1000; // 1000 as used in the java lib
        pin_mut!(user_acks);
        let mut user_ack_batches = user_acks.ready_chunks(MAX_PER_REQUEST_CHANGES);

        while let Some(user_ack_batch) = user_ack_batches.next().await {
            // collect the single stream of acks into individual lists depending on the ack type.
            // pre-alloc only the ack list as the anticipated normal outcome
            let mut acks = Vec::with_capacity(user_ack_batch.len());
            let mut deadlines = Vec::new();
            let mut deadline_acks = Vec::new();

            for user_ack in user_ack_batch {
                match user_ack {
                    UserAck::Ack { id } => acks.push(id),
                    UserAck::Modify { id, seconds } => {
                        deadlines.push(seconds);
                        deadline_acks.push(id);
                    }
                };
            }

            yield create_subsequent_streaming_pull_request(
                acks,
                deadlines,
                deadline_acks,
                &config
            );
        }
    }
}

/// The stream returned by the
/// [`stream_subscription`](crate::pubsub::SubscriberClient::stream_subscription) function
#[pin_project]
pub struct StreamSubscription<C = crate::DefaultConnector, R = ExponentialBackoff<PubSubRetryCheck>>
{
    state: StreamState<C, R>,
    // reserve the right to be !Unpin in the future without a major version bump
    _p: std::marker::PhantomPinned,
}

/// To allow some builder-like setup after the StreamSubscription is created but before it is used,
/// this enum holds the initialized state before switching to streaming.
///
/// Builder methods (e.g. [`StreamSubscription::with_retry_policy`]) should take `self` by value,
/// so that these options can only be changed before streaming actually begins. Any calls to
/// `poll_next` require `Pin<&mut Self>` which then prohibits moving `self`, so no builder methods
/// could be called after streaming
enum StreamState<C, R> {
    Initialized {
        client: api::subscriber_client::SubscriberClient<
            AuthGrpcService<tonic::transport::Channel, OAuthTokenSource<C>>,
        >,
        subscription: String,
        config: StreamSubscriptionConfig,
        retry_policy: R,
    },
    Transition,
    Streaming(
        // TODO(type_alias_impl_trait) don't box
        stream::BoxStream<'static, Result<(AcknowledgeToken, api::PubsubMessage), tonic::Status>>,
    ),
}

impl<C> StreamSubscription<C> {
    pub(super) fn new(
        client: api::subscriber_client::SubscriberClient<
            AuthGrpcService<tonic::transport::Channel, OAuthTokenSource<C>>,
        >,
        subscription: String,
        config: StreamSubscriptionConfig,
    ) -> Self {
        StreamSubscription {
            state: StreamState::Initialized {
                client,
                subscription: subscription,
                config,
                retry_policy: ExponentialBackoff::new(
                    PubSubRetryCheck::default(),
                    Self::default_retry_configuration(),
                ),
            },
            _p: std::marker::PhantomPinned,
        }
    }

    /// The default configuration values used for retrying connections to the PubSub streaming pull
    /// RPC
    pub fn default_retry_configuration() -> exponential_backoff::Config {
        // values pulled from java lib
        // https://github.com/googleapis/java-pubsub/blob/d969e8925edc3401e6eb534699ce0351a5f0b20b/google-cloud-pubsub/src/main/java/com/google/cloud/pubsub/v1/StreamingSubscriberConnection.java#L70
        exponential_backoff::Config {
            initial_interval: Duration::from_millis(100),
            max_interval: Duration::from_secs(10),
            multiplier: 2.0,
            ..Default::default()
        }
    }
}

impl<C, OldR> StreamSubscription<C, OldR> {
    /// Set the [`RetryPolicy`] to use for this streaming subscription.
    ///
    /// The stream will be reconnected if the policy indicates that an encountered
    /// [`Status`](tonic::Status) error should be retried
    // Because `poll_next` requires `Pin<&mut Self>`, this function cannot be called after the
    // stream has started because it moves `self`. That means that the retry policy can only be
    // changed before the polling starts, and is fixed from that point on
    pub fn with_retry_policy<R>(self, new_retry_policy: R) -> StreamSubscription<C, R>
    where
        R: RetryPolicy<(), tonic::Status>,
    {
        use StreamState::Initialized;
        StreamSubscription {
            state: match self.state {
                Initialized {
                    client,
                    subscription,
                    config,
                    retry_policy: _old,
                } => Initialized {
                    client,
                    subscription,
                    config,
                    retry_policy: new_retry_policy,
                },
                _ => unreachable!(
                    "state only changes in `poll_next`, which can't be called while `self` is \
                     movable"
                ),
            },
            _p: self._p,
        }
    }
}

impl<C, R> Stream for StreamSubscription<C, R>
where
    C: crate::Connect + Clone + Send + Sync + 'static,
    R: RetryPolicy<(), tonic::Status> + Send + 'static,
    R::RetryOp: Send + 'static,
    <R::RetryOp as RetryOperation<(), tonic::Status>>::Sleep: Send + 'static,
{
    type Item = Result<(AcknowledgeToken, api::PubsubMessage), tonic::Status>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        use StreamState::{Initialized, Streaming, Transition};

        let this = self.project();

        // switch between states using this loop + match + return
        loop {
            return match this.state {
                Initialized { .. } => {
                    // after checking the state in a borrow, get ownership by switching to the
                    // transition state
                    match std::mem::replace(this.state, Transition) {
                        Initialized {
                            client,
                            subscription,
                            config,
                            retry_policy,
                        } => {
                            *this.state = Streaming(Box::pin(stream_from_client(
                                client,
                                subscription,
                                config,
                                retry_policy,
                            )));
                            continue;
                        }
                        _ => unreachable!("just checked state"),
                    }
                }
                Transition => {
                    unreachable!("transition state should be transient and not witnessable")
                }
                Streaming(stream) => stream.poll_next_unpin(cx),
            };
        }
    }
}

/// Create a stream of PubSub results
///
/// The stream will internally reconnect on error if the given retry policy indicates the
/// error is retriable
fn stream_from_client<C, R>(
    mut client: api::subscriber_client::SubscriberClient<
        AuthGrpcService<tonic::transport::Channel, OAuthTokenSource<C>>,
    >,
    subscription: String,
    config: StreamSubscriptionConfig,
    mut retry_policy: R,
) -> impl Stream<Item = Result<(AcknowledgeToken, api::PubsubMessage), tonic::Status>> + Send + 'static
where
    C: crate::Connect + Clone + Send + Sync + 'static,
    R: RetryPolicy<(), tonic::Status> + Send + 'static,
    R::RetryOp: Send + 'static,
    <R::RetryOp as RetryOperation<(), tonic::Status>>::Sleep: Send + 'static,
{
    let subscription_meta =
        MetadataValue::from_str(&subscription).expect("valid subscription metadata");

    // the client id is used for stream reconnection on error
    let client_id = uuid::Uuid::new_v4().to_string();

    // Channel to send/receive message ids to ack.
    // This needs to be a multi-producer channel as each message will get a sender handle to send
    // its ack. In practice the receiver will only ever be polled as single-consumer; however there
    // is a period of time during reconnection when two receivers might exist, because the
    // disconnected stream is dropped in a background task at an unknown time (unknown to our
    // layer anyway), and the new receiver should continue to pull from the existing senders.
    let (sender, receiver) = mpmc::bounded(
        usize::try_from(config.max_outstanding_messages)
            .expect("outstanding messages should fit in usize"),
    );

    async_stream::stream! {
        let mut retry_op = None;

        'reconnect: loop {
            let request_stream = create_streaming_pull_request_stream(
                subscription.clone(),
                client_id.clone(),
                receiver.clone(),
                config,
            );

            debug!(message="connecting streaming pull stream", %subscription, %client_id);
            let mut error = match client
                .streaming_pull(request_stream)
                .await
                .map(|response| response.into_inner())
            {
                Err(err) => err,
                Ok(mut message_stream) => 'read: loop {
                    match message_stream.next().instrument(trace_span!("sub_stream_pull")).await {
                        // If the stream is out of elements, some connection must have been closed.
                        // However PubSub docs say StreamingPull always terminates with an error,
                        // so this normal end-of-stream shouldn't happen, and instead should fall
                        // to the error branch.
                        //
                        // Here we assume some other part of the stack ended the connection, and
                        // therefore attempting to reconnect (with retry/backoff) is the right
                        // resolution
                        None => break 'read tonic::Status::aborted("unexpected end of stream"),

                        // if there's an error in reading, break to the error handler and check
                        // whether to retry
                        Some(Err(err)) => break 'read err,

                        // otherwise, we got a successful response
                        Some(Ok(response)) => {
                            // If we were in a retry loop, declare it complete
                            retry_op = None;

                            for message in response.received_messages {
                                let ack_token = AcknowledgeToken {
                                    id: message.ack_id,
                                    channel: sender.clone(),
                                    delivery_attempt: message.delivery_attempt,
                                };
                                let message = match message.message {
                                    Some(msg) => msg,
                                    None => break 'read tonic::Status::internal(
                                        "message should be populated by RPC server"
                                    ),
                                };
                                yield Ok((ack_token, message));
                            }

                            continue 'read;
                        }
                    }
                }
            };
            debug!("Stream ended");

            // if either the streaming connection or a stream element produces an error,
            // the error will arrive here.

            // check if this error can be recovered by reconnecting the stream.
            let should_retry = retry_op
                .get_or_insert_with(|| retry_policy.new_operation())
                .check_retry(&(), &error);

            match should_retry {
                // if the retry policy determines a retry is possible, sleep for the
                // given backoff and then try reconnecting
                Some(backoff_sleep) => {
                    backoff_sleep.instrument(trace_span!("backoff_sleep")).await;
                    continue 'reconnect;
                }
                // if the policy does not provide a sleep, then it determined that the
                // operation is terminal (or the retries have been exhausted). Yield
                // the error, and then exit
                None => {
                    error.metadata_mut().insert("subscription", subscription_meta);
                    yield Err(error);
                    break 'reconnect;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;

    use super::*;

    #[test]
    fn streaming_pull_request_stream() {
        let subscription = "test-subscription";
        let client_id = "test-id";

        let (mut sender, receiver) = futures::channel::mpsc::unbounded();

        let requests = create_streaming_pull_request_stream(
            subscription.into(),
            client_id.into(),
            receiver,
            StreamSubscriptionConfig {
                max_outstanding_messages: 2000,
                max_outstanding_bytes: 3000,
                stream_ack_deadline: Duration::from_secs(20),
                ..Default::default()
            },
        );
        pin_mut!(requests);

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());

        // the request stream always starts with an initialized first request
        assert_eq!(
            Poll::Ready(Some(api::StreamingPullRequest {
                subscription: subscription.into(),
                ack_ids: vec![],
                modify_deadline_seconds: vec![],
                modify_deadline_ack_ids: vec![],
                stream_ack_deadline_seconds: 20,
                client_id: client_id.into(),
                max_outstanding_messages: 2000,
                max_outstanding_bytes: 3000,
            })),
            requests.as_mut().poll_next(&mut cx)
        );

        // no subsequent requests until a message is available on the channel
        assert_eq!(Poll::Pending, requests.as_mut().poll_next(&mut cx));

        // send 1 message ack
        assert_eq!(
            Ok(()),
            sender.unbounded_send(UserAck::Ack { id: "1st".into() })
        );

        // the output stream should be eager, and produce a request if only 1 message is available
        assert_eq!(
            Poll::Ready(Some(api::StreamingPullRequest {
                subscription: String::new(),
                ack_ids: vec!["1st".into()],
                modify_deadline_seconds: vec![],
                modify_deadline_ack_ids: vec![],
                stream_ack_deadline_seconds: 20,
                client_id: String::new(),
                max_outstanding_messages: 0,
                max_outstanding_bytes: 0,
            })),
            requests.as_mut().poll_next(&mut cx)
        );
        assert_eq!(Poll::Pending, requests.as_mut().poll_next(&mut cx));

        // send 3 message acks/modifies
        assert_eq!(
            Ok(()),
            sender.unbounded_send(UserAck::Ack { id: "2nd".into() })
        );
        assert_eq!(
            Ok(()),
            sender.unbounded_send(UserAck::Modify {
                id: "3rd".into(),
                seconds: 13
            })
        );
        assert_eq!(
            Ok(()),
            sender.unbounded_send(UserAck::Ack { id: "4th".into() })
        );
        assert_eq!(
            Ok(()),
            sender.unbounded_send(UserAck::Modify {
                id: "5th".into(),
                seconds: 15
            })
        );

        // the output stream should buffer when many acks/modifies are available
        assert_eq!(
            Poll::Ready(Some(api::StreamingPullRequest {
                subscription: String::new(),
                ack_ids: vec!["2nd".into(), "4th".into()],
                modify_deadline_seconds: vec![13, 15],
                modify_deadline_ack_ids: vec!["3rd".into(), "5th".into()],
                stream_ack_deadline_seconds: 20,
                client_id: String::new(),
                max_outstanding_messages: 0,
                max_outstanding_bytes: 0,
            })),
            requests.as_mut().poll_next(&mut cx)
        );
        assert_eq!(Poll::Pending, requests.as_mut().poll_next(&mut cx));

        // the output buffering has a limit of 1000. if more messages are immediately available,
        // they will be sent in multiple requests.

        // generate acks/modifies with random interleaving
        let inputs = std::iter::repeat_with(|| rand::thread_rng().gen::<bool>())
            .enumerate()
            .skip(6) // skip 0th,1st..5th
            .map(|(index, is_modify)| {
                let id = index.to_string() + "th";
                if is_modify {
                    UserAck::Modify {
                        id,
                        seconds: index as i32,
                    }
                } else {
                    UserAck::Ack { id }
                }
            })
            .take(1001)
            .collect::<Vec<_>>();

        let filter_acks = |acks: &[UserAck]| {
            acks.iter()
                .filter_map(|ack| match ack {
                    UserAck::Ack { id } => Some(id.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };

        let filter_modifies = |acks: &[UserAck]| {
            acks.iter()
                .filter_map(|ack| match ack {
                    UserAck::Modify { id, seconds } => Some((id.clone(), *seconds)),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };

        let expected_first_batch = api::StreamingPullRequest {
            subscription: String::new(),
            ack_ids: filter_acks(&inputs[..1000]),
            modify_deadline_seconds: filter_modifies(&inputs[..1000])
                .into_iter()
                .map(|tup| tup.1)
                .collect(),
            modify_deadline_ack_ids: filter_modifies(&inputs[..1000])
                .into_iter()
                .map(|tup| tup.0)
                .collect(),
            stream_ack_deadline_seconds: 20,
            client_id: String::new(),
            max_outstanding_messages: 0,
            max_outstanding_bytes: 0,
        };

        let expected_second_batch = api::StreamingPullRequest {
            subscription: String::new(),
            ack_ids: filter_acks(&inputs[1000..]),
            modify_deadline_seconds: filter_modifies(&inputs[1000..])
                .into_iter()
                .map(|tup| tup.1)
                .collect(),
            modify_deadline_ack_ids: filter_modifies(&inputs[1000..])
                .into_iter()
                .map(|tup| tup.0)
                .collect(),
            stream_ack_deadline_seconds: 20,
            client_id: String::new(),
            max_outstanding_messages: 0,
            max_outstanding_bytes: 0,
        };

        assert_eq!(
            Ok(()),
            inputs
                .into_iter()
                .try_for_each(|ack| sender.unbounded_send(ack))
        );

        assert_eq!(
            Poll::Ready(Some(expected_first_batch)),
            requests.as_mut().poll_next(&mut cx)
        );
        assert_eq!(
            Poll::Ready(Some(expected_second_batch)),
            requests.as_mut().poll_next(&mut cx)
        );
        assert_eq!(Poll::Pending, requests.as_mut().poll_next(&mut cx));

        // the request stream should end when the input stream ends
        sender.disconnect();
        assert_eq!(Poll::Ready(None), requests.as_mut().poll_next(&mut cx));
    }
}
