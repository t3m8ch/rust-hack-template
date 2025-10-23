use socketioxide::extract::SocketRef;

// Example

pub fn hello(s: &SocketRef) {
    s.on("msg", |s: SocketRef| {
        s.emit("msg-response", "Hello from server").ok();
    });
}
