FROM scratch
COPY target/x86_64-unknown-linux-musl/release/morph-server /morph-server
WORKDIR /app/
COPY ./static /app/static/
EXPOSE 80
CMD ["/morph-server"]
