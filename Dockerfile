FROM rust:1.67

COPY ./ ./

RUN ulimit -n 100000

RUN cargo build --release

# Run the binary
CMD ["./target/release/logsaurus"]
