FROM rust:1.86-slim

WORKDIR /usr/src/app
COPY . .

# Build the application
RUN cargo build

# Run the application
CMD ["cargo", "run"]
