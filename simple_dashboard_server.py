#!/usr/bin/env python3
"""
Simple HTTP server to preview OmniTAK dashboard
Serves static files from crates/omnitak-api/web/static/
"""

import http.server
import socketserver
import os
import json
from datetime import datetime

PORT = 8443

# Mock OpenAPI spec for testing
MOCK_OPENAPI_SPEC = {
    "openapi": "3.0.0",
    "info": {
        "title": "OmniTAK API",
        "version": "0.1.0",
        "description": "REST API and WebSocket interface for TAK aggregator"
    },
    "servers": [
        {"url": "http://localhost:8443", "description": "Development server"}
    ],
    "paths": {
        "/api/v1/status": {
            "get": {
                "summary": "Get system status",
                "tags": ["system"],
                "responses": {
                    "200": {
                        "description": "System status",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "status": {"type": "string"},
                                        "uptime": {"type": "integer"},
                                        "version": {"type": "string"}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/connections": {
            "get": {
                "summary": "List all connections",
                "tags": ["connections"],
                "responses": {
                    "200": {
                        "description": "List of connections"
                    }
                }
            },
            "post": {
                "summary": "Create new connection",
                "tags": ["connections"],
                "requestBody": {
                    "content": {
                        "application/json": {
                            "schema": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "address": {"type": "string"},
                                    "protocol": {"type": "string"}
                                }
                            }
                        }
                    }
                },
                "responses": {
                    "201": {
                        "description": "Connection created"
                    }
                }
            }
        },
        "/health": {
            "get": {
                "summary": "Health check",
                "tags": ["system"],
                "responses": {
                    "200": {
                        "description": "Service is healthy",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "status": {"type": "string"},
                                        "timestamp": {"type": "string"}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

class CustomHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # Add CORS headers to all responses
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, PATCH, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization, X-API-Key')
        http.server.SimpleHTTPRequestHandler.end_headers(self)

    def do_OPTIONS(self):
        """Handle CORS preflight requests"""
        self.send_response(200)
        self.end_headers()

    def do_GET(self):
        # Serve mock OpenAPI spec
        if self.path == '/api-docs/openapi.json':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps(MOCK_OPENAPI_SPEC).encode())
            return

        # Serve mock health endpoint
        if self.path == '/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "status": "healthy",
                "timestamp": datetime.now().isoformat()
            }
            self.wfile.write(json.dumps(response).encode())
            return

        # Serve mock status endpoint
        if self.path == '/api/v1/status':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "status": "operational",
                "uptime": 3600,
                "version": "0.1.0",
                "active_connections": 3,
                "messages_processed": 12543
            }
            self.wfile.write(json.dumps(response).encode())
            return

        # Mock list connections
        if self.path == '/api/v1/connections':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "connections": [
                    {"id": "conn-1", "name": "TAK Server 1", "status": "connected", "address": "192.168.1.100:8087"},
                    {"id": "conn-2", "name": "TAK Server 2", "status": "connected", "address": "192.168.1.101:8087"},
                    {"id": "conn-3", "name": "TAK Server 3", "status": "disconnected", "address": "192.168.1.102:8087"}
                ],
                "total": 3
            }
            self.wfile.write(json.dumps(response).encode())
            return

        # Default to serving static files
        return http.server.SimpleHTTPRequestHandler.do_GET(self)

    def do_POST(self):
        """Handle POST requests"""
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length).decode('utf-8') if content_length > 0 else '{}'

        try:
            request_data = json.loads(body)
        except:
            request_data = {}

        # Mock create connection
        if self.path == '/api/v1/connections':
            self.send_response(201)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "success": True,
                "connection_id": "conn-new-123",
                "message": "Connection created successfully",
                "data": request_data
            }
            self.wfile.write(json.dumps(response).encode())
            return

        # Mock login
        if self.path == '/api/v1/auth/login':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "success": True,
                "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.mock.token",
                "user": {
                    "username": request_data.get("username", "demo"),
                    "role": "admin"
                }
            }
            self.wfile.write(json.dumps(response).encode())
            return

        # Default response
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()
        response = {
            "success": True,
            "message": "Mock POST response",
            "received_data": request_data
        }
        self.wfile.write(json.dumps(response).encode())

    def do_PUT(self):
        """Handle PUT requests"""
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length).decode('utf-8') if content_length > 0 else '{}'

        try:
            request_data = json.loads(body)
        except:
            request_data = {}

        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()
        response = {
            "success": True,
            "message": "Resource updated successfully",
            "updated_data": request_data
        }
        self.wfile.write(json.dumps(response).encode())

    def do_DELETE(self):
        """Handle DELETE requests"""
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()
        response = {
            "success": True,
            "message": f"Resource at {self.path} deleted successfully"
        }
        self.wfile.write(json.dumps(response).encode())

    def do_PATCH(self):
        """Handle PATCH requests"""
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length).decode('utf-8') if content_length > 0 else '{}'

        try:
            request_data = json.loads(body)
        except:
            request_data = {}

        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()
        response = {
            "success": True,
            "message": "Resource patched successfully",
            "patched_fields": request_data
        }
        self.wfile.write(json.dumps(response).encode())

if __name__ == '__main__':
    # Change to the static files directory
    web_dir = os.path.join(os.path.dirname(__file__), 'crates', 'omnitak-api', 'web', 'static')
    os.chdir(web_dir)

    with socketserver.TCPServer(("", PORT), CustomHandler) as httpd:
        print(f"╔══════════════════════════════════════════════════════════════╗")
        print(f"║                                                              ║")
        print(f"║              🚀 OmniTAK Dashboard Server                     ║")
        print(f"║                                                              ║")
        print(f"╚══════════════════════════════════════════════════════════════╝\n")
        print(f"Server running at: http://localhost:{PORT}")
        print(f"\n📄 Available pages:")
        print(f"   • Dashboard:           http://localhost:{PORT}/")
        print(f"   • Custom API Docs:     http://localhost:{PORT}/api-docs.html")
        print(f"   • RapiDoc:             http://localhost:{PORT}/rapidoc.html")
        print(f"   • Redoc:               http://localhost:{PORT}/redoc.html")
        print(f"\n📡 Mock API endpoints:")
        print(f"   • OpenAPI Spec:        http://localhost:{PORT}/api-docs/openapi.json")
        print(f"   • Health Check:        http://localhost:{PORT}/health")
        print(f"   • System Status:       http://localhost:{PORT}/api/v1/status")
        print(f"\n✨ Press Ctrl+C to stop\n")

        httpd.serve_forever()
