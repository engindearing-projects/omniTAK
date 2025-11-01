"""
Claude Agent SDK Tools for TAK Operations

Natural language interface for creating TAK objects through Claude Code.
"""

import sys
import os
from typing import List, Tuple, Optional
import asyncio

# Add parent directory to path to import omnitak_client
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from omnitak_client import OmniTAKClient, OmniTAKError
from tak_geometry import (
    CotMessageBuilder, LatLon, Circle, Polygon, Route
)


# Claude Agent SDK tool decorators
# NOTE: Uncomment when claude-agent-sdk is installed
# from claude_agent_sdk import tool

# @tool
async def create_exclusion_zone(
    center_lat: float,
    center_lon: float,
    radius_km: float,
    zone_name: str,
    color: str = "red"
) -> str:
    """
    Create a circular exclusion zone on the TAK map

    Args:
        center_lat: Latitude of zone center
        center_lon: Longitude of zone center
        radius_km: Radius in kilometers
        zone_name: Display name for the zone
        color: Zone color (red, green, blue, yellow)

    Returns:
        Success message with zone details

    Example:
        User: "Create a 5km exclusion zone around 37.7749, -122.4194"
        Claude calls: create_exclusion_zone(37.7749, -122.4194, 5.0, "5km Exclusion Zone")
    """
    builder = CotMessageBuilder()
    circle = Circle(
        center=LatLon(center_lat, center_lon, 0),
        radius_meters=radius_km * 1000  # Convert km to meters
    )

    cot_message = builder.create_circle_event(circle, zone_name, color)

    try:
        async with OmniTAKClient() as client:
            result = await client.send_cot_message(cot_message)

        return f"✓ Created {zone_name}: {radius_km}km radius at ({center_lat}, {center_lon}). Sent to {result['sent_to_count']} TAK server(s)."
    except OmniTAKError as e:
        return f"✗ Failed to create zone: {e}"


# @tool
async def create_area_polygon(
    vertices: List[Tuple[float, float]],
    area_name: str,
    color: str = "green",
    filled: bool = True
) -> str:
    """
    Create a polygon area on the TAK map

    Args:
        vertices: List of (lat, lon) coordinate tuples defining the polygon
        area_name: Display name for the area
        color: Border color (red, green, blue, yellow)
        filled: Whether to fill the polygon

    Returns:
        Success message with area details

    Example:
        User: "Draw a restricted area with corners at 34.0,-118.0 and 34.0,-117.0 and 33.5,-117.0 and 33.5,-118.0"
        Claude calls: create_area_polygon([(34.0, -118.0), (34.0, -117.0), (33.5, -117.0), (33.5, -118.0)], "Restricted Area")
    """
    builder = CotMessageBuilder()
    polygon = Polygon(
        vertices=[LatLon(lat, lon, 0) for lat, lon in vertices]
    )

    cot_message = builder.create_polygon_event(polygon, area_name, color, filled)

    try:
        async with OmniTAKClient() as client:
            result = await client.send_cot_message(cot_message)

        return f"✓ Created polygon '{area_name}' with {len(vertices)} vertices. Sent to {result['sent_to_count']} TAK server(s)."
    except OmniTAKError as e:
        return f"✗ Failed to create polygon: {e}"


# @tool
async def create_patrol_route(
    waypoints: List[Tuple[float, float, str]],
    route_name: str,
    color: str = "yellow"
) -> str:
    """
    Create a patrol route with waypoints on the TAK map

    Args:
        waypoints: List of (lat, lon, name) tuples for each waypoint
        route_name: Display name for the route
        color: Route color (red, green, blue, yellow)

    Returns:
        Success message with route details

    Example:
        User: "Create a patrol route from Firebase Alpha at 33.123,-117.456 through Checkpoint Bravo at 33.234,-117.567 to Objective Charlie at 33.345,-117.678"
        Claude calls: create_patrol_route([
            (33.123, -117.456, "Firebase Alpha"),
            (33.234, -117.567, "Checkpoint Bravo"),
            (33.345, -117.678, "Objective Charlie")
        ], "Patrol Route Alpha")
    """
    builder = CotMessageBuilder()
    route = Route(
        waypoints=[(LatLon(lat, lon, 0), name) for lat, lon, name in waypoints]
    )

    cot_messages = builder.create_route_event(route, route_name, color)

    try:
        async with OmniTAKClient() as client:
            # Send all messages (route + waypoints)
            for msg in cot_messages:
                await client.send_cot_message(msg)

        return f"✓ Created route '{route_name}' with {len(waypoints)} waypoints. Sent to TAK server(s)."
    except OmniTAKError as e:
        return f"✗ Failed to create route: {e}"


# @tool
async def place_marker(
    lat: float,
    lon: float,
    callsign: str,
    marker_type: str = "friendly",
    remarks: Optional[str] = None
) -> str:
    """
    Place a marker/icon on the TAK map

    Args:
        lat: Latitude
        lon: Longitude
        callsign: Display name/callsign
        marker_type: Type of marker (friendly, hostile, neutral, unknown, emergency, medical)
        remarks: Optional notes/description

    Returns:
        Success message

    Example:
        User: "Mark 34.567, -118.123 as a casualty collection point named MEDEVAC-1"
        Claude calls: place_marker(34.567, -118.123, "MEDEVAC-1", "medical", "Casualty Collection Point")
    """
    builder = CotMessageBuilder()
    position = LatLon(lat, lon, 0)

    cot_message = builder.create_marker_event(position, callsign, marker_type, remarks)

    try:
        async with OmniTAKClient() as client:
            result = await client.send_cot_message(cot_message)

        return f"✓ Placed {marker_type} marker '{callsign}' at ({lat}, {lon}). Sent to {result['sent_to_count']} TAK server(s)."
    except OmniTAKError as e:
        return f"✗ Failed to place marker: {e}"


# @tool
async def get_tak_status() -> str:
    """
    Get the status of TAK server connections

    Returns:
        Status information about connected TAK servers

    Example:
        User: "What's the status of our TAK connections?"
        Claude calls: get_tak_status()
    """
    try:
        async with OmniTAKClient() as client:
            status = await client.get_status()
            connections = await client.get_connections()

        active = len([c for c in connections if c.status == 'connected'])
        total = len(connections)

        result = f"omniTAK Status:\n"
        result += f"  Version: {status.version}\n"
        result += f"  Uptime: {status.uptime_seconds}s\n"
        result += f"  Active connections: {active}/{total}\n"
        result += f"  Messages processed: {status.messages_processed}\n"
        result += f"  Messages/sec: {status.messages_per_second:.2f}\n"
        result += f"  Memory: {status.memory_usage_bytes / 1024 / 1024:.1f} MB\n\n"

        result += "Connections:\n"
        for conn in connections:
            status_icon = "✓" if conn.status == 'connected' else "✗"
            result += f"  {status_icon} {conn.name}: {conn.address} ({conn.connection_type})\n"
            result += f"      RX: {conn.messages_received} msgs, {conn.bytes_received} bytes\n"
            result += f"      TX: {conn.messages_sent} msgs, {conn.bytes_sent} bytes\n"

        return result
    except OmniTAKError as e:
        return f"✗ Failed to get status: {e}"


# @tool
async def query_tak_data(
    affiliation: Optional[str] = None,
    time_range_minutes: int = 5,
    geographic_bounds: Optional[Tuple[float, float, float, float]] = None
) -> str:
    """
    Query CoT messages from TAK servers

    Args:
        affiliation: Filter by affiliation (friend, hostile, neutral, unknown)
        time_range_minutes: How many minutes of history to retrieve
        geographic_bounds: Optional (min_lat, min_lon, max_lat, max_lon) bounding box

    Returns:
        Summary of messages matching the query

    Example:
        User: "Show me all hostile contacts from the last 10 minutes"
        Claude calls: query_tak_data(affiliation="hostile", time_range_minutes=10)
    """
    # TODO: Implement WebSocket streaming from omniTAK
    # This would connect to ws://localhost:9443/api/v1/stream
    # and filter messages based on parameters

    result = f"Querying TAK data:\n"
    result += f"  Affiliation: {affiliation or 'all'}\n"
    result += f"  Time range: last {time_range_minutes} minutes\n"

    if geographic_bounds:
        min_lat, min_lon, max_lat, max_lon = geographic_bounds
        result += f"  Geographic bounds: ({min_lat},{min_lon}) to ({max_lat},{max_lon})\n"

    result += "\n[WebSocket streaming implementation pending]"

    return result


# Example conversation flow
if __name__ == "__main__":
    print("Example Natural Language Commands → TAK Objects\n")
    print("=" * 80)

    examples = [
        {
            "user": "Create a 5km exclusion zone around San Francisco (37.7749, -122.4194)",
            "tool": "create_exclusion_zone",
            "params": {
                "center_lat": 37.7749,
                "center_lon": -122.4194,
                "radius_km": 5.0,
                "zone_name": "5km Exclusion Zone around San Francisco"
            }
        },
        {
            "user": "Draw a restricted airspace polygon with corners at 34.0,-118.0 and 34.0,-117.0 and 33.5,-117.0 and 33.5,-118.0",
            "tool": "create_area_polygon",
            "params": {
                "vertices": [(34.0, -118.0), (34.0, -117.0), (33.5, -117.0), (33.5, -118.0)],
                "area_name": "Restricted Airspace",
                "color": "red"
            }
        },
        {
            "user": "Create a patrol route from Firebase Alpha at 33.123,-117.456 through Checkpoint Bravo at 33.234,-117.567 to Objective Charlie at 33.345,-117.678",
            "tool": "create_patrol_route",
            "params": {
                "waypoints": [
                    (33.123, -117.456, "Firebase Alpha"),
                    (33.234, -117.567, "Checkpoint Bravo"),
                    (33.345, -117.678, "Objective Charlie")
                ],
                "route_name": "Patrol Route Alpha",
                "color": "yellow"
            }
        },
        {
            "user": "Place a medical marker at 34.567,-118.123 called MEDEVAC-1 for casualty collection",
            "tool": "place_marker",
            "params": {
                "lat": 34.567,
                "lon": -118.123,
                "callsign": "MEDEVAC-1",
                "marker_type": "medical",
                "remarks": "Casualty Collection Point"
            }
        }
    ]

    for example in examples:
        print(f"\nUser: {example['user']}")
        print(f"Claude calls: {example['tool']}({example['params']})")
        print("-" * 80)
