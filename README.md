# P2P Internals #4: Dissecting git

This repository is a resource for a tutorial I wrote (TODO LINK) to introduce how the git protocol works. Don't use it for a project.

The goal here is to create a minimal git server only supporting fetch/clone with some basic features (such as `side-band-64k`) on a custom transport.

## References

+ https://libgit2.org/libgit2/#HEAD (**!!!WARNING - the generated doc is incomplete and miss a lot of methods, like the one related to the smart transport!!!**)
+ https://github.com/git/git/blob/master/Documentation/technical/ is a good reference for implementing a git server
+ ===TODO: link to my work====
+ Rust: https://docs.rs/git2/0.13.17/git2/
+ https://github.com/rust-lang/git2-rs/tree/master/git2-curl (which implements git over curl)