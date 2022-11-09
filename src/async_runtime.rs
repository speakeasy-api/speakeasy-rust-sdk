use futures::Future;
#[cfg(feature = "tokio02")]
#[doc(hidden)]
pub(crate) fn spawn_task<T>(task: T) -> tokio02::task::JoinHandle<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    tokio02::task::spawn(task)
}

#[cfg(feature = "tokio")]
#[doc(hidden)]
pub(crate) fn spawn_task<T>(task: T) -> tokio::task::JoinHandle<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    tokio::task::spawn(task)
}
