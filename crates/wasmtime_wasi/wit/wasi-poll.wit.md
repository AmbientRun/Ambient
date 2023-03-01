# WASI Poll API

## `wasi-poll
```wit
///
/// WASI Poll is a poll API intended to let users wait for I/O events on
/// multiple handles at once.
default interface wasi-poll {
```

## `pollable`
```wit
  /// A "pollable" handle.
  ///
  /// This is conceptually represents a `stream<_, _>`, or in other words,
  /// a stream that one can wait on, repeatedly, but which does not itself
  /// produce any data. It's temporary scaffolding until component-model's
  /// async features are ready.
  ///
  /// And at present, it is a `u32` instead of being an actual handle, until
  /// the wit-bindgen implementation of handles and resources is ready.
  ///
  /// `pollable` lifetimes are not automatically managed. Users must ensure
  /// that they do not outlive the resource they reference.
  // TODO(resource pollable {)
  type pollable = u32
```

## `drop-pollable`
```wit
/// Dispose of the specified `pollable`, after which it may no longer be used.
// TODO(} /* resource pollable */)
drop-pollable: func(this: pollable)
```

## `poll-oneoff`
```wit
/// Poll for completion on a set of pollables.
///
/// The "oneoff" in the name refers to the fact that this function must do a
/// linear scan through the entire list of subscriptions, which may be
/// inefficient if the number is large and the same subscriptions are used
/// many times. In the future, it may be accompanied by an API similar to
/// Linux's `epoll` which allows sets of subscriptions to be registered and
/// made efficiently reusable.
///
/// Note that the return type would ideally be `list<bool>`, but that would
/// be more difficult to polyfill given the current state of `wit-bindgen`.
/// See https://github.com/bytecodealliance/preview2-prototyping/pull/11#issuecomment-1329873061
/// for details.  For now, we use zero to mean "not ready" and non-zero to
/// mean "ready".
poll-oneoff: func(in: list<pollable>) -> list<u8>
```

```wit
}
```
