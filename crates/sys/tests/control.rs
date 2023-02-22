use ambient_sys::{control::control_future, task::JoinError};

#[test]
fn control() {
    let (mut handle, future) = control_future(async move { "Hello, World" });

    use futures::FutureExt;

    assert_eq!((&mut handle).now_or_never(), None);

    futures::executor::block_on(future);

    assert_eq!((&mut handle).now_or_never(), Some(Ok("Hello, World")));
}

#[test]
fn control_abort() {
    let (mut handle, future) = control_future(async move { "Hello, World" });

    use futures::FutureExt;

    assert_eq!((&mut handle).now_or_never(), None);

    handle.abort();
    assert!(!handle.is_finished());

    futures::executor::block_on(future);

    assert!(handle.is_finished());

    assert_eq!((&mut handle).now_or_never(), Some(Err(JoinError::Aborted)));
}
