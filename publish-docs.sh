#!/bin/bash

# Anthropic Rust SDK Documentation Publishing Script
# This script helps publish the generated documentation to various platforms

set -e

echo "🚀 Anthropic Rust SDK Documentation Publisher"
echo "============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if cargo doc has been run
if [ ! -d "target/doc" ]; then
    echo -e "${YELLOW}📚 Documentation not found. Generating docs...${NC}"
    cargo doc --no-deps
fi

echo -e "${BLUE}📖 Choose publishing option:${NC}"
echo "1. GitHub Pages (recommended)"
echo "2. Copy to custom directory"
echo "3. Create archive for manual upload"
echo "4. Show docs.rs information"
echo "5. Local preview server"

read -p "Enter choice (1-5): " choice

case $choice in
    1)
        echo -e "${BLUE}🌐 Publishing to GitHub Pages...${NC}"
        
        # Check if gh-pages branch exists
        if ! git show-ref --verify --quiet refs/heads/gh-pages; then
            echo -e "${YELLOW}📝 Creating gh-pages branch...${NC}"
            git checkout --orphan gh-pages
            git rm -rf .
            echo "# Documentation" > README.md
            git add README.md
            git commit -m "Initial gh-pages commit"
            git checkout main
        fi
        
        # Build and copy documentation
        echo -e "${YELLOW}🔨 Building documentation...${NC}"
        cargo doc --no-deps
        
        # Switch to gh-pages branch
        git checkout gh-pages
        
        # Remove old docs
        rm -rf docs/
        
        # Copy new docs
        mkdir -p docs
        cp -r target/doc/* docs/
        
        # Create index.html redirect
        cat > docs/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta http-equiv="refresh" content="0; url=anthropic_sdk/index.html">
    <title>Anthropic Rust SDK Documentation</title>
</head>
<body>
    <p><a href="anthropic_sdk/index.html">Click here if not redirected automatically</a></p>
</body>
</html>
EOF
        
        # Add and commit
        git add docs/
        git commit -m "Update documentation $(date)"
        
        echo -e "${GREEN}✅ Documentation prepared for GitHub Pages!${NC}"
        echo -e "${YELLOW}📋 Next steps:${NC}"
        echo "   1. Push the gh-pages branch: git push origin gh-pages"
        echo "   2. Go to your GitHub repository settings"
        echo "   3. Enable GitHub Pages with source: gh-pages branch /docs folder"
        echo "   4. Your docs will be available at: https://<username>.github.io/<repo>/docs/"
        
        # Switch back to main branch
        git checkout main
        ;;
        
    2)
        read -p "Enter destination directory: " dest_dir
        echo -e "${BLUE}📁 Copying documentation to $dest_dir...${NC}"
        
        if [ ! -d "$dest_dir" ]; then
            mkdir -p "$dest_dir"
        fi
        
        cp -r target/doc/* "$dest_dir/"
        echo -e "${GREEN}✅ Documentation copied to $dest_dir${NC}"
        echo -e "${YELLOW}📋 You can now upload the contents to your web server${NC}"
        ;;
        
    3)
        echo -e "${BLUE}📦 Creating documentation archive...${NC}"
        
        archive_name="anthropic-sdk-docs-$(date +%Y%m%d-%H%M%S).tar.gz"
        tar -czf "$archive_name" -C target/doc .
        
        echo -e "${GREEN}✅ Documentation archived as: $archive_name${NC}"
        echo -e "${YELLOW}📋 Upload this archive to your hosting platform${NC}"
        ;;
        
    4)
        echo -e "${BLUE}📚 docs.rs Information${NC}"
        echo ""
        echo -e "${YELLOW}🔍 About docs.rs:${NC}"
        echo "   • Automatic documentation hosting for published crates"
        echo "   • Builds docs automatically when you publish to crates.io"
        echo "   • Available at: https://docs.rs/anthropic-sdk"
        echo ""
        echo -e "${YELLOW}📋 To use docs.rs:${NC}"
        echo "   1. Publish your crate: cargo publish"
        echo "   2. docs.rs will automatically build and host your documentation"
        echo "   3. Documentation will be available within minutes"
        echo ""
        echo -e "${YELLOW}📝 Cargo.toml configuration for docs.rs:${NC}"
        echo "   [package.metadata.docs.rs]"
        echo "   all-features = true"
        echo "   rustdoc-args = [\"--cfg\", \"docsrs\"]"
        ;;
        
    5)
        echo -e "${BLUE}🌐 Starting local preview server...${NC}"
        
        # Check if Python is available
        if command -v python3 &> /dev/null; then
            echo -e "${GREEN}🚀 Server starting at http://localhost:8000${NC}"
            echo -e "${YELLOW}📖 Navigate to: http://localhost:8000/anthropic_sdk/${NC}"
            echo -e "${RED}🛑 Press Ctrl+C to stop the server${NC}"
            cd target/doc && python3 -m http.server 8000
        elif command -v python &> /dev/null; then
            echo -e "${GREEN}🚀 Server starting at http://localhost:8000${NC}"
            echo -e "${YELLOW}📖 Navigate to: http://localhost:8000/anthropic_sdk/${NC}"
            echo -e "${RED}🛑 Press Ctrl+C to stop the server${NC}"
            cd target/doc && python -m SimpleHTTPServer 8000
        else
            echo -e "${RED}❌ Python not found. Please install Python to use the preview server.${NC}"
            echo -e "${YELLOW}📋 Alternative: Open target/doc/anthropic_sdk/index.html in your browser${NC}"
        fi
        ;;
        
    *)
        echo -e "${RED}❌ Invalid choice. Please run the script again.${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}🎉 Documentation publishing complete!${NC}"

# Additional hosting options
echo ""
echo -e "${BLUE}🌐 Additional Hosting Options:${NC}"
echo ""
echo "📌 Netlify Drop:"
echo "   • Visit https://app.netlify.com/drop"
echo "   • Drag and drop the target/doc folder"
echo "   • Get instant hosting with custom URL"
echo ""
echo "📌 Vercel:"
echo "   • Install: npm i -g vercel"
echo "   • Run: vercel target/doc"
echo "   • Follow prompts for deployment"
echo ""
echo "📌 GitHub Actions (Automated):"
echo "   • Create .github/workflows/docs.yml"
echo "   • Automate documentation building and deployment"
echo ""
echo "📌 Custom Domain:"
echo "   • Upload target/doc/* to your web server"
echo "   • Configure web server to serve static files"
echo "   • Point your domain to the server" 