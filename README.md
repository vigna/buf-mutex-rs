# Sync cells and slices

[![downloads](https://img.shields.io/crates/d/buf-mutex)](https://crates.io/crates/buf-mutex)
[![dependents](https://img.shields.io/librariesio/dependents/cargo/buf-mutex)](https://crates.io/crates/buf-mutex/reverse_dependencies)
![license](https://img.shields.io/crates/l/buf-mutex)
[![](https://tokei.rs/b1/github/vigna/buf-mutex-rs?type=Rust)](https://github.com/vigna/buf-mutex-rs)
[![Latest version](https://img.shields.io/crates/v/buf-mutex.svg)](https://crates.io/crates/buf-mutex)
[![Documentation](https://docs.rs/buf-mutex/badge.svg)](https://docs.rs/buf-mutex)

Sometimes, multiple threads need to update a value so frequently that
the overhead of atomic access becomes a bottleneck. If, however, only
the cumulative results of the update is important, and not any of
the intermediate values, a [`BufMutex`] will wrap the global value
in a [`Mutex`] and provide a public local value that can be modified
at will. Cloning a [`BufMutex`] provides a new local value and a
references to the original global value; when the [`BufMutex`]
is dropped, the local value is combined with the global value
using a provided reduction function. Thus, clones can be provided
to multiple threads.

## Acknowledgments

This software has been partially supported by project SERICS (PE00000014) under
the NRRP MUR program funded by the EU - NGEU. Views and opinions expressed are
however those of the authors only and do not necessarily reflect those of the
European Union or the Italian MUR. Neither the European Union nor the Italian
MUR can be held responsible for them.
