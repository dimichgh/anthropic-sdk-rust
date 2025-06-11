FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    wget \
    unzip \
    pkg-config \
    libssl-dev \
    python3 \
    python3-pip \
    nodejs \
    npm \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Install documentation processing tools
RUN npm install -g lunr

# Install Python dependencies for documentation processing
RUN pip3 install --no-cache-dir \
    jinja2 \
    markdown \
    pygments \
    beautifulsoup4 \
    pyyaml \
    requests

# Create working directories
RUN mkdir -p /opt/docs-builder /opt/templates /opt/output
WORKDIR /opt/docs-builder

# Copy builder scripts and templates
COPY scripts/ /opt/docs-builder/scripts/
COPY templates/ /opt/templates/
COPY config/ /opt/docs-builder/config/

# Make scripts executable
RUN chmod +x /opt/docs-builder/scripts/*.sh

# Set up Rust environment
ENV CARGO_HOME=/opt/cargo
ENV RUSTUP_HOME=/opt/rustup
ENV PATH="/opt/cargo/bin:${PATH}"

# Install additional Rust components
RUN rustup component add rustfmt clippy

# Create entrypoint
COPY entrypoint.sh /opt/docs-builder/entrypoint.sh
RUN chmod +x /opt/docs-builder/entrypoint.sh

# Set up volumes
VOLUME ["/opt/output", "/opt/source"]

# Environment variables
ENV DOCS_OUTPUT_PATH=/opt/output
ENV SOURCE_PATH=/opt/source
ENV TEMPLATE_PATH=/opt/templates

ENTRYPOINT ["/opt/docs-builder/entrypoint.sh"] 