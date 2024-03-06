use async_trait::async_trait;
use pingora::prelude::*;
use std::sync::Arc;

// A structure to encapsulate a thread-safe, reference-counted load balancer
// using a round-robin stategy for distributing tasks or requests
pub struct LB(Arc<LoadBalancer<RoundRobin>>);

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> () {
        ()
    }

    // Asynchronously determine the upstream peer to which requests should
    // be forwarded.
    // Uses a round-robin selection strategy
    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self
            .0
            .select(b"", 256) // hash doesn't matter for round robin
            .unwrap();

        // Print the selected upstream peer for debug
        println!("upstream peer is: {:?}", upstream);

        // Create a new HTTP peer with SNI
        // Set SNI to one.one.one.one
        let peer = Box::new(HttpPeer::new(upstream, true, "one.one.one.one".to_string()));

        // Return the newly created HTTP peer
        Ok(peer)
    }

    // Asynchronously filter the upstream HTTP request before it sent
    // This function modies the request header, e.g., setting "Host" header.
    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Insert or update the "Host" header in the upstream request to
        // "one.one.one.one"
        upstream_request
            .insert_header("Host", "one.one.one.one")
            .unwrap();

        // Return after request modified
        Ok(())
    }
}

fn main() {
    println!("Hello, world!");

    // Create a server instance with default config
    // Unwrap is used here to handle potential errors by panicking
    let mut my_server = Server::new(None).unwrap();

    // Initialize server
    my_server.bootstrap();

    // Define upstream servers. These servers receive forwarded requests
    // by the load balancer
    let mut upstreams =
        LoadBalancer::try_from_iter(["1.1.1.1:443", "1.0.0.1:443", "127.0.0.1:343"]).unwrap();

    let hc = TcpHealthCheck::new(); // Create a new TCP health check instance
    upstreams.set_health_check(hc); // Assign health check to load balancer
                                    // Set frequency to every second
    upstreams.health_check_frequency = Some(std::time::Duration::from_secs(1));

    // Launch a background service to continuosly perform health check on
    // the upstream servers. This service runs independently of the main
    // server logic.
    let background = background_service("health check", upstreams);

    // Retrieve the task (load balancer with health checks) from the
    // background service to be used in the HTTP proxy service setup
    let upstreams = background.task();

    // Setup the HTTP proxy service with the server configuration and the load balanced upstreams.
    // Note: The load balancer is directly passed without needing to be wrapped in an `Arc`,
    // indicating ownership is transferred to the proxy service.
    let mut lb = http_proxy_service(&my_server.configuration, LB(upstreams));
    lb.add_tcp("0.0.0.0:6188"); // Add TCP listener

    // Add the configured load balancing HTTP proxy service to the server.
    my_server.add_service(lb);

    // Start server
    my_server.run_forever();
}
