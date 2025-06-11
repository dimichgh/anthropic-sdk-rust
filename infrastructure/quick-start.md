# 🚀 Quick Start Guide: Custom Crates Registry & Documentation

## ⚡ 5-Minute Setup

### Step 1: Prerequisites Check

```bash
# Verify Docker is installed and running
docker --version
docker-compose --version

# Check if Docker daemon is running
docker info
```

### Step 2: Deploy Infrastructure

```bash
# Navigate to infrastructure directory
cd infrastructure/

# Make scripts executable
chmod +x *.sh

# Start interactive management
./manage.sh
```

### Step 3: Initial Deployment

From the management menu, select:
```
1. 🏗️ Deploy Infrastructure
```

This automatically:
- ✅ Creates directory structure
- ✅ Generates SSL certificates
- ✅ Builds Docker images
- ✅ Starts all services
- ✅ Initializes database
- ✅ Creates admin user

### Step 4: Verify Services

Your services will be available at:

| Service | URL | Purpose |
|---------|-----|---------|
| 🏠 Main Portal | https://localhost/ | Landing page |
| 📦 Registry | https://localhost/registry/ | Crate registry web UI |
| 📚 Documentation | https://localhost/docs/ | Documentation hosting |
| 📊 Monitoring | https://localhost/monitoring/ | Grafana dashboards |
| 🔍 Metrics | http://localhost:9090/ | Prometheus metrics |

### Step 5: Configure Cargo

```bash
# Add private registry to Cargo config
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << 'EOF'
[registries]
anthropic = { index = "https://localhost/api/v1/crates/git-index" }

[registry]
default = "anthropic"
EOF
```

### Step 6: Get API Token

```bash
# From management menu
./manage.sh
# Select: 8. 📦 Manage Registry
# Select: 2. 🔑 Generate API Token

# Login to registry
cargo login --registry anthropic <generated-token>
```

## 🎯 Test Your Setup

### Publish Your First Crate

```bash
# Navigate to your Rust project
cd /path/to/your/rust/project

# Publish to your private registry
cargo publish --registry anthropic
```

### Build Documentation

```bash
# From management menu
./manage.sh
# Select: 9. 📚 Build Documentation
# Select: 1. 🔨 Build from local project
# Enter path: /path/to/your/rust/project
```

### Verify Everything Works

```bash
# Check service status
./manage.sh
# Select: 6. 📊 Service Status

# View logs if needed
./manage.sh
# Select: 7. 📜 View Logs
```

## 🔧 Common Commands

```bash
# Start services
./manage.sh → 3. ▶️ Start Services

# Stop services
./manage.sh → 4. ⏹️ Stop Services

# Update services
./manage.sh → 2. 🔄 Update Services

# View logs
./manage.sh → 7. 📜 View Logs

# Backup data
./manage.sh → 10. 💾 Backup Data

# Health check
./manage.sh → 11. 🔧 System Health Check
```

## 🚨 Troubleshooting

### Services Won't Start

```bash
# Check Docker status
docker info

# Check logs
./manage.sh → 7. 📜 View Logs → 1. All services

# Restart everything
./manage.sh → 5. 🔄 Restart Services
```

### Can't Access Web Interface

```bash
# Check if services are running
docker-compose ps

# Verify ports are not blocked
curl -k https://localhost/
curl http://localhost:3000/health
```

### Cargo Login Issues

```bash
# Generate new token
./manage.sh → 8. 📦 Manage Registry → 2. 🔑 Generate API Token

# Try login again
cargo login --registry anthropic <new-token>

# Check registry configuration
./manage.sh → 8. 📦 Manage Registry → 5. 🔧 Registry Configuration
```

## 📈 Next Steps

1. **Customize Branding**: Edit `config/alexandrie.toml` and `docs-server/templates/`
2. **Set Up Monitoring**: Configure alerts in Grafana
3. **Plan Backups**: Test backup and restore procedures
4. **Security**: Replace self-signed certificates for production
5. **Scale**: Add more workers or storage as needed

## 🎉 Success!

You now have a fully functional private Rust ecosystem with:
- ✅ Private crates registry
- ✅ Custom documentation hosting
- ✅ Monitoring and analytics
- ✅ Backup and recovery
- ✅ Security and authentication

**Your team can now publish private crates and host beautiful documentation!**

---

**Need help?** Check the full [README.md](README.md) or run `./manage.sh` for the interactive management interface. 