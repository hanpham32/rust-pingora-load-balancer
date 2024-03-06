# rust-pingora-load-balancer

A simple HTTP proxy and load balancer in Rust using Pingora framework. Pingora sets up the server, forwarding HTTP requests to a set of upstream servers based on Round-robin selection method.

### Get started

Before you begin, ensure you have the following installed:

Rust and Cargo (latest stable version recommended)
Git (for cloning the repository)

### Build the project:
```
cargo build
```

### Running the server
```
cargo run
```

LB: A struct encapsulating a load balancer with round-robin logic.

ProxyHttp for LB: Trait implementation providing HTTP proxy functionality.
