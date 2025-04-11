# Buffered Mutexes Implementing the Reducer Parallel–Programming Pattern

[![downloads](https://img.shields.io/crates/d/buf-mutex)](https://crates.io/crates/buf-mutex)
[![dependents](https://img.shields.io/librariesio/dependents/cargo/buf-mutex)](https://crates.io/crates/buf-mutex/reverse_dependencies)
![license](https://img.shields.io/crates/l/buf-mutex)
[![](https://tokei.rs/b1/github/vigna/buf-mutex-rs?type=Rust)](https://github.com/vigna/buf-mutex-rs)
[![Latest version](https://img.shields.io/crates/v/buf-mutex.svg)](https://crates.io/crates/buf-mutex)
[![Documentation](https://docs.rs/buf-mutex/badge.svg)](https://docs.rs/buf-mutex)

Sometimes, multiple threads need to update a shared value so frequently that the
overhead of an [atomic] or a [`Mutex`] becomes a bottleneck. If, however, only
the cumulative result of the update is important, and not any of the
intermediate values, a [`BufMutex`] makes it possible to pass around multiple
cloneable [`SharedBufMutex`]s containing a local value that can be updated
without synchronization, and that will be combined into the global value when
the [`SharedBufMutex`]s are dropped. This pattern works particular well with
methods such as like [Rayon]'s [`for_each_with`], [`map_with`], and so on.

Structures of this type are called *reducers* in [OpenMP] and *reducer
hyperobjects* in [Cilk]; they fuel the inner workings of [Rayon]. However,
[Rayon] does not expose them—you access them through a functional (or *pull*)
interface (e.g., via the [`fold`] method). If you need an imperative (or *push*)
interface (e.g., direct mutation methods) you can use a [`BufMutex`].

## Acknowledgments

This software has been partially supported by project SERICS (PE00000014) under
the NRRP MUR program funded by the EU - NGEU. Views and opinions expressed are
however those of the authors only and do not necessarily reflect those of the
European Union or the Italian MUR. Neither the European Union nor the Italian
MUR can be held responsible for them.

[`for_each_with`]: <https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.for_each_with>
[`map_with`]: <https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.map_with>
[`Mutex`]: <https://doc.rust-lang.org/std/sync/struct.Mutex.html>
[`BufMutex`]: <https://docs.rs/buf-mutex/latest/buf_mutex/struct.BufMutex.html>
[`SharedBufMutex`]: <https://docs.rs/buf-mutex/latest/buf_mutex/struct.SharedBufMutex.html>
[atomic]: <https://doc.rust-lang.org/std/sync/atomic/>
[Rayon]: <https://docs.rs/rayon/latest/rayon/>
[Cilk]: <https://www.opencilk.org/>
[`fold`]: <https://docs.rs/rayon/1.10.0/rayon/iter/trait.ParallelIterator.html#method.fold>
[OpenMp]: <https://www.openmp.org/>
