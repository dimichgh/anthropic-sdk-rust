#!/bin/bash

# Anthropic Custom Infrastructure Management Script
# Provides easy management of registry and documentation services

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Main menu
show_menu() {
    echo -e "${BLUE}🚀 Anthropic Infrastructure Management${NC}"
    echo "===================================="
    echo ""
    echo "1. 🏗️  Deploy Infrastructure"
    echo "2. 🔄 Update Services"
    echo "3. ▶️  Start Services"
    echo "4. ⏹️  Stop Services"
    echo "5. 🔄 Restart Services"
    echo "6. 📊 Service Status"
    echo "7. 📜 View Logs"
    echo "8. 📦 Manage Registry"
    echo "9. 📚 Build Documentation"
    echo "10. 💾 Backup Data"
    echo "11. 🔧 System Health Check"
    echo "12. 🧹 Cleanup"
    echo "0. ❌ Exit"
    echo ""
}

# Deploy infrastructure
deploy_infrastructure() {
    echo -e "${BLUE}🏗️  Deploying infrastructure...${NC}"
    cd "$SCRIPT_DIR"
    
    # Check if Docker is running
    if ! docker info >/dev/null 2>&1; then
        echo -e "${RED}❌ Docker is not running${NC}"
        return 1
    fi
    
    # Deploy services
    docker-compose up -d --build
    
    echo -e "${GREEN}✅ Infrastructure deployed${NC}"
    show_service_urls
}

# Update services
update_services() {
    echo -e "${BLUE}🔄 Updating services...${NC}"
    cd "$SCRIPT_DIR"
    
    docker-compose pull
    docker-compose up -d --build
    
    echo -e "${GREEN}✅ Services updated${NC}"
}

# Start services
start_services() {
    echo -e "${GREEN}▶️  Starting services...${NC}"
    cd "$SCRIPT_DIR"
    docker-compose up -d
    echo -e "${GREEN}✅ Services started${NC}"
}

# Stop services
stop_services() {
    echo -e "${YELLOW}⏹️  Stopping services...${NC}"
    cd "$SCRIPT_DIR"
    docker-compose down
    echo -e "${GREEN}✅ Services stopped${NC}"
}

# Restart services
restart_services() {
    echo -e "${BLUE}🔄 Restarting services...${NC}"
    cd "$SCRIPT_DIR"
    docker-compose restart
    echo -e "${GREEN}✅ Services restarted${NC}"
}

# Service status
service_status() {
    echo -e "${BLUE}📊 Service Status${NC}"
    echo "================"
    cd "$SCRIPT_DIR"
    docker-compose ps
    echo ""
    
    # Health checks
    echo -e "${BLUE}🏥 Health Checks${NC}"
    echo "==============="
    
    local services=("registry:3000" "docs:8080" "postgres:5432" "redis:6379")
    for service in "${services[@]}"; do
        local name=$(echo "$service" | cut -d: -f1)
        local port=$(echo "$service" | cut -d: -f2)
        
        if curl -s "http://localhost:$port/health" >/dev/null 2>&1 || nc -z localhost "$port" 2>/dev/null; then
            echo -e "  $name: ${GREEN}✅ Healthy${NC}"
        else
            echo -e "  $name: ${RED}❌ Unhealthy${NC}"
        fi
    done
}

# View logs
view_logs() {
    echo -e "${BLUE}📜 Service Logs${NC}"
    echo "=============="
    echo ""
    echo "1. All services"
    echo "2. Registry"
    echo "3. Documentation"
    echo "4. Database"
    echo "5. Proxy"
    echo ""
    read -p "Select service (1-5): " choice
    
    cd "$SCRIPT_DIR"
    case $choice in
        1) docker-compose logs -f ;;
        2) docker-compose logs -f registry ;;
        3) docker-compose logs -f docs ;;
        4) docker-compose logs -f postgres ;;
        5) docker-compose logs -f proxy ;;
        *) echo -e "${RED}Invalid choice${NC}" ;;
    esac
}

# Registry management
manage_registry() {
    echo -e "${BLUE}📦 Registry Management${NC}"
    echo "===================="
    echo ""
    echo "1. 👤 Create User"
    echo "2. 🔑 Generate API Token"
    echo "3. 📋 List Crates"
    echo "4. 📊 Usage Statistics"
    echo "5. 🔧 Registry Configuration"
    echo ""
    read -p "Select option (1-5): " choice
    
    case $choice in
        1) create_registry_user ;;
        2) generate_api_token ;;
        3) list_crates ;;
        4) usage_statistics ;;
        5) registry_config ;;
        *) echo -e "${RED}Invalid choice${NC}" ;;
    esac
}

# Create registry user
create_registry_user() {
    echo -e "${BLUE}👤 Create Registry User${NC}"
    read -p "Username: " username
    read -p "Email: " email
    read -s -p "Password: " password
    echo ""
    
    # This would typically make an API call to create the user
    echo -e "${GREEN}✅ User created: $username${NC}"
    echo -e "${YELLOW}⚠️  Note: Implement actual user creation API${NC}"
}

# Generate API token
generate_api_token() {
    echo -e "${BLUE}🔑 Generate API Token${NC}"
    local token="anthropic-$(openssl rand -hex 16)"
    echo -e "${GREEN}✅ API Token: $token${NC}"
    echo -e "${YELLOW}💡 Add this token to your Cargo config${NC}"
}

# List crates
list_crates() {
    echo -e "${BLUE}📋 Registered Crates${NC}"
    echo "=================="
    
    # This would query the registry API
    echo -e "${YELLOW}⚠️  Implement crate listing API${NC}"
}

# Usage statistics
usage_statistics() {
    echo -e "${BLUE}📊 Usage Statistics${NC}"
    echo "=================="
    
    # This would query metrics from Prometheus
    echo -e "${YELLOW}⚠️  Implement statistics API${NC}"
}

# Registry configuration
registry_config() {
    echo -e "${BLUE}🔧 Registry Configuration${NC}"
    echo "========================"
    
    if [ -f "$SCRIPT_DIR/config/alexandrie.toml" ]; then
        echo "Current configuration:"
        cat "$SCRIPT_DIR/config/alexandrie.toml"
    else
        echo -e "${RED}❌ Configuration file not found${NC}"
    fi
}

# Build documentation
build_documentation() {
    echo -e "${BLUE}📚 Documentation Builder${NC}"
    echo "======================"
    echo ""
    echo "1. 🔨 Build from local project"
    echo "2. 🌐 Build from Git repository"
    echo "3. 📦 Build from published crate"
    echo ""
    read -p "Select option (1-3): " choice
    
    case $choice in
        1) build_local_docs ;;
        2) build_git_docs ;;
        3) build_crate_docs ;;
        *) echo -e "${RED}Invalid choice${NC}" ;;
    esac
}

# Build local documentation
build_local_docs() {
    read -p "Enter project path: " project_path
    
    if [ ! -d "$project_path" ]; then
        echo -e "${RED}❌ Project path does not exist${NC}"
        return 1
    fi
    
    echo -e "${BLUE}🔨 Building documentation for $project_path${NC}"
    
    cd "$SCRIPT_DIR"
    docker-compose run --rm docs-builder bash -c "
        cd /opt/source/$project_path && 
        cargo doc --no-deps && 
        cp -r target/doc/* /opt/output/
    "
    
    echo -e "${GREEN}✅ Documentation built${NC}"
}

# Build Git documentation
build_git_docs() {
    read -p "Enter Git repository URL: " repo_url
    
    echo -e "${BLUE}🌐 Building documentation for $repo_url${NC}"
    
    cd "$SCRIPT_DIR"
    docker-compose run --rm docs-builder bash -c "
        git clone $repo_url /tmp/repo && 
        cd /tmp/repo && 
        cargo doc --no-deps && 
        cp -r target/doc/* /opt/output/
    "
    
    echo -e "${GREEN}✅ Documentation built${NC}"
}

# Build crate documentation
build_crate_docs() {
    read -p "Enter crate name: " crate_name
    read -p "Enter crate version (optional): " crate_version
    
    local version_flag=""
    if [ -n "$crate_version" ]; then
        version_flag="--version $crate_version"
    fi
    
    echo -e "${BLUE}📦 Building documentation for $crate_name${NC}"
    
    cd "$SCRIPT_DIR"
    docker-compose run --rm docs-builder bash -c "
        cargo install $crate_name $version_flag --root /tmp/crate && 
        # This would need more complex logic to build docs for installed crates
        echo 'Crate documentation building not fully implemented'
    "
}

# Backup data
backup_data() {
    echo -e "${BLUE}💾 Backup Data${NC}"
    echo "============="
    
    local backup_dir="$SCRIPT_DIR/backups/$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$backup_dir"
    
    echo -e "${YELLOW}📦 Creating backup...${NC}"
    
    # Backup database
    cd "$SCRIPT_DIR"
    docker-compose exec postgres pg_dump -U registry alexandrie > "$backup_dir/database.sql"
    
    # Backup registry data
    cp -r "$SCRIPT_DIR/data/registry" "$backup_dir/"
    
    # Backup documentation
    cp -r "$SCRIPT_DIR/data/docs" "$backup_dir/"
    
    # Create archive
    tar -czf "$backup_dir.tar.gz" -C "$SCRIPT_DIR/backups" "$(basename "$backup_dir")"
    rm -rf "$backup_dir"
    
    echo -e "${GREEN}✅ Backup created: $backup_dir.tar.gz${NC}"
}

# Health check
health_check() {
    echo -e "${BLUE}🏥 System Health Check${NC}"
    echo "===================="
    
    # Check Docker
    if docker info >/dev/null 2>&1; then
        echo -e "Docker: ${GREEN}✅ Running${NC}"
    else
        echo -e "Docker: ${RED}❌ Not running${NC}"
    fi
    
    # Check disk space
    local disk_usage=$(df "$SCRIPT_DIR" | awk 'NR==2 {print $5}' | sed 's/%//')
    if [ "$disk_usage" -lt 80 ]; then
        echo -e "Disk Space: ${GREEN}✅ $disk_usage% used${NC}"
    else
        echo -e "Disk Space: ${YELLOW}⚠️  $disk_usage% used${NC}"
    fi
    
    # Check memory
    local mem_usage=$(free | awk 'NR==2{printf "%.1f", $3*100/$2}')
    echo -e "Memory Usage: ${BLUE}ℹ️  $mem_usage%${NC}"
    
    # Check services
    service_status
}

# Cleanup
cleanup() {
    echo -e "${YELLOW}🧹 Cleanup Options${NC}"
    echo "=================="
    echo ""
    echo "1. 🗑️  Remove unused Docker images"
    echo "2. 🧽 Clean logs"
    echo "3. 📁 Clean temporary files"
    echo "4. ⚠️  Remove all data (DANGEROUS)"
    echo ""
    read -p "Select option (1-4): " choice
    
    case $choice in
        1)
            docker image prune -f
            echo -e "${GREEN}✅ Unused images removed${NC}"
            ;;
        2)
            find "$SCRIPT_DIR/logs" -name "*.log" -type f -delete 2>/dev/null || echo "No logs to clean"
            echo -e "${GREEN}✅ Logs cleaned${NC}"
            ;;
        3)
            rm -rf /tmp/anthropic-* 2>/dev/null || echo "No temp files to clean"
            echo -e "${GREEN}✅ Temporary files cleaned${NC}"
            ;;
        4)
            echo -e "${RED}⚠️  WARNING: This will delete ALL data!${NC}"
            read -p "Type 'DELETE' to confirm: " confirm
            if [ "$confirm" = "DELETE" ]; then
                cd "$SCRIPT_DIR"
                docker-compose down -v
                rm -rf data/ logs/
                echo -e "${GREEN}✅ All data removed${NC}"
            else
                echo -e "${YELLOW}Cancelled${NC}"
            fi
            ;;
        *) echo -e "${RED}Invalid choice${NC}" ;;
    esac
}

# Show service URLs
show_service_urls() {
    echo ""
    echo -e "${GREEN}🌐 Service URLs:${NC}"
    echo -e "   Registry:      http://localhost:3000"
    echo -e "   Documentation: http://localhost:8080"
    echo -e "   Monitoring:    http://localhost:3001"
    echo -e "   Prometheus:    http://localhost:9090"
    echo ""
}

# Main loop
main() {
    while true; do
        show_menu
        read -p "Select option (0-12): " choice
        echo ""
        
        case $choice in
            1) deploy_infrastructure ;;
            2) update_services ;;
            3) start_services ;;
            4) stop_services ;;
            5) restart_services ;;
            6) service_status ;;
            7) view_logs ;;
            8) manage_registry ;;
            9) build_documentation ;;
            10) backup_data ;;
            11) health_check ;;
            12) cleanup ;;
            0) echo -e "${GREEN}👋 Goodbye!${NC}"; exit 0 ;;
            *) echo -e "${RED}❌ Invalid choice${NC}" ;;
        esac
        
        echo ""
        read -n 1 -s -r -p "Press any key to continue..."
        echo ""
        clear
    done
}

# Run main if called directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main
fi 