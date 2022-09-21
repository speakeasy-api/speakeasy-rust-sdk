use futures::Future;

#[cfg(feature = "tokio02")]
#[doc(hidden)]
pub(crate) type Sender<T> = tokio02::sync::mpsc::Sender<T>;

#[cfg(feature = "tokio02")]
#[doc(hidden)]
pub(crate) type Receiver<T> = tokio02::sync::mpsc::Receiver<T>;

#[cfg(feature = "tokio02")]
#[doc(hidden)]
pub(crate) fn spawn_task<T>(task: T) -> tokio02::task::JoinHandle<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    tokio02::task::spawn(task)
}

#[cfg(feature = "tokio02")]
#[doc(hidden)]
pub fn channel<T>(buffer: usize) -> (Sender<T>, Receiver<T>) {
    tokio02::sync::mpsc::channel(buffer)
}
