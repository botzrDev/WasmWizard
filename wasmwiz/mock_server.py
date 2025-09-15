#!/usr/bin/env python3
"""
Mock server to serve WasmWiz templates for UI testing.
This allows testing the web interface without needing the full Rust compilation.
"""

import os
import re
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

# Template directory
TEMPLATES_DIR = "/workspaces/WasmWiz/wasmwiz/templates"
STATIC_DIR = "/workspaces/WasmWiz/wasmwiz/static"

class WasmWizHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed_url = urlparse(self.path)
        path = parsed_url.path

        # Handle static files
        if path.startswith('/static/'):
            self.serve_static_file(path[8:])  # Remove '/static/' prefix
            return

        # Route mapping
        route_map = {
            '/': 'index.html',
            '/api-keys': 'api_keys.html',
            '/docs': 'docs.html',
            '/examples': 'examples.html',
            '/pricing': 'pricing.html',
            '/faq': 'faq.html',
            '/support': 'support.html',
            '/security': 'security.html',
            '/terms': 'terms.html',
            '/privacy': 'privacy.html'
        }

        template_file = route_map.get(path)
        if template_file:
            self.serve_template(template_file, path)
        else:
            self.send_error(404, f"Page not found: {path}")

    def serve_static_file(self, filepath):
        """Serve static files (CSS, JS, etc.)"""
        full_path = os.path.join(STATIC_DIR, filepath)

        if not os.path.exists(full_path):
            self.send_error(404, f"Static file not found: {filepath}")
            return

        # Determine content type
        if filepath.endswith('.css'):
            content_type = 'text/css'
        elif filepath.endswith('.js'):
            content_type = 'application/javascript'
        elif filepath.endswith('.ico'):
            content_type = 'image/x-icon'
        else:
            content_type = 'application/octet-stream'

        try:
            with open(full_path, 'rb') as f:
                content = f.read()

            self.send_response(200)
            self.send_header('Content-Type', content_type)
            self.send_header('Content-Length', str(len(content)))
            self.end_headers()
            self.wfile.write(content)
        except Exception as e:
            self.send_error(500, f"Error serving static file: {e}")

    def serve_template(self, template_file, route):
        """Load and render a template file"""
        template_path = os.path.join(TEMPLATES_DIR, template_file)

        if not os.path.exists(template_path):
            self.send_error(404, f"Template not found: {template_file}")
            return

        try:
            with open(template_path, 'r', encoding='utf-8') as f:
                content = f.read()

            # Simple template variable substitution
            context = {
                'title': self.get_page_title(route),
                'csrf_token': 'mock-csrf-token-123',
                'active_page': self.get_active_page(route)
            }

            # Replace template variables
            rendered = self.render_template(content, context)

            self.send_response(200)
            self.send_header('Content-Type', 'text/html; charset=utf-8')
            self.send_header('Content-Length', str(len(rendered.encode('utf-8'))))
            self.end_headers()
            self.wfile.write(rendered.encode('utf-8'))

        except Exception as e:
            self.send_error(500, f"Error rendering template: {e}")

    def render_template(self, content, context):
        """Simple template rendering - replace Askama-style variables"""

        # Handle extends and includes
        if '{% extends "base.html" %}' in content:
            # Load base template
            base_path = os.path.join(TEMPLATES_DIR, 'base.html')
            if os.path.exists(base_path):
                with open(base_path, 'r', encoding='utf-8') as f:
                    base_content = f.read()

                # Extract content blocks
                content_match = re.search(r'{% block content %}(.*?){% endblock %}', content, re.DOTALL)
                if content_match:
                    block_content = content_match.group(1).strip()
                    # Replace the content block in base template
                    rendered = re.sub(r'{% block content %}.*?{% endblock %}', block_content, base_content, flags=re.DOTALL)
                else:
                    rendered = base_content

                # Handle title block replacement
                title_match = re.search(r'{% block title %}(.*?){% endblock %}', content, re.DOTALL)
                if title_match:
                    title_content = title_match.group(1).strip()
                    rendered = re.sub(r'{% block title %}.*?{% endblock %}', title_content, rendered, flags=re.DOTALL)
                else:
                    # Use default title from context
                    rendered = re.sub(r'{% block title %}.*?{% endblock %}', context.get('title', 'WasmWiz'), rendered, flags=re.DOTALL)
            else:
                rendered = content
        else:
            rendered = content

        # Replace template variables
        for key, value in context.items():
            rendered = rendered.replace('{{ ' + key + ' }}', str(value))

        # Handle conditional active page classes
        active_page = context.get('active_page', '')
        rendered = re.sub(
            r'class="([^"]*?){% if active_page == "([^"]+)" %}active{% endif %}([^"]*?)"',
            lambda m: f'class="{m.group(1)}{"active" if active_page == m.group(2) else ""}{m.group(3)}"',
            rendered
        )

        return rendered

    def get_page_title(self, route):
        """Get page title based on route"""
        titles = {
            '/': 'Execute WebAssembly - WasmWiz',
            '/api-keys': 'API Key Management - WasmWiz',
            '/docs': 'API Documentation - WasmWiz',
            '/examples': 'WebAssembly Examples - WasmWiz',
            '/pricing': 'Pricing Plans - WasmWiz',
            '/faq': 'Frequently Asked Questions - WasmWiz',
            '/support': 'Get Support - WasmWiz',
            '/security': 'Security & Compliance - WasmWiz',
            '/terms': 'Terms of Service - WasmWiz',
            '/privacy': 'Privacy Policy - WasmWiz'
        }
        return titles.get(route, 'WasmWiz - WebAssembly Execution API')

    def get_active_page(self, route):
        """Get active page identifier"""
        active_pages = {
            '/': 'index',
            '/api-keys': 'api-keys',
            '/docs': 'docs',
            '/examples': 'examples',
            '/pricing': 'pricing',
            '/faq': 'faq',
            '/support': 'support',
            '/security': 'security',
            '/terms': 'terms',
            '/privacy': 'privacy'
        }
        return active_pages.get(route, '')

def run_server(port=8080):
    """Start the mock server"""
    server_address = ('0.0.0.0', port)
    httpd = HTTPServer(server_address, WasmWizHandler)
    print(f"🧙‍♂️ WasmWiz Mock Server starting on port {port}")
    print(f"📖 Serving templates from: {TEMPLATES_DIR}")
    print(f"📁 Serving static files from: {STATIC_DIR}")
    print(f"🌐 Open http://localhost:{port} to view the website")
    print("Press Ctrl+C to stop the server")

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nServer stopped.")
        httpd.server_close()

if __name__ == '__main__':
    run_server()