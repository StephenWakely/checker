FROM debian:latest
COPY ./target/release/checker /usr/local/bin/checker
EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/checker"]
