# Build Stage for Backend
FROM rust:1.85-slim-bookworm AS backend-builder
WORKDIR /app
COPY . .
RUN cd backend && cargo build --release

# Build Stage for Frontend
FROM rust:1.85-slim-bookworm AS frontend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y binaryen
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown
COPY . .
# Use the Trunk.toml in root which points to frontend/index.html
RUN trunk build --release

# Final Stage
FROM debian:bookworm-slim
WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /app/backend/target/release/backend /app/leptos-studio-backend

# Copy frontend assets
COPY --from=frontend-builder /app/dist /app/dist

# Environment variables
# Absolute paths so the server never depends on its working directory.
ENV STATIC_DIR=/app/dist
ENV DATA_FILE=/app/data/projects.json
ENV TEMPLATES_FILE=/app/data/templates.json
ENV GIT_DATA_FILE=/app/data/git_data.json
ENV ANALYTICS_DATA_FILE=/app/data/analytics.json
ENV LEPTOS_API_URL=http://localhost:3000
RUN mkdir -p /app/data
VOLUME ["/app/data"]
# Backend listens on 3000
EXPOSE 3000

CMD ["/app/leptos-studio-backend"]
