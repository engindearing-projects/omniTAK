"""
OmniTAK HTTP/WebSocket API Client

Python client for interacting with the omniTAK REST API and WebSocket streams.
"""

import aiohttp
import asyncio
from typing import List, Optional, Dict, Any
from dataclasses import dataclass
from datetime import datetime
import json
import logging

logger = logging.getLogger(__name__)


@dataclass
class ConnectionInfo:
    """TAK server connection information"""
    id: str
    name: str
    connection_type: str
    status: str
    address: str
    messages_received: int
    messages_sent: int
    bytes_received: int
    bytes_sent: int


@dataclass
class SystemStatus:
    """omniTAK system status"""
    uptime_seconds: int
    active_connections: int
    messages_processed: int
    messages_per_second: float
    memory_usage_bytes: int
    version: str


class OmniTAKError(Exception):
    """Base exception for omniTAK client errors"""
    pass


class OmniTAKClient:
    """
    Asynchronous client for omniTAK REST API

    Example:
        async with OmniTAKClient("http://localhost:9443") as client:
            # Optionally authenticate
            await client.login("username", "password")

            # Send a CoT message
            response = await client.send_cot_message(cot_xml)
            print(f"Sent to {response['sent_to_count']} connections")

            # Get system status
            status = await client.get_status()
            print(f"Active connections: {status.active_connections}")
    """

    def __init__(
        self,
        base_url: str = "http://localhost:9443",
        api_key: Optional[str] = None,
        timeout: int = 30
    ):
        """
        Initialize omniTAK client

        Args:
            base_url: Base URL of omniTAK API (e.g., "http://localhost:9443")
            api_key: Optional API key for authentication
            timeout: Request timeout in seconds
        """
        self.base_url = base_url.rstrip('/')
        self.api_key = api_key
        self.timeout = aiohttp.ClientTimeout(total=timeout)
        self.session: Optional[aiohttp.ClientSession] = None
        self._token: Optional[str] = None

    async def __aenter__(self):
        """Async context manager entry"""
        await self.connect()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit"""
        await self.close()

    async def connect(self):
        """Create HTTP session"""
        if self.session is None:
            self.session = aiohttp.ClientSession(timeout=self.timeout)

    async def close(self):
        """Close HTTP session"""
        if self.session:
            await self.session.close()
            self.session = None

    def _get_headers(self) -> Dict[str, str]:
        """Get HTTP headers with authentication"""
        headers = {"Content-Type": "application/json"}

        if self._token:
            headers["Authorization"] = f"Bearer {self._token}"
        elif self.api_key:
            headers["X-API-Key"] = self.api_key

        return headers

    async def _request(
        self,
        method: str,
        endpoint: str,
        data: Optional[Dict] = None,
        params: Optional[Dict] = None
    ) -> Dict[str, Any]:
        """
        Make HTTP request to omniTAK API

        Args:
            method: HTTP method (GET, POST, DELETE)
            endpoint: API endpoint (e.g., "/api/v1/status")
            data: Request body (for POST/PUT)
            params: Query parameters

        Returns:
            Response JSON as dictionary

        Raises:
            OmniTAKError: On API errors
        """
        if not self.session:
            await self.connect()

        url = f"{self.base_url}{endpoint}"
        headers = self._get_headers()

        try:
            async with self.session.request(
                method, url, json=data, params=params, headers=headers
            ) as response:
                response_text = await response.text()

                if response.status >= 400:
                    try:
                        error_data = json.loads(response_text)
                        error_msg = error_data.get('message', response_text)
                    except json.JSONDecodeError:
                        error_msg = response_text

                    raise OmniTAKError(
                        f"API error {response.status}: {error_msg}"
                    )

                return json.loads(response_text)

        except aiohttp.ClientError as e:
            raise OmniTAKError(f"HTTP request failed: {e}")
        except json.JSONDecodeError as e:
            raise OmniTAKError(f"Invalid JSON response: {e}")

    async def login(self, username: str, password: str) -> str:
        """
        Authenticate with username/password and get JWT token

        Args:
            username: Username
            password: Password

        Returns:
            JWT token
        """
        response = await self._request(
            "POST",
            "/api/v1/auth/login",
            data={"username": username, "password": password}
        )

        self._token = response["token"]
        logger.info(f"Logged in as {username}")
        return self._token

    async def get_status(self) -> SystemStatus:
        """
        Get system status

        Returns:
            SystemStatus object
        """
        data = await self._request("GET", "/api/v1/status")

        return SystemStatus(
            uptime_seconds=data["uptime_seconds"],
            active_connections=data["active_connections"],
            messages_processed=data["messages_processed"],
            messages_per_second=data["messages_per_second"],
            memory_usage_bytes=data["memory_usage_bytes"],
            version=data["version"]
        )

    async def health_check(self) -> bool:
        """
        Check if omniTAK is healthy

        Returns:
            True if healthy
        """
        try:
            data = await self._request("GET", "/api/v1/health")
            return data.get("status") == "healthy"
        except OmniTAKError:
            return False

    async def get_connections(self) -> List[ConnectionInfo]:
        """
        Get list of TAK server connections

        Returns:
            List of ConnectionInfo objects
        """
        data = await self._request("GET", "/api/v1/connections")

        connections = []
        for conn in data.get("connections", []):
            connections.append(ConnectionInfo(
                id=conn["id"],
                name=conn["name"],
                connection_type=conn["connection_type"],
                status=conn["status"],
                address=conn["address"],
                messages_received=conn.get("messages_received", 0),
                messages_sent=conn.get("messages_sent", 0),
                bytes_received=conn.get("bytes_received", 0),
                bytes_sent=conn.get("bytes_sent", 0)
            ))

        return connections

    async def send_cot_message(
        self,
        cot_xml: str,
        target_connections: Optional[List[str]] = None,
        apply_filters: bool = True,
        priority: int = 5
    ) -> Dict[str, Any]:
        """
        Inject a CoT message into omniTAK for distribution to TAK servers

        Args:
            cot_xml: CoT message in XML format
            target_connections: Optional list of connection IDs to send to
                              If None, broadcasts to all connected servers
            apply_filters: Whether to apply filter rules (default True)
            priority: Message priority 0-10, higher = more important

        Returns:
            Dictionary with:
                - message_id: Unique ID assigned to message
                - sent_to_count: Number of connections sent to
                - sent_to_connections: List of connection IDs
                - warnings: Any warnings during processing
                - timestamp: When message was processed

        Example:
            >>> cot_xml = '''<?xml version="1.0" encoding="UTF-8"?>
            ... <event version="2.0" uid="test-123" type="a-f-G" ...>
            ... </event>'''
            >>> response = await client.send_cot_message(cot_xml)
            >>> print(f"Sent to {response['sent_to_count']} servers")
        """
        data = {
            "message": cot_xml,
            "apply_filters": apply_filters,
            "priority": priority
        }

        if target_connections:
            data["target_connections"] = target_connections

        response = await self._request("POST", "/api/v1/cot/send", data=data)

        if response.get("warnings"):
            for warning in response["warnings"]:
                logger.warning(f"CoT message warning: {warning}")

        logger.info(
            f"CoT message {response['message_id']} sent to "
            f"{response['sent_to_count']} connection(s)"
        )

        return response

    async def create_connection(
        self,
        name: str,
        address: str,
        protocol: str = "tcp",
        **kwargs
    ) -> Dict[str, Any]:
        """
        Create a new TAK server connection

        Args:
            name: Connection name/label
            address: Server address (host:port)
            protocol: Connection protocol (tcp, tls, udp, websocket)
            **kwargs: Additional protocol-specific options

        Returns:
            Connection info dictionary
        """
        data = {
            "name": name,
            "address": address,
            "protocol": protocol,
            **kwargs
        }

        response = await self._request("POST", "/api/v1/connections", data=data)
        logger.info(f"Created connection {name} to {address}")
        return response

    async def delete_connection(self, connection_id: str) -> Dict[str, Any]:
        """
        Delete a TAK server connection

        Args:
            connection_id: UUID of connection to delete

        Returns:
            Success message
        """
        response = await self._request(
            "DELETE", f"/api/v1/connections/{connection_id}"
        )
        logger.info(f"Deleted connection {connection_id}")
        return response


# Example usage
async def main():
    """Example usage of OmniTAKClient"""

    # Using context manager (recommended)
    async with OmniTAKClient("http://localhost:9443") as client:
        # Check health
        if not await client.health_check():
            print("omniTAK is not healthy!")
            return

        # Get system status
        status = await client.get_status()
        print(f"omniTAK v{status.version}")
        print(f"Uptime: {status.uptime_seconds}s")
        print(f"Active connections: {status.active_connections}")
        print(f"Messages/sec: {status.messages_per_second:.2f}")

        # Get connections
        connections = await client.get_connections()
        print(f"\nConnections ({len(connections)}):")
        for conn in connections:
            print(f"  {conn.name}: {conn.status} ({conn.address})")
            print(f"    RX: {conn.messages_received} msgs, {conn.bytes_received} bytes")
            print(f"    TX: {conn.messages_sent} msgs, {conn.bytes_sent} bytes")

        # Send a test CoT message
        test_cot = '''<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="python-test-001" type="a-f-G-E-S" time="2025-01-01T12:00:00Z" start="2025-01-01T12:00:00Z" stale="2025-01-01T13:00:00Z" how="h-g-i-g-o">
  <point lat="37.7749" lon="-122.4194" hae="100" ce="10" le="10"/>
  <detail>
    <contact callsign="Python Test"/>
  </detail>
</event>'''

        response = await client.send_cot_message(test_cot)
        print(f"\nSent CoT message {response['message_id']}")
        print(f"  Delivered to {response['sent_to_count']} connection(s)")
        if response['warnings']:
            print(f"  Warnings: {response['warnings']}")


if __name__ == "__main__":
    # Configure logging
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

    # Run example
    asyncio.run(main())
