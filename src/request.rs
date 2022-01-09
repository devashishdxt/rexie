use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use js_sys::Function;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    oneshot,
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, IdbCursorWithValue, IdbOpenDbRequest, IdbRequest, IdbTransaction};

use crate::Error;

/// Waits for a request to finish.
pub async fn wait_request<R: DbRequest + Unpin>(
    request: R,
    map_err: fn(JsValue) -> Error,
) -> crate::Result<JsValue> {
    RequestFuture::new(request, map_err).await
}

/// Waits for transaction to abort.
pub async fn wait_transaction_abort(transaction: IdbTransaction) -> crate::Result<()> {
    let (sender, receiver) = oneshot::channel();

    let abort_closure: Closure<dyn FnMut(Event)> = Closure::once(move |_| {
        sender
            .send(())
            .map_err(|_| Error::AsyncChannelError)
            .unwrap_throw();
    });

    transaction.set_onabort(Some(get_callback(&abort_closure)));

    receiver.await.map_err(|_| Error::AsyncChannelError)
}

/// Waits for cursor to finish.
pub async fn wait_cursor_request(
    request: IdbRequest,
    limit: Option<u32>,
    offset: Option<u32>,
    map_err: fn(JsValue) -> Error,
) -> crate::Result<Vec<(JsValue, JsValue)>> {
    let offset = offset.unwrap_or_default();

    let advancing = if offset == 0 {
        Arc::new(AtomicBool::new(false))
    } else {
        Arc::new(AtomicBool::new(true))
    };

    let seen = Arc::new(AtomicU32::new(0));

    let (sender, mut receiver) = mpsc::unbounded_channel();

    let cursor_closure = get_cursor_closure(sender, seen, advancing, limit, offset);
    request.set_onsuccess(Some(get_callback(&cursor_closure)));

    let mut result = Vec::new();

    while let Some(pair) = receiver.recv().await {
        match pair.map_err(map_err)? {
            CursorAction::Break => break,
            CursorAction::BreakWithValue(key, value) => {
                result.push((key, value));
                break;
            }
            CursorAction::Continue => continue,
            CursorAction::ContinueWithValue(key, value) => result.push((key, value)),
        }
    }

    Ok(result)
}

pub enum CursorAction {
    Break,
    BreakWithValue(JsValue, JsValue),
    Continue,
    ContinueWithValue(JsValue, JsValue),
}

pub trait DbRequest {
    fn on_success(&self, callback: Option<&Function>);
    fn on_error(&self, callback: Option<&Function>);
}

impl DbRequest for IdbRequest {
    fn on_success(&self, callback: Option<&Function>) {
        self.set_onsuccess(callback);
    }

    fn on_error(&self, callback: Option<&Function>) {
        self.set_onerror(callback);
    }
}

impl DbRequest for IdbOpenDbRequest {
    fn on_success(&self, callback: Option<&Function>) {
        self.set_onsuccess(callback);
    }

    fn on_error(&self, callback: Option<&Function>) {
        self.set_onerror(callback);
    }
}

impl DbRequest for IdbTransaction {
    fn on_success(&self, callback: Option<&Function>) {
        self.set_oncomplete(callback);
    }

    fn on_error(&self, callback: Option<&Function>) {
        self.set_onerror(callback);
    }
}

#[must_use = "futures do nothing unless polled or spawned"]
pub struct RequestFuture<R>
where
    R: DbRequest + Unpin,
{
    _inner: R,
    _success_closure: Closure<dyn FnMut(Event)>,
    _error_closure: Closure<dyn FnMut(Event)>,
    receiver: UnboundedReceiver<Result<JsValue, JsValue>>,
    map_err: fn(JsValue) -> Error,
}

impl<R> RequestFuture<R>
where
    R: DbRequest + Unpin,
{
    pub fn new(request: R, map_err: fn(JsValue) -> Error) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let success_closure = get_success_closure(sender.clone());
        let error_closure = get_error_closure(sender);

        request.on_success(Some(get_callback(&success_closure)));
        request.on_error(Some(get_callback(&error_closure)));

        Self {
            _inner: request,
            _success_closure: success_closure,
            _error_closure: error_closure,
            receiver,
            map_err,
        }
    }
}

impl<R> Future for RequestFuture<R>
where
    R: DbRequest + Unpin,
{
    type Output = crate::Result<JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.receiver.poll_recv(cx).map(|option| match option {
            None => Err(Error::AsyncChannelError),
            Some(Ok(value)) => Ok(value),
            Some(Err(err)) => Err((self.map_err)(err)),
        })
    }
}

fn get_success_closure(
    sender: UnboundedSender<Result<JsValue, JsValue>>,
) -> Closure<dyn FnMut(Event)> {
    Closure::once(move |event: Event| {
        let target = event.target().unwrap_throw();
        let request: &IdbRequest = AsRef::<JsValue>::as_ref(&target).unchecked_ref();

        sender
            .send(request.result())
            .map_err(|_| Error::AsyncChannelError)
            .unwrap_throw();
    })
}

fn get_error_closure(
    sender: UnboundedSender<Result<JsValue, JsValue>>,
) -> Closure<dyn FnMut(Event)> {
    Closure::once(move |event: Event| {
        let target = event.target().unwrap_throw();
        let request: &IdbRequest = AsRef::<JsValue>::as_ref(&target).unchecked_ref();

        let error: Result<JsValue, JsValue> = match request.error() {
            Ok(Some(exception)) => Err(exception.into()),
            Ok(None) => Err(Error::DomExceptionNotFound.into()),
            Err(error) => Err(error),
        };

        sender
            .send(error)
            .map_err(|_| Error::AsyncChannelError)
            .unwrap_throw();
    })
}

fn get_cursor_closure(
    sender: UnboundedSender<Result<CursorAction, JsValue>>,
    seen: Arc<AtomicU32>,
    advancing: Arc<AtomicBool>,
    limit: Option<u32>,
    offset: u32,
) -> Closure<dyn FnMut(Event)> {
    Closure::wrap(Box::new(move |event| {
        sender
            .send(cursor_closure_inner(
                event, &seen, &advancing, limit, offset,
            ))
            .map_err(|_| Error::AsyncChannelError)
            .unwrap_throw();
    }) as Box<dyn FnMut(Event)>)
}

fn cursor_closure_inner(
    event: Event,
    seen: &AtomicU32,
    advancing: &AtomicBool,
    limit: Option<u32>,
    offset: u32,
) -> Result<CursorAction, JsValue> {
    let target = event.target().ok_or(Error::EventTargetNotFound)?;
    let request: &IdbRequest = AsRef::<JsValue>::as_ref(&target).unchecked_ref();

    let result = request.result()?;
    if result.is_falsy() {
        return Ok(CursorAction::Break);
    }

    let cursor = IdbCursorWithValue::from(result);

    if advancing.load(Ordering::Relaxed) {
        cursor.advance(offset)?;
        advancing.store(false, Ordering::Relaxed);

        Ok(CursorAction::Continue)
    } else {
        let key = cursor.key()?;
        let value = cursor.value()?;

        seen.fetch_add(1, Ordering::Relaxed);

        match limit {
            None => {
                cursor.continue_()?;
                Ok(CursorAction::ContinueWithValue(key, value))
            }
            Some(limit) => {
                let current_seen = seen.load(Ordering::Relaxed);

                match current_seen.cmp(&limit) {
                    std::cmp::Ordering::Less => {
                        cursor.continue_()?;
                        Ok(CursorAction::ContinueWithValue(key, value))
                    }
                    std::cmp::Ordering::Equal => Ok(CursorAction::BreakWithValue(key, value)),
                    std::cmp::Ordering::Greater => Ok(CursorAction::Break),
                }
            }
        }
    }
}

fn get_callback(closure: &Closure<dyn FnMut(Event)>) -> &Function {
    closure.as_ref().unchecked_ref()
}
