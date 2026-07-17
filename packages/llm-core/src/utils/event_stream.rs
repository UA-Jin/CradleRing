// LLM core event-stream implementations.
// 翻译自 packages/llm-core/src/utils/event-stream.ts
//
// 提供 `AssistantMessageEventStream` 作为 `push` / `end` / `result` 容器。
// `split()` 方法将内部 mpsc sender 暴露给消费者；生产者侧使用 handle
// 推送事件，最终通过 `result()` 等待终态消息。

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex as AsyncMutex};

use crate::types::{
    AssistantMessage, AssistantMessageEvent, AssistantMessageEventStreamContract,
};

/// State of the stream's terminal result.
struct StreamState<R> {
    done: bool,
    final_result: Option<R>,
}

impl<R> Default for StreamState<R> {
    fn default() -> Self {
        Self {
            done: false,
            final_result: None,
        }
    }
}

/// Generic async-iterable event stream with a separately awaited final result.
pub struct EventStream<T: Clone + Send + 'static, R: Clone + Send + 'static> {
    state: Arc<AsyncMutex<StreamState<R>>>,
    is_complete: Arc<dyn Fn(&T) -> bool + Send + Sync>,
    extract_result: Arc<dyn Fn(&T) -> R + Send + Sync>,
}

impl<T: Clone + Send + 'static, R: Clone + Send + 'static> EventStream<T, R> {
    pub fn new(is_complete: Box<dyn Fn(&T) -> bool + Send + Sync>, extract_result: Box<dyn Fn(&T) -> R + Send + Sync>) -> Self {
        Self {
            state: Arc::new(AsyncMutex::new(StreamState::default())),
            is_complete: Arc::new(is_complete),
            extract_result: Arc::new(extract_result),
        }
    }

    pub async fn mark_complete(&self, event: &T) -> Option<R> {
        if (self.is_complete)(event) {
            let result = (self.extract_result)(event);
            let mut state = self.state.lock().await;
            state.done = true;
            state.final_result = Some(result.clone());
            Some(result)
        } else {
            None
        }
    }

    pub async fn end(&self, result: Option<R>) {
        let mut state = self.state.lock().await;
        state.done = true;
        if let Some(r) = result {
            state.final_result = Some(r);
        }
    }

    pub async fn result(&self) -> R {
        loop {
            {
                let state = self.state.lock().await;
                if let Some(r) = state.final_result.clone() {
                    return r;
                }
            }
            tokio::task::yield_now().await;
        }
    }
}

/// Producer-side handle paired with a receiver returned by `split`.
pub struct AssistantMessageEventStreamHandle {
    inner: EventStream<AssistantMessageEvent, AssistantMessage>,
    sender: mpsc::UnboundedSender<AssistantMessageEvent>,
}

impl AssistantMessageEventStreamHandle {
    pub async fn push(&self, event: AssistantMessageEvent) {
        self.inner.mark_complete(&event).await;
        let _ = self.sender.send(event);
    }

    pub async fn end(&self, result: Option<AssistantMessage>) {
        self.inner.end(result).await;
    }

    pub async fn result(&self) -> AssistantMessage {
        self.inner.result().await
    }
}

/// Assistant-message event stream.
pub struct AssistantMessageEventStream {
    inner: EventStream<AssistantMessageEvent, AssistantMessage>,
    push_tx: mpsc::UnboundedSender<AssistantMessageEvent>,
    push_rx: AsyncMutex<Option<mpsc::UnboundedReceiver<AssistantMessageEvent>>>,
}

impl AssistantMessageEventStream {
    pub fn new() -> Self {
        let is_complete: Box<dyn Fn(&AssistantMessageEvent) -> bool + Send + Sync> =
            Box::new(|event| {
                matches!(
                    event,
                    AssistantMessageEvent::Done { .. } | AssistantMessageEvent::Error { .. }
                )
            });
        let extract_result: Box<
            dyn Fn(&AssistantMessageEvent) -> AssistantMessage + Send + Sync,
        > = Box::new(|event| match event {
            AssistantMessageEvent::Done { message, .. } => message.clone(),
            AssistantMessageEvent::Error { error, .. } => error.clone(),
            _ => panic!("Unexpected event type for final result"),
        });
        let inner = EventStream::new(is_complete, extract_result);
        let (push_tx, push_rx) = mpsc::unbounded_channel();
        Self {
            inner,
            push_tx,
            push_rx: AsyncMutex::new(Some(push_rx)),
        }
    }

    /// Split into a producer handle and consumer receiver.
    pub async fn split(
        &self,
    ) -> (
        AssistantMessageEventStreamHandle,
        mpsc::UnboundedReceiver<AssistantMessageEvent>,
    ) {
        let rx = {
            let mut guard = self.push_rx.lock().await;
            guard.take().expect("split called more than once")
        };
        let handle = AssistantMessageEventStreamHandle {
            inner: EventStream {
                state: self.inner.state.clone(),
                is_complete: self.inner.is_complete.clone(),
                extract_result: self.inner.extract_result.clone(),
            },
            sender: self.push_tx.clone(),
        };
        (handle, rx)
    }
}

impl Default for AssistantMessageEventStream {
    fn default() -> Self {
        Self::new()
    }
}

impl AssistantMessageEventStreamContract for AssistantMessageEventStream {
    fn push(&self, event: AssistantMessageEvent) {
        let inner = &self.inner;
        let sender = self.push_tx.clone();
        let is_complete = inner.is_complete.clone();
        let extract_result = inner.extract_result.clone();
        let state = inner.state.clone();
        let event_clone = event.clone();
        // Synchronous push: spawn a task to update terminal state, then send.
        tokio::spawn(async move {
            if (is_complete)(&event_clone) {
                let result = (extract_result)(&event_clone);
                let mut guard = state.lock().await;
                guard.done = true;
                guard.final_result = Some(result);
            }
        });
        let _ = sender.send(event);
    }

    fn end(&self, result: Option<AssistantMessage>) {
        let state = self.inner.state.clone();
        tokio::spawn(async move {
            let mut guard = state.lock().await;
            guard.done = true;
            if let Some(r) = result {
                guard.final_result = Some(r);
            }
        });
    }

    fn result(&self) -> Pin<Box<dyn Future<Output = AssistantMessage> + Send>> {
        let state = self.inner.state.clone();
        Box::pin(async move {
            loop {
                {
                    let guard = state.lock().await;
                    if let Some(r) = guard.final_result.clone() {
                        return r;
                    }
                }
                tokio::task::yield_now().await;
            }
        })
    }
}

/// Creates an assistant-message stream for provider and plugin adapters.
pub fn create_assistant_message_event_stream() -> AssistantMessageEventStream {
    AssistantMessageEventStream::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AssistantMessage, Usage, UsageCost};

    fn dummy_message() -> AssistantMessage {
        AssistantMessage {
            role: "assistant".to_string(),
            content: vec![],
            api: "openai-completions".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            response_model: None,
            response_id: None,
            diagnostics: None,
            usage: Usage {
                input: 0,
                output: 0,
                cache_read: 0,
                cache_write: 0,
                context_usage: None,
                total_tokens: 0,
                cost: UsageCost {
                    input: 0.0,
                    output: 0.0,
                    cache_read: 0.0,
                    cache_write: 0.0,
                    total: 0.0,
                    total_origin: None,
                },
            },
            stop_reason: "stop".to_string(),
            error_message: None,
            error_code: None,
            error_type: None,
            error_body: None,
            timestamp: 0,
        }
    }

    #[tokio::test]
    async fn event_stream_done_resolves_result() {
        let stream = AssistantMessageEventStream::new();
        let (handle, mut rx) = stream.split().await;
        let message = dummy_message();
        let push_task = tokio::spawn(async move {
            handle
                .push(AssistantMessageEvent::Done {
                    reason: "stop".to_string(),
                    message: message.clone(),
                })
                .await;
            handle.end(None).await;
        });
        let event = rx.recv().await.unwrap();
        match event {
            AssistantMessageEvent::Done { message, .. } => {
                assert_eq!(message.stop_reason, "stop")
            }
            _ => panic!("expected done"),
        }
        push_task.await.unwrap();
    }
}