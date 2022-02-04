FROM node:17.4.0 AS client-base
COPY ./ ./app
WORKDIR /app
RUN cd client && npm ci && npm run build-prod

FROM rust:1.58.1 as server-base
COPY --from=client-base ./app ./app
WORKDIR /app
RUN cd server && cargo build --release && mkdir -p /build-out && cp target/release/red-dragon-inn-server /build-out/

# TODO - Revert base image back to debian:10-slim. I changed it because the Meilisearch client requires libssl.
FROM ubuntu:21.10
COPY --from=server-base /build-out/red-dragon-inn-server /
ENV ROCKET_PORT=80
ENV ROCKET_ADDRESS="0.0.0.0"
EXPOSE 80
CMD /red-dragon-inn-server