FROM nginx:alpine as base

# Install dependencies for documentation processing
RUN apk add --no-cache \
    nodejs \
    npm \
    python3 \
    py3-pip \
    curl \
    jq \
    git

# Install documentation processing tools
RUN npm install -g lunr uglify-js clean-css-cli

# Create directories
RUN mkdir -p /opt/templates /opt/static /opt/scripts

# Copy custom templates and assets
COPY templates/ /opt/templates/
COPY static/ /opt/static/
COPY scripts/ /opt/scripts/
COPY nginx.conf /etc/nginx/nginx.conf

# Set up Python environment for documentation processing
RUN pip3 install --no-cache-dir \
    jinja2 \
    markdown \
    pygments \
    beautifulsoup4 \
    pyyaml

# Custom entrypoint script
COPY entrypoint.sh /opt/entrypoint.sh
RUN chmod +x /opt/entrypoint.sh

# Create nginx user directories
RUN mkdir -p /var/cache/nginx /var/log/nginx /var/run/nginx \
    && chown -R nginx:nginx /var/cache/nginx /var/log/nginx /var/run/nginx

EXPOSE 80

ENTRYPOINT ["/opt/entrypoint.sh"]
CMD ["nginx", "-g", "daemon off;"] 