FROM rust:1.65.0 AS builder
WORKDIR /builder
COPY data_processor.zip .
RUN unzip -o data_processor.zip
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /builder/target/release/data-processor-sample .
USER 1000
CMD ["./data-processor-sample"]
