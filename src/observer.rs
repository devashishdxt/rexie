use std::fmt::Debug;

use js_sys::Function;
use tokio::{
    select,
    sync::oneshot::{channel as oneshot, Receiver, Sender},
};
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::Event;

use crate::{ErrorType, Result};

pub struct ResultObserver<T>
where
    T: Debug + 'static,
{
    pub(crate) success_observer: Observer<Result<T>>,
    pub(crate) error_observer: Observer<Result<T>>,
}

impl<T> ResultObserver<T>
where
    T: Debug + 'static,
{
    pub fn new(ok: T, err: impl Fn(&Event) -> Result<T> + 'static) -> Self {
        let success_observer = Observer::new(Ok(ok));
        let error_observer = Observer::new_lazy(err);

        Self {
            success_observer,
            error_observer,
        }
    }

    pub fn get_success_callback(&self) -> &Function {
        self.success_observer.get_callback()
    }

    pub fn get_error_callback(&self) -> &Function {
        self.error_observer.get_callback()
    }

    pub async fn finish(self) -> Result<T> {
        let success_fut = self.success_observer.finish();
        let error_fut = self.error_observer.finish();

        let res = select! {
            res = success_fut => res,
            res = error_fut => res,
        }?;

        res
    }
}

pub struct Observer<T>
where
    T: Debug + 'static,
{
    pub(crate) closure: Closure<dyn FnMut(Event)>,
    pub(crate) receiver: Receiver<T>,
}

impl<T> Observer<T>
where
    T: Debug + 'static,
{
    pub fn new(value: T) -> Self {
        let (sender, receiver) = oneshot();
        build_observer(value, sender, receiver)
    }

    pub fn new_lazy(f: impl Fn(&Event) -> T + 'static) -> Self {
        let (sender, receiver) = oneshot();
        build_lazy_observer(f, sender, receiver)
    }

    pub fn get_callback(&self) -> &Function {
        self.closure.as_ref().unchecked_ref()
    }

    pub async fn finish(self) -> Result<T> {
        let result = self
            .receiver
            .await
            .map_err(|_| ErrorType::AsyncChannelError.into_error())?;

        Ok(result)
    }
}

fn build_observer<T>(value: T, sender: Sender<T>, receiver: Receiver<T>) -> Observer<T>
where
    T: Debug + 'static,
{
    build_lazy_observer(move |_| value, sender, receiver)
}

fn build_lazy_observer<T>(
    f: impl FnOnce(&Event) -> T + 'static,
    sender: Sender<T>,
    receiver: Receiver<T>,
) -> Observer<T>
where
    T: Debug + 'static,
{
    let closure = Closure::once(move |event: Event| {
        sender
            .send(f(&event))
            .map_err(|_| ErrorType::AsyncChannelError.into_error())
            .unwrap_throw();
    });

    Observer { closure, receiver }
}
