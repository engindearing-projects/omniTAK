"""
Claude Agent SDK Custom Tools for TAK Geometry Creation

This module provides natural language interfaces for creating TAK objects
like circles, polygons, routes, and markers.
"""

from typing import List, Tuple, Optional
from dataclasses import dataclass
from datetime import datetime, timedelta
import uuid


@dataclass
class LatLon:
    """Geographic coordinate"""
    lat: float
    lon: float
    hae: float = 0.0  # Height above ellipsoid


@dataclass
class Circle:
    """Circle geometry for exclusion zones, range rings"""
    center: LatLon
    radius_meters: float


@dataclass
class Polygon:
    """Polygon geometry for areas, zones"""
    vertices: List[LatLon]


@dataclass
class Route:
    """Route/path with waypoints"""
    waypoints: List[Tuple[LatLon, str]]  # (position, name)


class CotMessageBuilder:
    """Builds CoT XML messages for TAK objects"""

    @staticmethod
    def create_circle_event(
        circle: Circle,
        callsign: str,
        color: str = "red",
        uid: Optional[str] = None
    ) -> str:
        """
        Create a CoT event for a circular zone

        Args:
            circle: Circle geometry
            callsign: Display name for the circle
            color: Color (red, green, blue, yellow)
            uid: Unique identifier (generated if None)

        Returns:
            CoT XML message string
        """
        uid = uid or f"circle-{uuid.uuid4()}"
        now = datetime.utcnow()
        stale = now + timedelta(hours=24)

        # Color mapping
        colors = {
            "red": "-65536",
            "green": "-16711936",
            "blue": "-16776961",
            "yellow": "-256"
        }
        color_value = colors.get(color, "-65536")

        xml = f'''<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="{uid}" type="u-d-f" how="h-g-i-g-o">
  <point lat="{circle.center.lat}" lon="{circle.center.lon}" hae="{circle.center.hae}" ce="9999999" le="9999999"/>
  <time>{now.isoformat()}Z</time>
  <start>{now.isoformat()}Z</start>
  <stale>{stale.isoformat()}Z</stale>
  <detail>
    <contact callsign="{callsign}"/>
    <link uid="{uid}" type="a-f-G-E-V-C" relation="p-p"/>
    <shape>
      <ellipse major="{circle.radius_meters}" minor="{circle.radius_meters}" angle="0"/>
    </shape>
    <color value="{color_value}"/>
    <strokeColor value="{color_value}"/>
    <strokeWeight value="2.0"/>
    <labels_on value="true"/>
  </detail>
</event>'''
        return xml

    @staticmethod
    def create_polygon_event(
        polygon: Polygon,
        callsign: str,
        color: str = "green",
        filled: bool = True,
        uid: Optional[str] = None
    ) -> str:
        """
        Create a CoT event for a polygon area

        Args:
            polygon: Polygon geometry
            callsign: Display name
            color: Border color
            filled: Whether to fill the polygon
            uid: Unique identifier

        Returns:
            CoT XML message string
        """
        uid = uid or f"polygon-{uuid.uuid4()}"
        now = datetime.utcnow()
        stale = now + timedelta(hours=24)

        colors = {
            "red": "-65536",
            "green": "-16711936",
            "blue": "-16776961",
            "yellow": "-256"
        }
        color_value = colors.get(color, "-16711936")
        fill_value = "1342177280" if filled else "0"  # Semi-transparent

        # Build vertex list
        vertices_xml = "\n        ".join(
            f'<vertex lat="{v.lat}" lon="{v.lon}" hae="{v.hae}"/>'
            for v in polygon.vertices
        )

        # Use first vertex as the main point
        first = polygon.vertices[0]

        xml = f'''<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="{uid}" type="u-d-f" how="h-g-i-g-o">
  <point lat="{first.lat}" lon="{first.lon}" hae="{first.hae}" ce="9999999" le="9999999"/>
  <time>{now.isoformat()}Z</time>
  <start>{now.isoformat()}Z</start>
  <stale>{stale.isoformat()}Z</stale>
  <detail>
    <contact callsign="{callsign}"/>
    <link relation="p-p"/>
    <shape>
      <polyline closed="true">
        {vertices_xml}
      </polyline>
    </shape>
    <color value="{color_value}"/>
    <fillColor value="{fill_value}"/>
    <strokeColor value="{color_value}"/>
    <strokeWeight value="2.0"/>
    <labels_on value="true"/>
  </detail>
</event>'''
        return xml

    @staticmethod
    def create_route_event(
        route: Route,
        route_name: str,
        color: str = "yellow",
        uid: Optional[str] = None
    ) -> List[str]:
        """
        Create CoT events for a route with waypoints

        Args:
            route: Route with waypoints
            route_name: Name of the route
            color: Route color
            uid: Unique identifier for route

        Returns:
            List of CoT XML messages (route + waypoints)
        """
        uid = uid or f"route-{uuid.uuid4()}"
        now = datetime.utcnow()
        stale = now + timedelta(hours=24)

        colors = {
            "red": "-65536",
            "green": "-16711936",
            "blue": "-16776961",
            "yellow": "-256"
        }
        color_value = colors.get(color, "-256")

        messages = []
        waypoint_links = []
        waypoint_messages = []

        # Create waypoint messages and links
        for idx, (position, name) in enumerate(route.waypoints):
            wp_uid = f"{uid}-wp-{idx}"
            waypoint_links.append(
                f'<link relation="c" type="b-m-p-s-p-loc" uid="{wp_uid}"/>'
            )

            wp_xml = f'''<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="{wp_uid}" type="b-m-p-s-p-loc" how="h-g-i-g-o">
  <point lat="{position.lat}" lon="{position.lon}" hae="{position.hae}" ce="10" le="10"/>
  <time>{now.isoformat()}Z</time>
  <start>{now.isoformat()}Z</start>
  <stale>{stale.isoformat()}Z</stale>
  <detail>
    <contact callsign="{name}"/>
    <labels_on value="true"/>
  </detail>
</event>'''
            waypoint_messages.append(wp_xml)

        # Create main route message
        first = route.waypoints[0][0]
        links_xml = "\n    ".join(waypoint_links)

        route_xml = f'''<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="{uid}" type="b-m-p-s-p-loc" how="h-g-i-g-o">
  <point lat="{first.lat}" lon="{first.lon}" hae="{first.hae}" ce="9999999" le="9999999"/>
  <time>{now.isoformat()}Z</time>
  <start>{now.isoformat()}Z</start>
  <stale>{stale.isoformat()}Z</stale>
  <detail>
    <contact callsign="{route_name}"/>
    {links_xml}
    <labels_on value="true"/>
    <color value="{color_value}"/>
  </detail>
</event>'''

        # Return route first, then waypoints
        return [route_xml] + waypoint_messages

    @staticmethod
    def create_marker_event(
        position: LatLon,
        callsign: str,
        marker_type: str = "friendly",
        remarks: Optional[str] = None,
        uid: Optional[str] = None
    ) -> str:
        """
        Create a CoT event for a simple marker

        Args:
            position: Marker location
            callsign: Display name
            marker_type: Type (friendly, hostile, neutral, unknown, emergency, medical)
            remarks: Optional notes
            uid: Unique identifier

        Returns:
            CoT XML message string
        """
        uid = uid or f"marker-{uuid.uuid4()}"
        now = datetime.utcnow()
        stale = now + timedelta(hours=12)

        # Type mapping (MIL-STD-2525)
        types = {
            "friendly": "a-f-G-E-S",       # Friendly ground equipment
            "hostile": "a-h-G-E-S",        # Hostile ground equipment
            "neutral": "a-n-G-E-S",        # Neutral
            "unknown": "a-u-G-E-S",        # Unknown
            "emergency": "b-a-o-tl",       # Emergency beacon
            "medical": "b-m-p-c",          # Medical casualty
        }
        event_type = types.get(marker_type, "a-f-G-E-S")

        remarks_xml = f"<remarks>{remarks}</remarks>" if remarks else ""

        xml = f'''<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="{uid}" type="{event_type}" how="h-g-i-g-o">
  <point lat="{position.lat}" lon="{position.lon}" hae="{position.hae}" ce="10" le="10"/>
  <time>{now.isoformat()}Z</time>
  <start>{now.isoformat()}Z</start>
  <stale>{stale.isoformat()}Z</stale>
  <detail>
    <contact callsign="{callsign}"/>
    {remarks_xml}
    <labels_on value="true"/>
  </detail>
</event>'''
        return xml


# Example usage
if __name__ == "__main__":
    builder = CotMessageBuilder()

    # Create a 5km exclusion zone
    circle = Circle(
        center=LatLon(37.7749, -122.4194, 0),
        radius_meters=5000
    )
    circle_msg = builder.create_circle_event(circle, "5km Exclusion Zone", "red")
    print("CIRCLE MESSAGE:")
    print(circle_msg)
    print("\n" + "="*80 + "\n")

    # Create an area polygon
    polygon = Polygon(vertices=[
        LatLon(34.0, -118.0),
        LatLon(34.0, -117.0),
        LatLon(33.5, -117.0),
        LatLon(33.5, -118.0),
    ])
    poly_msg = builder.create_polygon_event(polygon, "Area of Operations Bravo", "green")
    print("POLYGON MESSAGE:")
    print(poly_msg)
    print("\n" + "="*80 + "\n")

    # Create a patrol route
    route = Route(waypoints=[
        (LatLon(33.123, -117.456), "Firebase Alpha"),
        (LatLon(33.234, -117.567), "Checkpoint Bravo"),
        (LatLon(33.345, -117.678), "Objective Charlie"),
    ])
    route_msgs = builder.create_route_event(route, "Patrol Route Alpha", "yellow")
    print("ROUTE MESSAGES:")
    for msg in route_msgs:
        print(msg)
        print("\n" + "-"*80 + "\n")
