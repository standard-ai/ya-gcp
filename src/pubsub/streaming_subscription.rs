use std::{
    convert::TryFrom,
    future::Future,
    mem,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures::{
    future::{self, Either, FutureExt, TryFutureExt},
    pin_mut,
    stream::{self, Stream},
    StreamExt,
};
use pin_project::pin_project;
use tokio::{
    sync::{mpsc, oneshot, Notify},
    time::error::Elapsed as TimeoutElapsed,
};
use tonic::metadata::MetadataValue;
use tracing::{debug, trace_span, Instrument};

use crate::{
    grpc::{Body, BoxBody, Bytes, GrpcService, StdError},
    pubsub::{api, PubSubRetryCheck},
    retry_policy::{exponential_backoff, ExponentialBackoff, RetryOperation, RetryPolicy},
};

/// The maximum deadline supported by the RPC service
const MAX_DEADLINE_SEC: u32 = 600;

/// The maximum number of acks or modacks in a single request
// 2500 as used in the go lib
// https://github.com/googleapis/google-cloud-go/blob/94d040898cc9e85fdac76560765b01cfd019d0b4/pubsub/iterator.go#L44-L52
const MAX_ACK_BATCH_SIZE: usize = 2500;

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
        //TODO(0.12.0) remove deprecated field
        #[deprecated]
        @default(0, "StreamSubscriptionConfig::default_ack_channel_capacity")
        pub ack_channel_capacity: usize,
    }
}

/// An error encountered when issuing acks, nacks, or modifications from an
/// [`AcknowledgeToken`](AcknowledgeToken)
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
#[error("failed to ack/nack/modify")]
pub struct AcknowledgeError(#[source] AckErr);

#[derive(Debug, Clone, thiserror::Error)]
enum AckErr {
    #[error("error in background task; check primary pull stream for errors")]
    BackgroundTaskPanic,

    #[error(transparent)]
    Request(tonic::Status),
}

// TODO(0.12.0) remove eq/partialeq, use `matches!` in tests instead
impl PartialEq for AckErr {
    fn eq(&self, other: &AckErr) -> bool {
        use AckErr::*;
        match (self, other) {
            (BackgroundTaskPanic, BackgroundTaskPanic) => true,
            (Request(status_a), Request(status_b)) => status_a.code() == status_b.code(),
            _ => false,
        }
    }
}

impl Eq for AckErr {}

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

#[derive(Debug)]
struct AckRouter {
    // These channels are unbounded in a technical sense; however in practice there is a bound to
    // the number of outstanding messages which pubsub will issue to a streaming caller. This will
    // implicitly limit the ack/nack channel sizes.
    //
    // However modacks are not limited, as well as acks/nacks if a user sets both
    // max_outstanding_bytes and max_outstanding_messages to zero (unbounded). Then it's up to the
    // user to impose backpressure by awaiting the returned futures
    acks: mpsc::UnboundedSender<TokenFeedback<String>>,
    nacks: mpsc::UnboundedSender<TokenFeedback<String>>,
    modacks: mpsc::UnboundedSender<TokenFeedback<ModAck>>,
}

struct ModAck {
    id: String,
    deadline: i32,
}

/// A token with an associated message produced by the [`StreamSubscription`](StreamSubscription)
/// stream, used to control that message's re-delivery within the message queue.
#[derive(Debug)]
pub struct AcknowledgeToken {
    id: String,
    router: Arc<AckRouter>,
    delivery_attempt: i32,
}

impl AcknowledgeToken {
    /// Acknowledge the corresponding message as received, so that the message service will stop
    /// attempting to deliver it to subscribers.
    ///
    /// Note that acknowledgements may not arrive to the service within the deadline (or at all),
    /// so this is only a best-effort means of preventing re-delivery.
    ///
    /// The returned future will complete once the acknowledge request has been sent. It is not
    /// necessary to wait for the future's completion however; calling this function will initiate
    /// an acknowledgement, which will finish even without awaiting the future. This can be useful
    /// for callers that don't need explicit ack settlement and prefer to save latency.
    pub fn ack(self) -> impl Future<Output = Result<(), AcknowledgeError>> + Send {
        TokenFeedback::send(&self.router.acks, self.id)
    }

    /// Negatively acknowledge the corresponding message, requesting that the message service
    /// re-deliver it to subscribers.
    ///
    /// This may be useful if the message consumer encounters an error while processing the
    /// message.
    ///
    /// The returned future need not be awaited, see [`ack`]
    pub fn nack(self) -> impl Future<Output = Result<(), AcknowledgeError>> + Send {
        TokenFeedback::send(&self.router.nacks, self.id)
    }

    /// Modify the acknowledgement deadline of the corresponding message, so that re-delivery is
    /// not attempted until the given time elapses (unless an ack/nack is sent).
    ///
    /// The message's deadline will be set to the given number of seconds after the service
    /// receives the modify request. Note that this could be shorter than the message's prior
    /// deadline, if for example the [subscription-level
    /// deadline](StreamSubscriptionConfig::stream_ack_deadline) is longer. A deadline of
    /// 0 seconds is equivalent to a [`nack`](AcknowledgeToken::nack) and will make the message
    /// immediately available for re-delivery. The maximum deadline accepted by the service is 600
    /// seconds.
    ///
    /// The returned future need not be awaited, see [`ack`]
    pub fn modify_deadline(
        &mut self,
        seconds: u32,
    ) -> impl Future<Output = Result<(), ModifyAcknowledgeError>> + Send {
        if seconds > MAX_DEADLINE_SEC {
            return Either::Left(future::ready(Err(
                ModifyAcknowledgeError::InvalidDeadline { seconds },
            )));
        }

        Either::Right(
            TokenFeedback::send(
                &self.router.modacks,
                ModAck {
                    id: self.id.clone(),
                    deadline: seconds as i32,
                },
            )
            .map_err(ModifyAcknowledgeError::Modify),
        )
    }

    /// The approximate number of times that Cloud Pub/Sub has attempted to deliver the associated
    /// message to a subscriber.
    ///
    /// See [`delivery_attempt`](api::ReceivedMessage::delivery_attempt)
    pub fn delivery_attempt(&self) -> i32 {
        self.delivery_attempt
    }
}

// Each ack token's ack/nack/modack calls are carried by this struct to the background task that
// polls the mpsc channels. That task will then notify the token of the request's completion by sending
// a result back over the given oneshot channel
struct TokenFeedback<T> {
    payload: T,
    completion: oneshot::Sender<Result<(), AcknowledgeError>>,
}

impl<T: Send> TokenFeedback<T> {
    fn send(
        channel: &mpsc::UnboundedSender<TokenFeedback<T>>,
        payload: T,
    ) -> impl Future<Output = Result<(), AcknowledgeError>> + Send {
        let (completion, listener) = oneshot::channel();

        // send the payload over the channel synchronously. After this, the caller could drop the
        // returned future and the work would still happen (barring errors/panics)
        let send_result = channel.send(Self {
            completion,
            payload,
        });

        // now create the future to actually wait on the outcome
        async move {
            match send_result {
                Ok(()) => match listener.await {
                    // if the background task completed a request with our payload, then
                    // we're ready to return a normal case to the user. Note this might still be a
                    // failed response, but the request completed a trip to the pubsub service
                    Ok(server_response) => return server_response,

                    Err(oneshot::error::RecvError { .. }) => {}
                },
                Err(mpsc::error::SendError { .. }) => {}
            }
            // Either SendError or RecvError imply that the other end of the channel was dropped.
            // This means the background handler has stopped; if there are still senders open, that
            // should only happen if a panic happened in that task
            Err(AcknowledgeError(AckErr::BackgroundTaskPanic))
        }
    }
}

/// Wait for acks to arrive over the given channel, and send them to the server via the acknowledge
/// grpc method.
async fn handle_acks<S>(
    mut client: api::subscriber_client::SubscriberClient<S>,
    subscription: String,
    mut acks: mpsc::UnboundedReceiver<TokenFeedback<String>>,
) where
    S: GrpcService<BoxBody> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<StdError>,
    S::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <S::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    let mut batch = Vec::new();
    loop {
        let fetch_count = acks.recv_many(&mut batch, MAX_ACK_BATCH_SIZE).await;
        if fetch_count == 0 {
            // all senders dropped, pull stream must have closed
            break;
        }
        let request = api::AcknowledgeRequest {
            subscription: subscription.clone(),
            ack_ids: batch
                .iter_mut()
                .map(|TokenFeedback { payload, .. }| mem::take(payload))
                .collect(),
        };

        let response = client
            .acknowledge(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(|err| AcknowledgeError(AckErr::Request(err)));

        let mut listeners = batch
            .drain(..)
            .map(|TokenFeedback { completion, .. }| completion);

        // peel off the first to avoid cloning in the common single-message case
        let first = listeners.next().expect("fetched > 0");

        // Send failures can only happen if the receiver was dropped.
        // That's benign, the user isn't listening for ack responses. Ignore such failures
        for listener in listeners {
            let _ = listener.send(response.clone());
        }
        let _ = first.send(response);
    }
}

/// much like handle_acks except includes ack_deadline_seconds and calls modify_ack_deadline.
// unfortunately hard to unify the two without proper async closures (or macros i guess)
async fn handle_nacks<S>(
    mut client: api::subscriber_client::SubscriberClient<S>,
    subscription: String,
    mut nacks: mpsc::UnboundedReceiver<TokenFeedback<String>>,
) where
    S: GrpcService<BoxBody> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<StdError>,
    S::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <S::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    let mut batch = Vec::new();
    loop {
        let fetch_count = nacks.recv_many(&mut batch, MAX_ACK_BATCH_SIZE).await;
        if fetch_count == 0 {
            break;
        }
        let request = api::ModifyAckDeadlineRequest {
            subscription: subscription.clone(),
            // zero seconds implies nack
            ack_deadline_seconds: 0,
            ack_ids: batch
                .iter_mut()
                .map(|TokenFeedback { payload, .. }| mem::take(payload))
                .collect(),
        };

        let response = client
            .modify_ack_deadline(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(|err| AcknowledgeError(AckErr::Request(err)));

        let mut listeners = batch
            .drain(..)
            .map(|TokenFeedback { completion, .. }| completion);

        let first = listeners.next().expect("fetched > 0");
        for listener in listeners {
            let _ = listener.send(response.clone());
        }
        let _ = first.send(response);
    }
}

async fn handle_modacks<S>(
    mut client: api::subscriber_client::SubscriberClient<S>,
    subscription: String,
    mut modacks: mpsc::UnboundedReceiver<TokenFeedback<ModAck>>,
) where
    S: GrpcService<BoxBody> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<StdError>,
    S::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <S::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    let mut batch = Vec::new();
    loop {
        let fetch_count = modacks.recv_many(&mut batch, MAX_ACK_BATCH_SIZE).await;
        if fetch_count == 0 {
            break;
        }

        // Unlike acks and nacks, each modack can have a different deadline. The request schema
        // specifies a single deadline for all tokens in its batch. To resolve these two
        // constraints, sort the batch into sections of identical deadlines, and send a batched
        // request for each section.
        //
        // This is sorted in descending order because the below code pulls off the end. We want the
        // shortest deadlines sent to the server first, on the theory that shorter deadlines imply
        // tighter timing (though maybe it's irrelevant, deadlines are seconds while requests are
        // hopefully millis)
        batch.sort_by_key(|entry| std::cmp::Reverse(entry.payload.deadline));

        // take sections off the end to avoid shifting values down as they're drained
        while let Some(last_entry) = batch.last() {
            let ack_deadline_seconds = last_entry.payload.deadline;
            let section_start =
                batch.partition_point(|entry| entry.payload.deadline > ack_deadline_seconds);

            let request = api::ModifyAckDeadlineRequest {
                subscription: subscription.clone(),
                ack_deadline_seconds,
                ack_ids: batch[section_start..]
                    .iter_mut()
                    .map(|TokenFeedback { payload, .. }| mem::take(&mut payload.id))
                    .collect(),
            };

            let response = client
                .modify_ack_deadline(request)
                .await
                .map(tonic::Response::into_inner)
                .map_err(|err| AcknowledgeError(AckErr::Request(err)));

            let mut listeners = batch
                .drain(section_start..)
                .map(|TokenFeedback { completion, .. }| completion);

            let first = listeners.next().expect("fetched > 0");
            for listener in listeners {
                let _ = listener.send(response.clone());
            }
            let _ = first.send(response);
        }
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
        max_outstanding_bytes: config.max_outstanding_bytes,
        ..Default::default()
    }
}

/// Create the Nth StreamingPullRequest message (i.e. 2nd or later).
fn create_subsequent_streaming_pull_request(
    config: &StreamSubscriptionConfig,
) -> api::StreamingPullRequest {
    api::StreamingPullRequest {
        // Even though this was set on the first request, it is reset on every subsequent request.
        // Here we "reset" it to the same value.
        stream_ack_deadline_seconds: i32::try_from(config.stream_ack_deadline.as_secs())
            .expect("ack deadline seconds should fit in i32"),
        ..Default::default()
    }
}

/// Create an indefinite request stream of StreamingPullRequests. The first request initializes a new
/// gRPC stream and subsequent messages will keep the connection alive.
///
/// The returned pair includes a stop indicator: after the value is dropped, the stream will stop
/// yielding elements and terminate.
fn create_streaming_pull_request_stream(
    subscription: String,
    client_id: String,
    config: StreamSubscriptionConfig,
) -> (impl Stream<Item = api::StreamingPullRequest>, impl Drop) {
    let stop_check = Arc::new(Notify::new());

    struct StopOnDrop(Arc<Notify>);
    impl Drop for StopOnDrop {
        fn drop(&mut self) {
            self.0.notify_one();
        }
    }
    let stopper = StopOnDrop(Arc::clone(&stop_check));

    let stream = async_stream::stream! {
        // start by issuing the first request
        yield create_initial_streaming_pull_request(subscription, client_id, &config);

        let should_stop = stop_check.notified();
        pin_mut!(should_stop);

        // Periodically send requests to keep the grpc connection active. This can help in cases
        // where messages aren't being actively read (e.g. processing messages takes a long time).
        //
        // This does not send acks back over the stream, instead opting for explicit ack requests
        // to have better feedback over ack completion/success/failure.
        loop {
            match tokio::time::timeout(Duration::from_secs(30), should_stop.as_mut()).await {
                Ok(()) => break,
                Err(TimeoutElapsed { .. }) => {
                    yield create_subsequent_streaming_pull_request(&config);
                }
            }
        }
    };

    (stream, stopper)
}

/// The stream returned by the
/// [`stream_subscription`](crate::pubsub::SubscriberClient::stream_subscription) function
#[pin_project]
pub struct StreamSubscription<
    S = crate::grpc::DefaultGrpcImpl,
    R = ExponentialBackoff<PubSubRetryCheck>,
> {
    state: StreamState<S, R>,
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
enum StreamState<S, R> {
    Initialized {
        client: [api::subscriber_client::SubscriberClient<S>; 4],
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

impl<S> StreamSubscription<S> {
    pub(super) fn new(
        client: [api::subscriber_client::SubscriberClient<S>; 4],
        subscription: String,
        config: StreamSubscriptionConfig,
    ) -> Self {
        StreamSubscription {
            state: StreamState::Initialized {
                client,
                subscription,
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

impl<S, OldR> StreamSubscription<S, OldR> {
    /// Set the [`RetryPolicy`] to use for this streaming subscription.
    ///
    /// The stream will be reconnected if the policy indicates that an encountered
    /// [`Status`](tonic::Status) error should be retried
    // Because `poll_next` requires `Pin<&mut Self>`, this function cannot be called after the
    // stream has started because it moves `self`. That means that the retry policy can only be
    // changed before the polling starts, and is fixed from that point on
    pub fn with_retry_policy<R>(self, new_retry_policy: R) -> StreamSubscription<S, R>
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

impl<S, R> Stream for StreamSubscription<S, R>
where
    S: GrpcService<BoxBody> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<StdError>,
    S::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <S::ResponseBody as Body>::Error: Into<StdError> + Send,
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
fn stream_from_client<S, R>(
    clients: [api::subscriber_client::SubscriberClient<S>; 4],
    subscription: String,
    config: StreamSubscriptionConfig,
    mut retry_policy: R,
) -> impl Stream<Item = Result<(AcknowledgeToken, api::PubsubMessage), tonic::Status>> + Send + 'static
where
    S: GrpcService<BoxBody> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<StdError>,
    S::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <S::ResponseBody as Body>::Error: Into<StdError> + Send,
    R: RetryPolicy<(), tonic::Status> + Send + 'static,
    R::RetryOp: Send + 'static,
    <R::RetryOp as RetryOperation<(), tonic::Status>>::Sleep: Send + 'static,
{
    let subscription_meta =
        MetadataValue::try_from(&subscription).expect("valid subscription metadata");

    // the client id is used for stream reconnection on error
    let client_id = uuid::Uuid::new_v4().to_string();

    let [mut client, ack_client, nack_client, modack_client] = clients;

    async_stream::stream! {
        let mut retry_op = None;

        let (acks, acks_rx) = mpsc::unbounded_channel();
        let (nacks, nacks_rx) = mpsc::unbounded_channel();
        let (modacks, modacks_rx) = mpsc::unbounded_channel();
        let ack_router = Arc::new(AckRouter { acks, nacks, modacks });

        // spawn the ack processing in the background. These should continue to process even
        // when messages are not being pulled.
        let ack_processor = tokio::spawn(future::join3(
                handle_acks(ack_client, subscription.clone(), acks_rx),
                handle_nacks(nack_client, subscription.clone(), nacks_rx),
                handle_modacks(modack_client, subscription.clone(), modacks_rx),
            ))
            .unwrap_or_else(|join_err| std::panic::resume_unwind(join_err.into_panic()))
            .map(|((), (), ())| ());
        pin_mut!(ack_processor);

        'reconnect: loop {
            let (request_stream, stream_drop_stopper) = create_streaming_pull_request_stream(
                subscription.clone(),
                client_id.clone(),
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
                    // check if the background processor encountered any panics;
                    // if not, try to read messages from the stream.
                    let msg = message_stream.next().instrument(trace_span!("sub_stream_pull"));
                    pin_mut!(msg);
                    let next = future::poll_fn(|cx| match ack_processor.as_mut().poll(cx) {
                        Poll::Ready(()) => unreachable!("shouldn't complete while stream is active"),
                        Poll::Pending => msg.as_mut().poll(cx)
                    });
                    match next.await {
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
                                    router: Arc::clone(&ack_router),
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
            std::mem::drop(stream_drop_stopper);
            debug!(%client_id, "Stream ended");

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
    use super::*;
    use std::sync::Mutex;
    use tonic::Code;

    #[test]
    fn token_send() {
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        let (send, mut recv) = mpsc::unbounded_channel();
        let mut fut = TokenFeedback::send(&send, "hello world").boxed();

        // without any poll, the item should already be sent over the channel
        let TokenFeedback {
            payload,
            completion,
        } = recv.try_recv().expect("send should be synchronous");
        assert_eq!(payload, "hello world");

        // the future should now be waiting for a response on the completion channel
        assert!(matches!(fut.as_mut().poll(&mut cx), Poll::Pending));
        completion.send(Ok(())).expect("oneshot is open");
        assert!(matches!(fut.as_mut().poll(&mut cx), Poll::Ready(Ok(()))));

        // setup another send to witness an error response
        let mut fut = TokenFeedback::send(&send, "abc123").boxed();
        let TokenFeedback { completion, .. } = recv.try_recv().expect("send should be synchronous");
        assert!(matches!(fut.as_mut().poll(&mut cx), Poll::Pending));
        completion
            .send(Err(AcknowledgeError(AckErr::BackgroundTaskPanic)))
            .expect("oneshot is open");
        assert!(matches!(
            fut.as_mut().poll(&mut cx),
            Poll::Ready(Err(AcknowledgeError(AckErr::BackgroundTaskPanic)))
        ));
    }

    #[test]
    fn token_wait() {
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        let (send, mut recv) = mpsc::unbounded_channel();

        let mut fut = TokenFeedback::send(&send, "hello world").boxed();
        let TokenFeedback { completion, .. } = recv.try_recv().expect("send should be synchronous");

        // if the completion gets dropped, the waiting future should resolve with an error
        std::mem::drop(completion);
        assert!(matches!(
            fut.as_mut().poll(&mut cx),
            Poll::Ready(Err(AcknowledgeError(AckErr::BackgroundTaskPanic)))
        ));

        // that also applies if the future is already polling
        let mut fut = TokenFeedback::send(&send, "hello world").boxed();
        let TokenFeedback { completion, .. } = recv.try_recv().expect("send should be synchronous");
        assert!(matches!(fut.as_mut().poll(&mut cx), Poll::Pending));
        std::mem::drop(completion);
        assert!(matches!(
            fut.as_mut().poll(&mut cx),
            Poll::Ready(Err(AcknowledgeError(AckErr::BackgroundTaskPanic)))
        ));

        // start a send which will encounter a receiver drop
        let mut fut = TokenFeedback::send(&send, "hello world").boxed();
        assert!(matches!(fut.as_mut().poll(&mut cx), Poll::Pending));
        std::mem::drop(recv);
        assert!(matches!(
            fut.as_mut().poll(&mut cx),
            Poll::Ready(Err(AcknowledgeError(AckErr::BackgroundTaskPanic)))
        ));

        // a send started after the receiver drop should also fail
        let mut fut = TokenFeedback::send(&send, "hello world").boxed();
        assert!(matches!(
            fut.as_mut().poll(&mut cx),
            Poll::Ready(Err(AcknowledgeError(AckErr::BackgroundTaskPanic)))
        ));
    }

    #[test]
    fn ack_handling() {
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        #[derive(Default, Clone)]
        struct MockSubscriberServer {
            acks: Arc<Mutex<Vec<api::AcknowledgeRequest>>>,
            injected_errors: Arc<Mutex<Vec<tonic::Status>>>,
        }

        #[tonic::codegen::async_trait]
        impl api::subscriber_server::Subscriber for MockSubscriberServer {
            async fn acknowledge(
                &self,
                request: tonic::Request<api::AcknowledgeRequest>,
            ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
                self.acks.lock().unwrap().push(request.into_inner());

                let mut errs = self.injected_errors.lock().unwrap();
                if errs.is_empty() {
                    Ok(tonic::Response::new(()))
                } else {
                    Err(errs.remove(0))
                }
            }
        }

        let (ack_send, recv) = mpsc::unbounded_channel();
        let server = MockSubscriberServer::default();
        let take_server_acks = || server.acks.lock().unwrap().drain(..).collect::<Vec<_>>();

        let mut ack_handler = handle_acks(
            api::subscriber_client::SubscriberClient::new(
                api::subscriber_server::SubscriberServer::new(server.clone()),
            ),
            "test-subscription".into(),
            recv,
        )
        .boxed();

        // simple single ack case
        let mut ack_fut = TokenFeedback::send(&ack_send, "ack-id1".into()).boxed();
        // drive ack handler
        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));

        // check that the server got the request
        assert_eq!(
            take_server_acks(),
            vec![api::AcknowledgeRequest {
                subscription: "test-subscription".into(),
                ack_ids: vec!["ack-id1".into()],
            }]
        );
        // and that the ack token got its response
        assert!(matches!(
            ack_fut.as_mut().poll(&mut cx),
            Poll::Ready(Ok(()))
        ));

        // send multiple acks before the handler polls again
        let mut ack_fut = future::join3(
            TokenFeedback::send(&ack_send, "ack-id2".into()),
            TokenFeedback::send(&ack_send, "ack-id3".into()),
            TokenFeedback::send(&ack_send, "ack-id4".into()),
        )
        .boxed();
        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));

        // check that the server got a single buffered request
        assert_eq!(
            take_server_acks(),
            vec![api::AcknowledgeRequest {
                subscription: "test-subscription".into(),
                ack_ids: vec!["ack-id2".into(), "ack-id3".into(), "ack-id4".into()],
            }]
        );

        // and that all the futures got responses
        assert!(matches!(
            ack_fut.as_mut().poll(&mut cx),
            Poll::Ready((Ok(()), Ok(()), Ok(())))
        ));

        // send multiple acks again
        let mut ack_fut = future::join3(
            TokenFeedback::send(&ack_send, "ack-id5".into()),
            TokenFeedback::send(&ack_send, "ack-id6".into()),
            TokenFeedback::send(&ack_send, "ack-id7".into()),
        )
        .boxed();
        // however this time inject an error response from the server
        server
            .injected_errors
            .lock()
            .unwrap()
            .push(tonic::Status::aborted("injected-error"));

        // drive the ack handler
        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));
        assert_eq!(
            take_server_acks(),
            vec![api::AcknowledgeRequest {
                subscription: "test-subscription".into(),
                ack_ids: vec!["ack-id5".into(), "ack-id6".into(), "ack-id7".into()],
            }]
        );

        // the ack tokens should each get back the error
        let ack_responses = ack_fut.as_mut().poll(&mut cx);
        match ack_responses {
            Poll::Ready((
                Err(AcknowledgeError(AckErr::Request(status1))),
                Err(AcknowledgeError(AckErr::Request(status2))),
                Err(AcknowledgeError(AckErr::Request(status3))),
            )) if (
                (status1.code(), status2.code(), status3.code()),
                (status1.message(), status2.message(), status3.message()),
            ) == (
                (Code::Aborted, Code::Aborted, Code::Aborted),
                ("injected-error", "injected-error", "injected-error"),
            ) => {}
            _ => panic!("unexpected future output {ack_responses:?}"),
        };

        // if more than the batch limit is submitted, the handler will send multiple requests
        let futs = (0..(MAX_ACK_BATCH_SIZE + 2))
            .map(|i| TokenFeedback::send(&ack_send, format!("mass-ack{i}")))
            .collect::<Vec<_>>();

        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));
        let server_acks = take_server_acks();
        assert_eq!(server_acks.len(), 2);
        assert_eq!(
            server_acks[0].ack_ids,
            (0..MAX_ACK_BATCH_SIZE)
                .map(|i| format!("mass-ack{i}"))
                .collect::<Vec<_>>()
        );

        const _SANITY_CHECK: [(); 2500] = [(); MAX_ACK_BATCH_SIZE];
        assert_eq!(
            server_acks[1].ack_ids,
            vec!["mass-ack2500".to_owned(), "mass-ack2501".to_owned()]
        );

        // all the futures should get their success response
        for fut in futs {
            assert!(matches!(
                fut.boxed().as_mut().poll(&mut cx),
                Poll::Ready(Ok(()))
            ));
        }

        // the handler future can complete after the sender side is dropped.
        // however it must first flush any acks still in the queue.
        let mut ack_fut = TokenFeedback::send(&ack_send, "ack-id99".into()).boxed();
        std::mem::drop(ack_send);
        assert!(take_server_acks().is_empty()); // sanity check
        assert!(matches!(
            ack_handler.as_mut().poll(&mut cx),
            Poll::Ready(())
        ));
        assert_eq!(
            take_server_acks(),
            vec![api::AcknowledgeRequest {
                subscription: "test-subscription".into(),
                ack_ids: vec!["ack-id99".into()]
            }]
        );
        assert!(matches!(
            ack_fut.as_mut().poll(&mut cx),
            Poll::Ready(Ok(()))
        ));
    }

    // copy-paste of ack handler. practically identical functionality
    #[test]
    fn nack_handling() {
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        #[derive(Default, Clone)]
        struct MockSubscriberServer {
            acks: Arc<Mutex<Vec<api::ModifyAckDeadlineRequest>>>,
            injected_errors: Arc<Mutex<Vec<tonic::Status>>>,
        }

        #[tonic::codegen::async_trait]
        impl api::subscriber_server::Subscriber for MockSubscriberServer {
            async fn modify_ack_deadline(
                &self,
                request: tonic::Request<api::ModifyAckDeadlineRequest>,
            ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
                self.acks.lock().unwrap().push(request.into_inner());

                let mut errs = self.injected_errors.lock().unwrap();
                if errs.is_empty() {
                    Ok(tonic::Response::new(()))
                } else {
                    Err(errs.remove(0))
                }
            }
        }

        let (ack_send, recv) = mpsc::unbounded_channel();
        let server = MockSubscriberServer::default();
        let take_server_acks = || server.acks.lock().unwrap().drain(..).collect::<Vec<_>>();

        let mut ack_handler = handle_nacks(
            api::subscriber_client::SubscriberClient::new(
                api::subscriber_server::SubscriberServer::new(server.clone()),
            ),
            "test-subscription".into(),
            recv,
        )
        .boxed();

        // simple single ack case
        let mut ack_fut = TokenFeedback::send(&ack_send, "ack-id1".into()).boxed();
        // drive ack handler
        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));

        // check that the server got the request
        assert_eq!(
            take_server_acks(),
            vec![api::ModifyAckDeadlineRequest {
                subscription: "test-subscription".into(),
                ack_ids: vec!["ack-id1".into()],
                ack_deadline_seconds: 0,
            }]
        );
        // and that the ack token got its response
        assert!(matches!(
            ack_fut.as_mut().poll(&mut cx),
            Poll::Ready(Ok(()))
        ));

        // send multiple acks before the handler polls again
        let mut ack_fut = future::join3(
            TokenFeedback::send(&ack_send, "ack-id2".into()),
            TokenFeedback::send(&ack_send, "ack-id3".into()),
            TokenFeedback::send(&ack_send, "ack-id4".into()),
        )
        .boxed();
        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));

        // check that the server got a single buffered request
        assert_eq!(
            take_server_acks(),
            vec![api::ModifyAckDeadlineRequest {
                subscription: "test-subscription".into(),
                ack_ids: vec!["ack-id2".into(), "ack-id3".into(), "ack-id4".into()],
                ack_deadline_seconds: 0,
            }]
        );

        // and that all the futures got responses
        assert!(matches!(
            ack_fut.as_mut().poll(&mut cx),
            Poll::Ready((Ok(()), Ok(()), Ok(())))
        ));

        // send multiple acks again
        let mut ack_fut = future::join3(
            TokenFeedback::send(&ack_send, "ack-id5".into()),
            TokenFeedback::send(&ack_send, "ack-id6".into()),
            TokenFeedback::send(&ack_send, "ack-id7".into()),
        )
        .boxed();
        // however this time inject an error response from the server
        server
            .injected_errors
            .lock()
            .unwrap()
            .push(tonic::Status::aborted("injected-error"));

        // drive the ack handler
        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));
        assert_eq!(
            take_server_acks(),
            vec![api::ModifyAckDeadlineRequest {
                subscription: "test-subscription".into(),
                ack_ids: vec!["ack-id5".into(), "ack-id6".into(), "ack-id7".into()],
                ack_deadline_seconds: 0,
            }]
        );

        // the ack tokens should each get back the error
        let ack_responses = ack_fut.as_mut().poll(&mut cx);
        match ack_responses {
            Poll::Ready((
                Err(AcknowledgeError(AckErr::Request(status1))),
                Err(AcknowledgeError(AckErr::Request(status2))),
                Err(AcknowledgeError(AckErr::Request(status3))),
            )) if (
                (status1.code(), status2.code(), status3.code()),
                (status1.message(), status2.message(), status3.message()),
            ) == (
                (Code::Aborted, Code::Aborted, Code::Aborted),
                ("injected-error", "injected-error", "injected-error"),
            ) => {}
            _ => panic!("unexpected future output {ack_responses:?}"),
        };

        // if more than the batch limit is submitted, the handler will send multiple requests
        let futs = (0..(MAX_ACK_BATCH_SIZE + 2))
            .map(|i| TokenFeedback::send(&ack_send, format!("mass-ack{i}")))
            .collect::<Vec<_>>();

        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));
        let server_acks = take_server_acks();
        assert_eq!(server_acks.len(), 2);
        assert_eq!(
            server_acks[0].ack_ids,
            (0..MAX_ACK_BATCH_SIZE)
                .map(|i| format!("mass-ack{i}"))
                .collect::<Vec<_>>()
        );

        const _SANITY_CHECK: [(); 2500] = [(); MAX_ACK_BATCH_SIZE];
        assert_eq!(
            server_acks[1].ack_ids,
            vec!["mass-ack2500".to_owned(), "mass-ack2501".to_owned()]
        );

        // all the futures should get their success response
        for fut in futs {
            assert!(matches!(
                fut.boxed().as_mut().poll(&mut cx),
                Poll::Ready(Ok(()))
            ));
        }

        // the handler future can complete after the sender side is dropped.
        // however it must first flush any acks still in the queue.
        let mut ack_fut = TokenFeedback::send(&ack_send, "ack-id99".into()).boxed();
        std::mem::drop(ack_send);
        assert!(take_server_acks().is_empty()); // sanity check
        assert!(matches!(
            ack_handler.as_mut().poll(&mut cx),
            Poll::Ready(())
        ));
        assert_eq!(
            take_server_acks(),
            vec![api::ModifyAckDeadlineRequest {
                subscription: "test-subscription".into(),
                ack_ids: vec!["ack-id99".into()],
                ack_deadline_seconds: 0,
            }]
        );
        assert!(matches!(
            ack_fut.as_mut().poll(&mut cx),
            Poll::Ready(Ok(()))
        ));
    }

    // *NOT* a (direct) copy-paste of ack handler, accounting for multiple deadlines. still mostly
    // a copy though...
    #[test]
    fn modack_handling() {
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        #[derive(Default, Clone)]
        struct MockSubscriberServer {
            acks: Arc<Mutex<Vec<api::ModifyAckDeadlineRequest>>>,
            injected_errors: Arc<Mutex<Vec<tonic::Status>>>,
        }

        #[tonic::codegen::async_trait]
        impl api::subscriber_server::Subscriber for MockSubscriberServer {
            async fn modify_ack_deadline(
                &self,
                request: tonic::Request<api::ModifyAckDeadlineRequest>,
            ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
                self.acks.lock().unwrap().push(request.into_inner());

                let mut errs = self.injected_errors.lock().unwrap();
                if errs.is_empty() {
                    Ok(tonic::Response::new(()))
                } else {
                    Err(errs.remove(0))
                }
            }
        }

        let (ack_send, recv) = mpsc::unbounded_channel();
        let server = MockSubscriberServer::default();
        let take_server_acks = || server.acks.lock().unwrap().drain(..).collect::<Vec<_>>();

        let mut ack_handler = handle_modacks(
            api::subscriber_client::SubscriberClient::new(
                api::subscriber_server::SubscriberServer::new(server.clone()),
            ),
            "test-subscription".into(),
            recv,
        )
        .boxed();

        // simple single ack case
        let mut ack_fut = TokenFeedback::send(
            &ack_send,
            ModAck {
                id: "ack-id1".into(),
                deadline: 1,
            },
        )
        .boxed();
        // drive ack handler
        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));

        // check that the server got the request
        assert_eq!(
            take_server_acks(),
            vec![api::ModifyAckDeadlineRequest {
                subscription: "test-subscription".into(),
                ack_ids: vec!["ack-id1".into()],
                ack_deadline_seconds: 1,
            }]
        );
        // and that the ack token got its response
        assert!(matches!(
            ack_fut.as_mut().poll(&mut cx),
            Poll::Ready(Ok(()))
        ));

        // send multiple acks before the handler polls again.
        // note these have varying deadlines. The handler may only batch together acks with the same
        // deadline
        let mut ack_fut = future::join5(
            TokenFeedback::send(
                &ack_send,
                ModAck {
                    id: "ack-id2".into(),
                    deadline: 2,
                },
            ),
            TokenFeedback::send(
                &ack_send,
                ModAck {
                    id: "ack-id3".into(),
                    deadline: 5,
                },
            ),
            TokenFeedback::send(
                &ack_send,
                ModAck {
                    id: "ack-id4".into(),
                    deadline: 2,
                },
            ),
            TokenFeedback::send(
                &ack_send,
                ModAck {
                    id: "ack-id5".into(),
                    deadline: 5,
                },
            ),
            TokenFeedback::send(
                &ack_send,
                ModAck {
                    id: "ack-id6".into(),
                    deadline: 1,
                },
            ),
        )
        .boxed();
        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));

        // the server should have gotten separate requests for each deadline, in increasing
        // order by deadline.
        assert_eq!(
            take_server_acks(),
            vec![
                api::ModifyAckDeadlineRequest {
                    subscription: "test-subscription".into(),
                    ack_ids: vec!["ack-id6".into()],
                    ack_deadline_seconds: 1,
                },
                api::ModifyAckDeadlineRequest {
                    subscription: "test-subscription".into(),
                    ack_ids: vec!["ack-id2".into(), "ack-id4".into()],
                    ack_deadline_seconds: 2,
                },
                api::ModifyAckDeadlineRequest {
                    subscription: "test-subscription".into(),
                    ack_ids: vec!["ack-id3".into(), "ack-id5".into()],
                    ack_deadline_seconds: 5,
                },
            ]
        );

        // check that all the futures got responses
        assert!(matches!(
            ack_fut.as_mut().poll(&mut cx),
            Poll::Ready((Ok(()), Ok(()), Ok(()), Ok(()), Ok(())))
        ));

        // send multiple acks again
        let mut ack_fut = future::join3(
            TokenFeedback::send(
                &ack_send,
                ModAck {
                    id: "ack-id7".into(),
                    deadline: 1,
                },
            ),
            TokenFeedback::send(
                &ack_send,
                ModAck {
                    id: "ack-id8".into(),
                    deadline: 1,
                },
            ),
            TokenFeedback::send(
                &ack_send,
                ModAck {
                    id: "ack-id9".into(),
                    deadline: 1,
                },
            ),
        )
        .boxed();
        // however this time inject an error response from the server
        server
            .injected_errors
            .lock()
            .unwrap()
            .push(tonic::Status::aborted("injected-error"));

        // drive the ack handler
        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));
        assert_eq!(
            take_server_acks(),
            vec![api::ModifyAckDeadlineRequest {
                subscription: "test-subscription".into(),
                ack_ids: vec!["ack-id7".into(), "ack-id8".into(), "ack-id9".into()],
                ack_deadline_seconds: 1,
            },]
        );

        // the ack tokens should each get back the error
        let ack_responses = ack_fut.as_mut().poll(&mut cx);
        match ack_responses {
            Poll::Ready((
                Err(AcknowledgeError(AckErr::Request(status1))),
                Err(AcknowledgeError(AckErr::Request(status2))),
                Err(AcknowledgeError(AckErr::Request(status3))),
            )) if (
                (status1.code(), status2.code(), status3.code()),
                (status1.message(), status2.message(), status3.message()),
            ) == (
                (Code::Aborted, Code::Aborted, Code::Aborted),
                ("injected-error", "injected-error", "injected-error"),
            ) => {}
            _ => panic!("unexpected future output {ack_responses:?}"),
        };

        // if more than the batch limit is submitted, the handler will send multiple requests.
        let futs = (0..(MAX_ACK_BATCH_SIZE + 2))
            .map(|i| {
                TokenFeedback::send(
                    &ack_send,
                    ModAck {
                        id: format!("mass-ack{i}"),
                        deadline: 1,
                    },
                )
            })
            .collect::<Vec<_>>();

        assert!(matches!(ack_handler.as_mut().poll(&mut cx), Poll::Pending));
        let server_acks = take_server_acks();
        assert_eq!(server_acks.len(), 2);
        assert_eq!(
            server_acks[0].ack_ids,
            (0..MAX_ACK_BATCH_SIZE)
                .map(|i| format!("mass-ack{i}"))
                .collect::<Vec<_>>()
        );

        const _SANITY_CHECK: [(); 2500] = [(); MAX_ACK_BATCH_SIZE];
        assert_eq!(
            server_acks[1].ack_ids,
            vec!["mass-ack2500".to_owned(), "mass-ack2501".to_owned()]
        );

        // all the futures should get their success response
        for fut in futs {
            assert!(matches!(
                fut.boxed().as_mut().poll(&mut cx),
                Poll::Ready(Ok(()))
            ));
        }

        // the handler future can complete after the sender side is dropped.
        // however it must first flush any acks still in the queue.
        let mut ack_fut = TokenFeedback::send(
            &ack_send,
            ModAck {
                id: "ack-id99".into(),
                deadline: 2,
            },
        )
        .boxed();
        std::mem::drop(ack_send);
        assert!(take_server_acks().is_empty()); // sanity check
        assert!(matches!(
            ack_handler.as_mut().poll(&mut cx),
            Poll::Ready(())
        ));
        assert_eq!(
            take_server_acks(),
            vec![api::ModifyAckDeadlineRequest {
                subscription: "test-subscription".into(),
                ack_ids: vec!["ack-id99".into()],
                ack_deadline_seconds: 2,
            }]
        );
        assert!(matches!(
            ack_fut.as_mut().poll(&mut cx),
            Poll::Ready(Ok(()))
        ));
    }

    #[tokio::test]
    async fn streaming_reqs_stop_drop() {
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        let (stream, stop_drop) = create_streaming_pull_request_stream(
            "test-subscription".into(),
            "test-client".into(),
            StreamSubscriptionConfig::default(),
        );
        let mut stream = stream.boxed();

        // first call always yield the first element
        assert!(matches!(
            stream.as_mut().poll_next(&mut cx),
            Poll::Ready(Some(api::StreamingPullRequest { .. }))
        ));

        // then a periodic element waits for time
        tokio::time::pause();
        assert!(matches!(stream.as_mut().poll_next(&mut cx), Poll::Pending));
        tokio::time::advance(Duration::from_secs(31)).await;
        assert!(matches!(
            stream.as_mut().poll_next(&mut cx),
            Poll::Ready(Some(api::StreamingPullRequest { .. }))
        ));
        // pending on the next one
        assert!(matches!(stream.as_mut().poll_next(&mut cx), Poll::Pending));

        // however dropping the notifier should wake the stream and end it
        std::mem::drop(stop_drop);
        assert!(matches!(
            stream.as_mut().poll_next(&mut cx),
            Poll::Ready(None)
        ));
    }

    // panics in the background ack-handler task should be forwarded _somewhere_
    // ideally it would be to the issuing ack tokens, but it's much easier to pass to the message
    // streamer
    #[tokio::test]
    async fn background_panic_forwarded() {
        use std::panic;
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());

        #[derive(Default, Clone)]
        struct MockSubscriberServer {}

        #[tonic::codegen::async_trait]
        impl api::subscriber_server::Subscriber for MockSubscriberServer {
            async fn acknowledge(
                &self,
                _request: tonic::Request<api::AcknowledgeRequest>,
            ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
                panic!("injected test panic");
            }

            async fn streaming_pull(
                &self,
                _request: tonic::Request<tonic::Streaming<api::StreamingPullRequest>>,
            ) -> std::result::Result<
                tonic::Response<tonic::codegen::BoxStream<api::StreamingPullResponse>>,
                tonic::Status,
            > {
                // send one message in order to provide an ack token. after that, produce no
                // additional messages but don't end the stream, just hang basically
                Ok(tonic::Response::new(
                    async_stream::stream! {
                        yield Ok(api::StreamingPullResponse {
                            received_messages: vec![api::ReceivedMessage {
                                ack_id: "ack1".into(),
                                delivery_attempt: 1,
                                message: Some(api::PubsubMessage {
                                    data: vec![0u8; 16].into(),
                                    ..Default::default()
                                })
                            }],
                            ..Default::default()
                        });

                        future::pending::<()>().await;
                    }
                    .boxed(),
                ))
            }
        }

        let server = MockSubscriberServer::default();

        let mut stream = stream_from_client(
            std::array::from_fn(|_| {
                api::subscriber_client::SubscriberClient::new(
                    api::subscriber_server::SubscriberServer::new(server.clone()),
                )
            }),
            "test-subscription".into(),
            StreamSubscriptionConfig::default(),
            ExponentialBackoff::new(PubSubRetryCheck::default(), Default::default()),
        )
        .boxed();

        // pull the first message out to get an ack token
        let ack_token = match stream.as_mut().poll_next(&mut cx) {
            Poll::Ready(Some(Ok((ack_token, _message)))) => ack_token,
            other => panic!("unexpected stream value {other:?}"),
        };

        // the stream is otherwise empty for now
        assert!(matches!(stream.as_mut().poll_next(&mut cx), Poll::Pending));

        // start an ack from the first message's token. this should trigger a panic in the
        // background ack-handler task once it calls the mocked `acknowledge` function
        let mut ack_fut = ack_token.ack().boxed();

        // give the tokio test runtime an opportunity to run background tasks
        tokio::task::yield_now().await;

        // the stream's poll should now forward that panic to the caller
        match panic::catch_unwind(panic::AssertUnwindSafe(|| {
            stream.as_mut().poll_next(&mut cx)
        })) {
            Ok(poll) => panic!("stream did not panic when expected, instead produced {poll:?}"),
            Err(panic_cause) => match panic_cause.downcast::<&'static str>() {
                Ok(text) => assert_eq!(*text, "injected test panic"),
                Err(_) => panic!("unexpected panic contents"),
            },
        }

        // and the ack future is informed that an error occurred
        assert!(matches!(
            ack_fut.as_mut().poll(&mut cx),
            Poll::Ready(Err(AcknowledgeError(AckErr::BackgroundTaskPanic)))
        ));
    }
}
