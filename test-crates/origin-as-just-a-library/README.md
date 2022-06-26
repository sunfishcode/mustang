This crate demonstrates the use of origin as a plain library API in non-mustang
builds.

In non-mustang builds, origin lets libc perform program startup, and it
implements its thread API using the libc pthread API. Its own code for program
startup and thread creation is disabled, so the API fully interoperates with
the libc runtime.
