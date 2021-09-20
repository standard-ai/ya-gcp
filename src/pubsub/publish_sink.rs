use super::{api, ProjectTopicName};
use crate::{
    auth::grpc::{AuthGrpcService, OAuthTokenSource},
    grpc::StatusCodeSet,
    retry_policy::{exponential_backoff, ExponentialBackoff, RetryOperation, RetryPolicy},
};
use futures::{future::BoxFuture, ready, stream, Sink, SinkExt};
use pin_project::pin_project;
use prost::Message;
use std::{
    cmp::Ordering,
    convert::Infallible,
    error::Error as StdError,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

const MB: usize = 1000 * 1000;

// resource limits imposed by pubsub
// see <https://cloud.google.com/pubsub/quotas#resource_limits>
const MAX_ATTR_PER_MESSAGE: usize = 100;
const MAX_ATTR_KEY_BYTES: usize = 256;
const MAX_ATTR_VALUE_BYTES: usize = 1024;
const MAX_MESSAGES_PER_PUBLISH: usize = 1000;
const MAX_DATA_FIELD_BYTES: usize = 10 * MB;
const MAX_PUBLISH_REQUEST_BYTES: usize = 10 * MB;

type ApiPublisherClient<C> = api::publisher_client::PublisherClient<
    AuthGrpcService<tonic::transport::Channel, OAuthTokenSource<C>>,
>;
type Drain = futures::sink::Drain<api::PubsubMessage>;
type FlushOutput<Si, E> = (Si, Result<(), SinkError<E>>);

/// An error that may occur when publishing batches of messages to PubSub
#[derive(Debug, thiserror::Error)]
#[error("failed to publish message(s)")]
pub struct PublishError {
    /// The source of the error
    #[source]
    pub source: tonic::Status,

    /// The messages which failed to be published
    pub messages: Vec<api::PubsubMessage>,
}

impl From<PublishError> for tonic::Status {
    fn from(err: PublishError) -> Self {
        err.source
    }
}

impl<E> From<PublishError> for SinkError<E> {
    fn from(err: PublishError) -> Self {
        SinkError::Publish(err)
    }
}

/// An error that may occur when submitting messages to the [`PublishTopicSink`]
#[derive(Debug)]
pub enum SinkError<E = Infallible> {
    /// An error from publishing messages to PubSub
    Publish(PublishError),

    /// An error from reporting successful publish responses to the user-provided sink
    Response(E),
}

impl<E: fmt::Display> fmt::Display for SinkError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SinkError::Publish(err) => fmt::Display::fmt(&err, f),
            SinkError::Response(err) => fmt::Display::fmt(&err, f),
        }
    }
}

impl<E: StdError + 'static> StdError for SinkError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            SinkError::Publish(err) => Some(err as &_),
            SinkError::Response(err) => Some(err as &_),
        }
    }
}

// convenience impl for `?` conversion to tonic::Status in the default case of no reporting sink
impl From<SinkError<Infallible>> for tonic::Status {
    fn from(err: SinkError<Infallible>) -> Self {
        match err {
            SinkError::Publish(err) => tonic::Status::from(err),
            SinkError::Response(err) => {
                let _: Infallible = err;
                unreachable!()
            }
        }
    }
}

/// A sink created by [`publish_topic_sink`] for publishing messages to a topic.
///
/// ## Buffering
/// This sink will attempt to buffer messages up to the resource limits outlined in the [PubSub
/// documentation] when using `poll_ready`+`start_send` alone. The `poll_flush` method may be used
/// to publish messages before the buffering limits. Higher-level combinators like [`send_all`] will
/// invoke these methods to meet the combinator's documented behavior.
///
/// ## Errors and Retrying
/// By default, the sink constructed by `publish_topic_sink` will re-attempt certain failed publish
/// requests by using an [exponential backoff] policy, for typically [retriable status codes] and
/// up to a fixed retry count. This behavior can be controlled with the [`with_retry_policy`]
/// method, which will apply a given retry policy to the sink's publish attempts. Should a publish
/// request return an error, the sink will internally try again based on the retry policy, and may
/// succeed in subsequent requests. If the retry attempts reach the policy's limits, a
/// [`PublishError`] will be returned with the messages which failed to be published.
///
/// The user may continue to use the sink after errors are encountered, regardless of the retry
/// policy.
///
/// ## Responses
/// By default, the sink does not report when and which messages are successfully published, only
/// whether errors occured. Calling the [`with_response_sink`] method will attach a sink to the
/// response outputs, which allows users to observe successful publishing. The given sink will be
/// sent elements for each message successfully published by the publishing sink. See the method's
/// documentation for more details.
///
/// [`publish_topic_sink`]: super::PublisherClient::publish_topic_sink
/// [PubSub documentation]: https://cloud.google.com/pubsub/quotas#resource_limits
/// [`send_all`]: futures::SinkExt::send_all
/// [exponential backoff]: crate::retry_policy::ExponentialBackoff
/// [retriable status codes]: crate::pubsub::DEFAULT_RETRY_CODES
/// [`with_retry_policy`]: PublishTopicSink::with_retry_policy
/// [`with_response_sink`]: PublishTopicSink::with_response_sink
#[pin_project(project=PublishTopicSinkProjection)]
pub struct PublishTopicSink<
    C = crate::DefaultConnector,
    Retry = ExponentialBackoff<StatusCodeSet>,
    ResponseSink: Sink<api::PubsubMessage> = Drain,
> {
    /// the underlying client used to execute the requests
    client: ApiPublisherClient<C>,

    /// the publish request currently being populated with messages
    buffer: PublishBuffer,

    /// whether the sink is currently flushing
    #[pin]
    flush_state: FlushState<ResponseSink>,

    /// the retry policy to use when a publish attempt fails
    retry_policy: Retry,

    // reserve the right to be !Unpin in the future without breaking changes
    _pin: std::marker::PhantomPinned,
}

#[pin_project(project=FlushStateProjection, project_replace=FlushStateReplace)]
enum FlushState<ResponseSink: Sink<api::PubsubMessage>> {
    /// The publisher is not currently flushing. When the flush begins, the ResponseSink will be
    /// moved into the flushing future for use; it is moved back to this variant after completion
    NotFlushing(ResponseSink),

    /// A transient state between flushing and not flushing; only used to move variables out of
    /// one variant and into the next
    Transitioning,

    /// The publisher is currently flushing. When the flush completes, it should move the
    /// ResponseSink back to the state so that it can be used by the next flush
    // TODO(type_alias_impl_trait) don't box
    Flushing(#[pin] BoxFuture<'static, FlushOutput<ResponseSink, ResponseSink::Error>>),
}

impl<C> PublishTopicSink<C, ExponentialBackoff<StatusCodeSet>, Drain> {
    /// Create a new `PublishTopicSink` with the default retry policy and no response sink
    pub(super) fn new(client: ApiPublisherClient<C>, topic: ProjectTopicName) -> Self
    where
        C: crate::Connect + Clone + Send + Sync + 'static,
    {
        PublishTopicSink {
            client,
            buffer: PublishBuffer::new(String::from(topic)),
            flush_state: FlushState::NotFlushing(futures::sink::drain()),
            retry_policy: ExponentialBackoff::new(
                crate::pubsub::DEFAULT_RETRY_CODES,
                exponential_backoff::Config::default(),
            ),
            _pin: std::marker::PhantomPinned,
        }
    }
}

impl<C, Retry, ResponseSink: Sink<api::PubsubMessage>> PublishTopicSink<C, Retry, ResponseSink> {
    /// Set the sink to receive successful publishing responses for published messages.
    ///
    /// By default, the `PublishTopicSink` will not report when messages are successfully published.
    /// When a sink is provided to this method, the `PublishTopicSink` will instead send messages to
    /// the given sink after they have been successfully published, in the order in which they
    /// were published.
    ///
    /// The provided messages will have their [`message_id`](api::PubsubMessage::message_id) set to
    /// the value assigned by the server based on the successful response.
    // because using the sink requires Pin<&mut Self>, taking ownership of `self` here prevents
    // switching a retry policy after anything was started.
    //
    // This essentially makes Self a builder during the time before its pinning
    pub fn with_response_sink<Si>(self, sink: Si) -> PublishTopicSink<C, Retry, Si>
    where
        Si: Sink<api::PubsubMessage>,
    {
        PublishTopicSink {
            flush_state: FlushState::NotFlushing(sink),
            retry_policy: self.retry_policy,
            client: self.client,
            buffer: self.buffer,
            _pin: self._pin,
        }
    }

    /// Set the retry policy for this `PublishTopicSink`.
    ///
    /// If a publishing operation encounters an error, the given retry policy will be consulted to
    /// possibly retry the operation, or otherwise propagate the error to the caller.
    // like `with_response_sink`, taking `self` means the sink hasn't done anything yet
    pub fn with_retry_policy<R>(self, retry_policy: R) -> PublishTopicSink<C, R, ResponseSink>
    where
        R: RetryPolicy<api::PublishRequest, tonic::Status>,
    {
        PublishTopicSink {
            retry_policy,
            client: self.client,
            buffer: self.buffer,
            flush_state: self.flush_state,
            _pin: self._pin,
        }
    }
}

impl<C, Retry, ResponseSink> Sink<api::PubsubMessage> for PublishTopicSink<C, Retry, ResponseSink>
where
    C: crate::Connect + Clone + Send + Sync + 'static,
    // TODO(type_alias_impl_trait) remove most of these 'static (and Sync?) bounds
    Retry: RetryPolicy<api::PublishRequest, tonic::Status> + 'static,
    Retry::RetryOp: Send + Sync + 'static,
    <Retry::RetryOp as RetryOperation<api::PublishRequest, tonic::Status>>::Sleep:
        Send + Sync + 'static,
    ResponseSink: Sink<api::PubsubMessage> + Unpin + Send + Sync + 'static,
{
    type Error = SinkError<ResponseSink::Error>;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().poll_flush_projected(cx, false)
    }

    fn start_send(
        mut self: Pin<&mut Self>,
        message: api::PubsubMessage,
    ) -> Result<(), Self::Error> {
        self.buffer.push(message)?;
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().poll_flush_projected(cx, true)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // first flush ourselves, then close the user's sink
        ready!(self.as_mut().poll_flush(cx))?;

        match self.project().flush_state.project() {
            FlushStateProjection::NotFlushing(response_sink) => response_sink
                .poll_close_unpin(cx)
                .map_err(SinkError::Response),
            _ => unreachable!("just flushed, state should be not flushing"),
        }
    }
}

// some methods are implemented against a projection instead of the base type to make nested
// borrowing/pinning in callers easier
impl<'pin, C, Retry, ResponseSink> PublishTopicSinkProjection<'pin, C, Retry, ResponseSink>
where
    C: crate::Connect + Clone + Send + Sync + 'static,
    Retry: RetryPolicy<api::PublishRequest, tonic::Status> + 'static,
    Retry::RetryOp: Send + Sync + 'static,
    <Retry::RetryOp as RetryOperation<api::PublishRequest, tonic::Status>>::Sleep:
        Send + Sync + 'static,
    ResponseSink: Sink<api::PubsubMessage> + Unpin + Send + Sync + 'static,
{
    fn poll_flush_projected(
        &mut self,
        cx: &mut Context<'_>,
        // the caller indicates whether the poll should flush every element possible, even if below
        // the message limit thresholds. `poll_flush` and `poll_close` should because they
        // explicitly flush all messages, `poll_ready` should not because it's only making space
        // for 1 additional `start_send`
        flush_everything: bool,
    ) -> Poll<Result<(), SinkError<ResponseSink::Error>>> {
        use FlushStateProjection::{Flushing, NotFlushing, Transitioning};

        // switch between flushing states using this loop + match + continues
        loop {
            return match self.flush_state.as_mut().project() {
                Transitioning => {
                    // The transitioning state is only observable if the transition to flushing was
                    // interrupted by a panic (possible from user code in `clone` or retry setup)
                    panic!("the sink cannot be used after recovering from a panic")
                }

                // if currently flushing, try to drive the flush to completion
                Flushing(mut flush_fut) => {
                    // if the flush is ready, switch the state back to not flushing
                    let (response_sink, flush_result) = ready!(flush_fut.as_mut().poll(cx));
                    self.flush_state.set(FlushState::NotFlushing(response_sink));

                    // if there was an error, propagate it after switching the state
                    flush_result?;

                    // the flush's buffer swap may have pushed an over-limits element into the next
                    // buffer, now waiting for the next flush. Loop back to re-check the flushing
                    // state and possibly initiate another flush
                    continue;
                }

                // if not flushing and there's nothing to flush, then we're Ready
                NotFlushing(_) if self.buffer.is_empty() => Poll::Ready(Ok(())),

                // if not flushing and we have space for another message, return Ready if the
                // caller doesn't want to flush everything (i.e. poll_ready)
                NotFlushing(_)
                    if !flush_everything
                        && self.buffer.compare_flush_threshold() == Ordering::Less =>
                {
                    Poll::Ready(Ok(()))
                }

                // the last case is initiating a flush because we're past the threshold, or the
                // caller asked to flush everything
                NotFlushing(_) => {
                    // swap out the buffered request for a new empty one
                    let request = self.buffer.swap_buffers()?;

                    // move the ResponseSink from the state enum with a transition replacement
                    let response_sink = match self
                        .flush_state
                        .as_mut()
                        .project_replace(FlushState::Transitioning)
                    {
                        FlushStateReplace::NotFlushing(response_sink) => response_sink,
                        _ => unreachable!("already checked state"),
                    };

                    // construct the flush future
                    let flush_fut =
                        Self::flush(self.client, request, response_sink, self.retry_policy);

                    // transition the state
                    self.flush_state
                        .set(FlushState::Flushing(Box::pin(flush_fut)));

                    // loop back with the new state to start polling the flush
                    continue;
                }
            };
        }
    }

    fn flush(
        client: &mut ApiPublisherClient<C>,
        mut request: api::PublishRequest,
        mut response_sink: ResponseSink,
        retry_policy: &mut Retry,
    ) -> impl Future<Output = FlushOutput<ResponseSink, ResponseSink::Error>> {
        // until Sink gets syntax sugar like generators, internal futures can't borrow (safely) and
        // have to own their referents
        let mut client = client.clone();

        // start a new potential retry operation for the publish
        let mut retry = retry_policy.new_operation();

        async move {
            // send the request, with some potential retries
            let response = loop {
                // the request unfortunately has to be cloned -- tonic can't take references
                // because it requires 'static payloads (probably for spawning?), and we need the
                // original to reuse for retries/errors/responses. On the bright side, message
                // payloads are Bytes which are essentially Arc'ed, so alloc clones mostly apply to
                // the attribute maps
                break match client.publish(request.clone()).await {
                    Ok(response) => Ok(response.into_inner()),
                    // if the status code is retry worthy, poll the retry policy to backoff
                    // before retrying, or stop retrying if the backoff is exhausted.
                    Err(status) => match retry.check_retry(&request, &status) {
                        // if the retry is non-empty, await the backoff then loop back to retry
                        Some(sleep) => {
                            sleep.await;
                            continue;
                        }
                        // otherwise we hit the policy's retry limit. return this error as-is
                        None => Err(status),
                    },
                };
            };

            let response = match response {
                Ok(response) => response,
                Err(status) => {
                    return (
                        response_sink,
                        Err(SinkError::Publish(PublishError {
                            source: status,
                            messages: request.messages,
                        })),
                    );
                }
            };

            // quick sanity check on length parity before zip
            if response.message_ids.len() != request.messages.len() {
                return (
                    response_sink,
                    Err(SinkError::Publish(PublishError {
                        source: tonic::Status::internal(format!(
                            "publish response had {} ids when expected {}",
                            response.message_ids.len(),
                            request.messages.len()
                        )),
                        messages: request.messages,
                    })),
                );
            }

            // The response is composed of a message id list, with ids corresponding to the
            // messages in the outgoing message list. Assign those ids to the corresponding
            // messages
            for (message, message_id) in request.messages.iter_mut().zip(response.message_ids) {
                message.message_id = message_id;
            }

            // send the batch of newly id-ed messages out to the response sink
            let response_result = response_sink
                .send_all(&mut stream::iter(request.messages.into_iter().map(Ok)))
                .await
                .map_err(SinkError::Response);

            (response_sink, response_result)
        }
    }
}

/// A buffer to hold several sink messages before flushing them in a batch as a single request
struct PublishBuffer {
    /// The actual buffer is the message list of this request
    request: api::PublishRequest,

    /// A measure of the protobuf encoded size of the currently buffered request.
    ///
    /// This will always equal `request.encoded_len()`; it's cached in this field because
    /// the calculation is not trivial, and the len check is performed often to assess whether to
    /// flush
    encoded_len: usize,
}

impl PublishBuffer {
    fn new(topic: String) -> Self {
        let request = api::PublishRequest {
            topic,
            messages: Vec::with_capacity(MAX_MESSAGES_PER_PUBLISH),
        };

        let buffer = PublishBuffer {
            encoded_len: request.encoded_len(),
            request,
        };

        // check that the topic is not insanely large
        if buffer.compare_flush_threshold() > Ordering::Less {
            panic!(
                "publish request exceeds size limit with topic alone ({} bytes)",
                buffer.encoded_len
            );
        }

        buffer
    }

    fn is_empty(&self) -> bool {
        self.request.messages.is_empty()
    }

    /// Check whether the currently buffered request has hit the flushing threshold
    fn compare_flush_threshold(&self) -> Ordering {
        let messages = self.request.messages.len();
        let bytes = self.encoded_len;

        if messages > MAX_MESSAGES_PER_PUBLISH || bytes > MAX_PUBLISH_REQUEST_BYTES {
            Ordering::Greater
        } else if messages == MAX_MESSAGES_PER_PUBLISH || bytes == MAX_PUBLISH_REQUEST_BYTES {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }

    fn push(&mut self, message: api::PubsubMessage) -> Result<(), PublishError> {
        macro_rules! check_argument {
            ($value:expr, $limit:expr, $fmt_string:literal) => {
                check_argument!($value, $value, $limit, $fmt_string)
            };

            ($value_str:expr, $value:expr, $limit:expr, $fmt_string:literal) => {{
                let value_str = $value_str;
                let value = $value;
                let limit = $limit;
                if value > limit {
                    return Err(PublishError {
                        source: tonic::Status::invalid_argument(format!(
                            $fmt_string,
                            value_str, limit
                        )),
                        messages: vec![message],
                    });
                }
            }};
        }

        check_argument!(
            message.attributes.len(),
            MAX_ATTR_PER_MESSAGE,
            "message has {} attributes, exceeds max {}"
        );
        check_argument!(
            message.data.len(),
            MAX_DATA_FIELD_BYTES,
            "message has {} data bytes, exceeds max {} bytes"
        );

        for (key, value) in message.attributes.iter() {
            check_argument!(
                &key,
                key.len(),
                MAX_ATTR_KEY_BYTES,
                "message attribute key {} exceeds max {} bytes"
            );
            check_argument!(
                &value,
                value.len(),
                MAX_ATTR_VALUE_BYTES,
                "message attribute value {} exceeds max {} bytes"
            );
        }

        self.request.messages.push(message);
        self.encoded_len = self.request.encoded_len();

        Ok(())
    }

    fn pop(&mut self) -> Option<api::PubsubMessage> {
        let msg = self.request.messages.pop();
        self.encoded_len = self.request.encoded_len();
        msg
    }

    /// Switch the currently buffering request for a new one, returning the filled request for
    /// publishing.
    fn swap_buffers(&mut self) -> Result<api::PublishRequest, PublishError> {
        // create the new buffer to switch to
        let mut new_messages = Vec::with_capacity(MAX_MESSAGES_PER_PUBLISH);

        // Because elements are inserted with `start_send` which by itself cannot initiate a flush,
        // it's possible that an insertion put the buffer over the resource maximums and we won't
        // learn about it until the next `poll_ready` or `poll_flush`. Before swapping the buffers,
        // check if there is such an element which pushed past the limits and try to move it to the
        // next buffer, bringing the current request back under limits.

        // check if the buffer has passed the resource limits
        if self.compare_flush_threshold() == Ordering::Greater {
            let len_before_pop = self.encoded_len;
            match self.pop() {
                None => {
                    // if there are no messages, the encoded_len check tripped the threshold. That
                    // means the request is over 10MB but contains... only the topic string?
                    //
                    // This case was checked in the buffer constructor, so it should not be
                    // reachable now
                    unreachable!()
                }

                Some(message) if self.is_empty() => {
                    // if there was exactly one message (implied by an empty buffer after popping),
                    // the encoded_len check tripped the threshold. Pushing this message to the
                    // next buffer would leave an empty request, and the next buffer would try to
                    // pass this message again, pop+push+send another empty request, ad infinitum.
                    // That's not good!
                    //
                    // The problem is that a single message may have a data payload up to 10MB
                    // (checked in start_send); but the limit on a request's total size is *also*
                    // 10MB (checked here), so a 10MB payload + some headers could exceed the
                    // request limit. That must have happened in this case.

                    return Err(PublishError {
                        source: tonic::Status::invalid_argument(format!(
                            "submitted message which exceeds request size limit ({} bytes, limit \
                             {})",
                            len_before_pop, MAX_PUBLISH_REQUEST_BYTES,
                        )),
                        messages: vec![message],
                    });
                }

                Some(message) => {
                    // All other cases are typical, where some single message sent the request over
                    // the count or byte limit

                    new_messages.push(message);

                    // The assumption that a single message put the request over the limits relies
                    // on the Sink contract that each `start_send` is preceded by a successful
                    // `poll_ready`: our poll_ready checks the flush threshold, so it would
                    // prohibit subsequent start_sends after one has breached the limits by
                    // initiating a flush (ending up in this function).
                    //
                    // However, it's possible for a user to violate this contract and invoke
                    // start_send arbitrarily. Check that and panic if so
                    assert!(
                        self.compare_flush_threshold() == Ordering::Less,
                        "start_send was previously called without a corresponding poll_ready"
                    );
                }
            }
        }

        // having reduced the buffer to the resource limits, swap it out for the new one

        let new_request = api::PublishRequest {
            topic: self.request.topic.clone(),
            messages: new_messages,
        };

        let old_buffer = std::mem::replace(
            self,
            PublishBuffer {
                encoded_len: new_request.encoded_len(),
                request: new_request,
            },
        );

        Ok(old_buffer.request)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(feature = "emulators")]
    use crate::pubsub;
    use futures::{channel::mpsc, SinkExt, StreamExt};
    use std::{collections::BTreeMap, iter};

    impl<E: PartialEq> PartialEq for SinkError<E> {
        fn eq(&self, other: &SinkError<E>) -> bool {
            use SinkError::{Publish, Response};

            match (self, other) {
                (
                    Publish(PublishError {
                        source: status1,
                        messages: messages1,
                    }),
                    Publish(PublishError {
                        source: status2,
                        messages: messages2,
                    }),
                ) => {
                    // equality of status codes and failed messages should be enough for tests.
                    // could check status error message equality too, but that's likely unnecessary
                    // complexity
                    status1.code() == status2.code() && messages1 == messages2
                }

                (Response(err1), Response(err2)) => err1 == err2,

                _ => false,
            }
        }
    }

    fn dummy_client() -> ApiPublisherClient<crate::DefaultConnector> {
        // by connecting to the endpoint lazily, this client will only work until the first request
        // is made (then it will error). That's good enough for testing certain functionality
        // that doesn't require the requests themselves, like validity checking
        ApiPublisherClient::new(crate::auth::grpc::oauth_grpc(
            tonic::transport::channel::Endpoint::from_static("https://localhost")
                .connect_lazy()
                .unwrap(),
            None,
            vec![],
        ))
    }

    fn dummy_sink() -> PublishTopicSink {
        PublishTopicSink::new(
            dummy_client(),
            ProjectTopicName::new("dummy-project", "dummy-topic"),
        )
    }

    #[tokio::test]
    async fn too_many_attributes() {
        let mut sink = Box::pin(dummy_sink());

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        assert_eq!(Poll::Ready(Ok(())), sink.poll_ready_unpin(&mut cx));

        // make one too many attributes
        let attributes = (0..(MAX_ATTR_PER_MESSAGE + 1))
            .map(|n| (n.to_string(), n.to_string()))
            .collect::<BTreeMap<String, String>>();

        let message = api::PubsubMessage {
            attributes,
            ..api::PubsubMessage::default()
        };

        assert_eq!(
            Err(SinkError::Publish(PublishError {
                source: tonic::Status::invalid_argument(""),
                messages: vec![message.clone()]
            })),
            sink.start_send_unpin(message)
        );
    }

    #[tokio::test]
    async fn too_long_attribute_key() {
        let mut sink = Box::pin(dummy_sink());

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        assert_eq!(Poll::Ready(Ok(())), sink.poll_ready_unpin(&mut cx));

        // make an attribute with too long of a key
        let attributes = {
            let mut map = BTreeMap::new();
            map.insert(
                iter::repeat('a')
                    .take(MAX_ATTR_KEY_BYTES + 1)
                    .collect::<String>(),
                "".into(),
            );
            map
        };

        let message = api::PubsubMessage {
            attributes,
            ..api::PubsubMessage::default()
        };

        assert_eq!(
            Err(SinkError::Publish(PublishError {
                source: tonic::Status::invalid_argument(""),
                messages: vec![message.clone()]
            })),
            sink.start_send_unpin(message)
        );
    }

    #[tokio::test]
    async fn too_long_attribute_value() {
        let mut sink = Box::pin(dummy_sink());

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        assert_eq!(Poll::Ready(Ok(())), sink.poll_ready_unpin(&mut cx));

        // make an attribute with too long of a value
        let attributes = {
            let mut map = BTreeMap::new();
            map.insert(
                "".into(),
                iter::repeat('a')
                    .take(MAX_ATTR_VALUE_BYTES + 1)
                    .collect::<String>(),
            );
            map
        };

        let message = api::PubsubMessage {
            attributes,
            ..api::PubsubMessage::default()
        };

        assert_eq!(
            Err(SinkError::Publish(PublishError {
                source: tonic::Status::invalid_argument(""),
                messages: vec![message.clone()]
            })),
            sink.start_send_unpin(message)
        );
    }

    #[tokio::test]
    async fn too_large_message_data() {
        let mut sink = Box::pin(dummy_sink());

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        assert_eq!(Poll::Ready(Ok(())), sink.poll_ready_unpin(&mut cx));

        // make a message with too much data
        let message = api::PubsubMessage {
            data: vec![0; MAX_DATA_FIELD_BYTES + 1].into(),
            ..api::PubsubMessage::default()
        };

        assert_eq!(
            Err(SinkError::Publish(PublishError {
                source: tonic::Status::invalid_argument(""),
                messages: vec![message.clone()]
            })),
            sink.start_send_unpin(message)
        );
    }

    /// It's possible to construct a message whose combined size is too large, despite the
    /// individual components being within limits
    #[tokio::test]
    async fn too_large_combined_message_size() {
        let mut sink = Box::pin(dummy_sink());

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        assert_eq!(Poll::Ready(Ok(())), sink.poll_ready_unpin(&mut cx));

        // make a message with *almost* too much data, and a few attributes to push it over the
        // total message size limit
        let message = api::PubsubMessage {
            data: vec![0; MAX_DATA_FIELD_BYTES - 1].into(),
            attributes: iter::once(("a".to_string(), "b".to_string())).collect(),
            ..api::PubsubMessage::default()
        };

        // the total size check is not done in `start_send`, but rather in each flush attempt.
        // Inserting this message will succeed, but the next `poll_ready` will error because it
        // attempts a flush (for lack of space) and it can't flush such a large message
        assert_eq!(Ok(()), sink.start_send_unpin(message.clone()));
        assert_eq!(
            Poll::Ready(Err(SinkError::Publish(PublishError {
                source: tonic::Status::invalid_argument(""),
                messages: vec![message],
            }))),
            sink.poll_ready_unpin(&mut cx)
        );
    }

    /// `poll_ready` is required between `start_send`s -- this isn't checked on every send, but
    /// there are particular cases around flush thresholds where errors will be thrown if the user
    /// didn't check `poll_ready` appropriately.
    ///
    /// In particular, `poll_ready` checks that at any given time there will be no more than 1
    /// "excess" message in the buffer (this excess is a consequence of `start_send` not being able
    /// to initiate flushes, and a subsequent `poll_ready` is required). That is, it should always
    /// be possible to flush the buffer and have at most 1 message that couldn't fit within the
    /// limits. This helps to enforce the "total message size limits" constraint, so that checking
    /// an insertion's total size only needs to be isolated to a single overflow message.
    #[tokio::test]
    async fn did_not_poll_ready() {
        let mut sink = Box::pin(dummy_sink());

        // make a message which would fit in a single publish, but multiple such messages would
        // trigger segmentation and intermediate flushes
        let message = api::PubsubMessage {
            data: vec![0; MAX_DATA_FIELD_BYTES * 2 / 3].into(),
            ..api::PubsubMessage::default()
        };

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());

        // prime the sink with a normal start_send
        assert_eq!(Poll::Ready(Ok(())), sink.poll_ready_unpin(&mut cx));
        assert_eq!(Ok(()), sink.start_send_unpin(message.clone()));

        // erroneously call `start_send` without intervening `poll_ready`s.
        // One additional call would not be enough to trigger the panic because having 1 excess
        // message is normal for the poll_ready check. Two excess messages will trigger the panic
        assert_eq!(Ok(()), sink.start_send_unpin(message.clone()));
        assert_eq!(Ok(()), sink.start_send_unpin(message));

        // check that the panic happens on poll_ready
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || sink.poll_ready_unpin(&mut cx)
        ))
        .is_err());
    }

    #[test]
    #[should_panic]
    fn buffer_topic_too_large() {
        let too_large_topic = std::iter::repeat('a')
            .take(MAX_PUBLISH_REQUEST_BYTES)
            .collect::<String>();
        PublishBuffer::new(too_large_topic);
    }

    /// Check that publish responses are issued in the same order as messages are received. This
    /// includes across publish request segmentation boundaries
    #[cfg(feature = "emulators")]
    #[tokio::test]
    async fn message_responses_in_order() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let emulator = pubsub::emulator::EmulatorClient::new().await?;
        let project_topic =
            ProjectTopicName::new(emulator.project(), "message_responses_in_order_test");
        let project_subscription = pubsub::ProjectSubscriptionName::new(
            emulator.project(),
            "message_responses_in_order_test_subscription",
        );

        let pubsub_config = pubsub::PubSubConfig {
            endpoint: emulator.endpoint(),
            ..pubsub::PubSubConfig::default()
        };

        let mut publisher = emulator
            .builder()
            .build_pubsub_publisher(pubsub_config.clone())
            .await?;

        publisher
            .create_topic(api::Topic {
                name: project_topic.clone().into(),
                ..api::Topic::default()
            })
            .await?;

        let mut subscriber = emulator
            .builder()
            .build_pubsub_subscriber(pubsub_config)
            .await?;

        subscriber
            .create_subscription(api::Subscription {
                name: project_subscription.clone().into(),
                topic: project_topic.clone().into(),
                ..api::Subscription::default()
            })
            .await?;

        let (response_sink, responses) = futures::channel::mpsc::unbounded();
        let publish_topic_sink = publisher
            .publish_topic_sink(project_topic.clone())
            .with_response_sink(response_sink);

        // send 10 messages, each large enough such that at most 4 messages fit into a single
        // publish request. This will require segmentation and exercise under/over fill
        futures::stream::iter(0..10)
            .map(|n| api::PubsubMessage {
                data: vec![n; MAX_PUBLISH_REQUEST_BYTES / 4].into(),
                attributes: iter::once(("msg_num".into(), format!("{}", n))).collect(),
                ..api::PubsubMessage::default()
            })
            .map(Ok)
            .forward(publish_topic_sink)
            .await?;

        let sent_messages = responses.collect::<Vec<_>>().await;

        let mut received_messages = Vec::new();
        while received_messages.len() < sent_messages.len() {
            received_messages.extend(
                subscriber
                    .pull(api::PullRequest {
                        subscription: project_subscription.clone().into(),
                        max_messages: 10,
                        ..api::PullRequest::default()
                    })
                    .await?
                    .into_inner()
                    .received_messages
                    .into_iter()
                    .filter_map(|msg| {
                        Some(api::PubsubMessage {
                            // strip the publish time so that the eq check is easy
                            publish_time: None,
                            ..msg.message?
                        })
                    }),
            );
        }

        assert_eq!(sent_messages, received_messages);

        Ok(())
    }

    /// Ensure that the user's response sink is closed when the publishing sink is closed without
    /// having to flush
    #[tokio::test]
    async fn user_sink_closed_no_flush() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (user_sink, mut user_sink_receiver) = mpsc::unbounded();
        let mut publish_sink = dummy_sink().with_response_sink(user_sink);

        // sanity check that the sink is initially not terminated
        assert_eq!(Poll::Pending, futures::poll!(user_sink_receiver.next()));

        publish_sink.close().await?;

        assert_eq!(Poll::Ready(None), futures::poll!(user_sink_receiver.next()));

        Ok(())
    }

    /// Ensure that the user's response sink is closed when the publishing sink is closed while a
    /// message is waiting to flush
    #[cfg(feature = "emulators")]
    #[tokio::test]
    async fn user_sink_closed_with_flush() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let emulator = pubsub::emulator::EmulatorClient::new().await?;
        let project_topic = ProjectTopicName::new(emulator.project(), "user_sink_test");

        let mut publisher = emulator
            .builder()
            .build_pubsub_publisher(pubsub::PubSubConfig {
                endpoint: emulator.endpoint(),
                ..pubsub::PubSubConfig::default()
            })
            .await?;

        publisher
            .create_topic(api::Topic {
                name: project_topic.clone().into(),
                ..api::Topic::default()
            })
            .await?;

        let (user_sink, mut user_sink_receiver) = futures::channel::mpsc::unbounded();
        let mut publish_sink = publisher
            .publish_topic_sink(project_topic.clone())
            .with_response_sink(user_sink);

        let message = api::PubsubMessage {
            data: "test-data".as_bytes().to_vec().into(),
            ..api::PubsubMessage::default()
        };

        publish_sink.feed(message.clone()).await?;

        // sanity check that the sink is initially not terminated and hasn't received the element
        // because it hasn't been flushed
        assert_eq!(Poll::Pending, futures::poll!(user_sink_receiver.next()));

        // the fed message will be buffered waiting for a flush. closing contractually requires a
        // flush, so this close should send that message before closing the user sink
        publish_sink.close().await?;

        // The response sink should have received the message from the flush
        assert_eq!(
            Poll::Ready(Some(message)),
            futures::poll!(user_sink_receiver.next()).map(|msg| Some(api::PubsubMessage {
                // erase the id because it's assigned only after flush
                message_id: "".into(),
                ..msg?
            }))
        );

        // The sink should then have been terminated
        assert_eq!(Poll::Ready(None), futures::poll!(user_sink_receiver.next()));

        Ok(())
    }
}
