# 🚀 Anthropic Custom Crates Registry & Documentation Infrastructure

A complete, production-ready infrastructure for hosting private Rust crates and documentation, providing full control over your development ecosystem.

## 🎯 Overview

This infrastructure provides:
- **Private Crates Registry**: Self-hosted alternative to crates.io using Alexandrie
- **Documentation Hosting**: Custom documentation server with search and branding
- **Monitoring & Analytics**: Prometheus + Grafana for metrics and insights
- **Backup & Recovery**: Automated backup system with disaster recovery
- **Security**: Authentication, rate limiting, and secure access controls

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Nginx Proxy   │    │  Alexandrie     │    │  Documentation  │
│   (Port 80/443) │───▶│  Registry       │    │  Server         │
│                 │    │  (Port 3000)    │    │  (Port 8080)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   PostgreSQL    │    │     Redis       │    │   Prometheus    │
│   Database      │    │     Cache       │    │   Monitoring    │
│   (Port 5432)   │    │   (Port 6379)   │    │   (Port 9090)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📦 Prerequisites

- **Docker** (20.10+)
- **Docker Compose** (2.0+)
- **OpenSSL** (for certificate generation)
- **curl** (for health checks)
- **Git** (for repository management)

## 🚀 Quick Start

### 1. Clone and Setup

```bash
# Navigate to infrastructure directory
cd infrastructure/

# Make scripts executable
chmod +x *.sh

# Run interactive management script
./manage.sh
```

### 2. Initial Deployment

Choose option `1` from the management menu to deploy the infrastructure:

```bash
./manage.sh
# Select: 1. 🏗️ Deploy Infrastructure
```

This will:
- Create necessary directories
- Generate SSL certificates
- Build and start all services
- Initialize the database
- Create default admin user

### 3. Access Services

Once deployed, access your services at:

- **🏠 Main Portal**: https://localhost/
- **📦 Registry**: https://localhost/registry/
- **📚 Documentation**: https://localhost/docs/
- **📊 Monitoring**: https://localhost/monitoring/
- **🔍 Metrics**: http://localhost:9090/

### 4. Configure Cargo

Add your private registry to Cargo configuration:

```bash
# Create/edit ~/.cargo/config.toml
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << 'EOF'
[registries]
anthropic = { index = "https://localhost/api/v1/crates/git-index" }

[registry]
default = "anthropic"
EOF
```

### 5. Login to Registry

```bash
# Generate API token from management interface
./manage.sh
# Select: 8. 📦 Manage Registry -> 2. 🔑 Generate API Token

# Login with the generated token
cargo login --registry anthropic <your-api-token>
```

## 📖 Usage Guide

### Publishing Crates

```bash
# In your Rust project directory
cargo publish --registry anthropic
```

### Building Documentation

```bash
# Using the management script
./manage.sh
# Select: 9. 📚 Build Documentation

# Or directly build your project documentation
./manage.sh
# Select: 9. 📚 Build Documentation -> 1. 🔨 Build from local project
# Enter path: /path/to/your/rust/project
```

### User Management

```bash
# Access registry management
./manage.sh
# Select: 8. 📦 Manage Registry

# Create new user
# Select: 1. 👤 Create User

# Generate API tokens
# Select: 2. 🔑 Generate API Token
```

## 🔧 Configuration

### Registry Configuration

Edit `config/alexandrie.toml` to customize:

```toml
[general]
name = "Your Organization Crates Registry"
description = "Private Rust crates for your organization"
base_url = "https://your-domain.com"

[frontend]
title = "Your Organization Crates"
custom_css = "/opt/templates/custom.css"
logo_url = "/static/your-logo.png"

[security]
cors_allow_origins = ["https://your-domain.com"]
api_key_prefix = "your-org-"
```

### Documentation Customization

Customize documentation appearance:

```bash
# Edit documentation templates
vim docs-server/templates/index.html
vim docs-server/static/custom-docs.css
vim docs-server/static/custom-docs.js
```

### SSL/TLS Configuration

For production, replace self-signed certificates:

```bash
# Copy your certificates
cp your-domain.crt infrastructure/ssl/server.crt
cp your-domain.key infrastructure/ssl/server.key

# Restart services
./manage.sh
# Select: 5. 🔄 Restart Services
```

## 📊 Monitoring & Analytics

### Prometheus Metrics

Access metrics at: http://localhost:9090/

Available metrics:
- Registry API requests
- Crate download counts
- User activity
- System resources
- Error rates

### Grafana Dashboards

Access dashboards at: http://localhost:3001/
- Default login: `admin/admin`

Pre-configured dashboards:
- Registry Overview
- API Performance
- User Activity
- System Health

### Custom Alerts

Configure alerts in `monitoring/prometheus.yml`:

```yaml
rule_files:
  - "alert_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

## 💾 Backup & Recovery

### Automated Backups

Backups run automatically at 2 AM daily. Configure in `config/alexandrie.toml`:

```toml
[backup]
enabled = true
schedule = "0 2 * * *"  # Daily at 2 AM
s3_bucket = "your-backup-bucket"
```

### Manual Backup

```bash
./manage.sh
# Select: 10. 💾 Backup Data
```

This creates:
- Database dump
- Registry files
- Documentation
- Configuration

### Restore from Backup

```bash
# Stop services
./manage.sh
# Select: 4. ⏹️ Stop Services

# Restore database
cat backup/database.sql | docker-compose exec -T postgres psql -U registry alexandrie

# Restore files
cp -r backup/registry/* data/registry/
cp -r backup/docs/* data/docs/

# Start services
./manage.sh
# Select: 3. ▶️ Start Services
```

## 🔐 Security Considerations

### Authentication

- **Registry**: Token-based authentication
- **Monitoring**: Basic HTTP authentication
- **Database**: Internal network only

### Network Security

- **Firewall**: Configure firewall rules
- **SSL/TLS**: Use valid certificates for production
- **Rate Limiting**: Configured per service

### Best Practices

1. **Change Default Passwords**: Update all default passwords
2. **Regular Updates**: Keep Docker images updated
3. **Monitor Logs**: Review access and error logs
4. **Backup Verification**: Test backup restoration regularly
5. **Security Scanning**: Scan for vulnerabilities

## 🚨 Troubleshooting

### Common Issues

#### Services Won't Start

```bash
# Check Docker status
docker info

# Check logs
./manage.sh
# Select: 7. 📜 View Logs

# Check system health
./manage.sh
# Select: 11. 🔧 System Health Check
```

#### Database Connection Errors

```bash
# Check PostgreSQL logs
./manage.sh
# Select: 7. 📜 View Logs -> 4. Database

# Restart database
docker-compose restart postgres
```

#### Registry API Errors

```bash
# Check registry configuration
./manage.sh
# Select: 8. 📦 Manage Registry -> 5. 🔧 Registry Configuration

# Verify API token
cargo login --registry anthropic <new-token>
```

#### Documentation Build Failures

```bash
# Check documentation builder logs
docker-compose logs docs-builder

# Verify project structure
ls -la /path/to/your/project/Cargo.toml
```

### Performance Tuning

#### Database Optimization

```sql
-- Connect to PostgreSQL
\c alexandrie

-- Check query performance
EXPLAIN ANALYZE SELECT * FROM crates WHERE name = 'example';

-- Add indexes for frequently queried fields
CREATE INDEX idx_crates_name_trgm ON crates USING gin (name gin_trgm_ops);
```

#### Resource Allocation

```yaml
# docker-compose.yml
services:
  postgres:
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '1.0'
```

## 🔄 Updates & Maintenance

### Update Services

```bash
./manage.sh
# Select: 2. 🔄 Update Services
```

### Maintenance Tasks

```bash
# Clean up unused resources
./manage.sh
# Select: 12. 🧹 Cleanup

# Health check
./manage.sh
# Select: 11. 🔧 System Health Check
```

### Version Management

Keep track of component versions:

```bash
# Check running versions
docker-compose ps
docker images | grep anthropic
```

## 🤝 Contributing

### Development Setup

```bash
# Clone repository
git clone <your-repo-url>
cd anthropic-sdk-rust/infrastructure

# Development mode
docker-compose -f docker-compose.yml -f docker-compose.dev.yml up
```

### Adding Features

1. **Registry Features**: Modify `config/alexandrie.toml`
2. **Documentation**: Update `docs-server/` templates
3. **Monitoring**: Add metrics to `monitoring/prometheus.yml`

## 📞 Support

### Getting Help

1. **Documentation**: Check this README
2. **Logs**: Use `./manage.sh` to view logs
3. **Health Check**: Run system health check
4. **Issues**: Create GitHub issues for bugs

### Community

- **GitHub Discussions**: For questions and ideas
- **Issues**: For bug reports and feature requests
- **Wiki**: For additional documentation

## 📄 License

This infrastructure is licensed under the MIT License. See LICENSE file for details.

---

## 🎯 Next Steps

1. **Deploy Infrastructure**: Run the deployment script
2. **Configure Registry**: Set up your organization's registry
3. **Publish First Crate**: Test with a sample crate
4. **Build Documentation**: Generate docs for your projects
5. **Set Up Monitoring**: Configure alerts and dashboards
6. **Plan Backups**: Test backup and restore procedures

**Ready to build your private Rust ecosystem? Start with `./manage.sh`!** 🚀 