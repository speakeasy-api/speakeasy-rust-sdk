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
