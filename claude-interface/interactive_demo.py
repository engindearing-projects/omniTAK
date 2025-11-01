#!/usr/bin/env python3
"""
Interactive OmniTAK Demo - Type commands to create TAK objects in real-time
"""

import asyncio
import aiohttp
from claude_tools.tak_geometry import CotMessageBuilder, Circle, Polygon, Route, LatLon

# Global API configuration
API_URL = "http://127.0.0.1:9443"
TOKEN = None

async def login():
    """Login and get authentication token"""
    global TOKEN
    async with aiohttp.ClientSession() as session:
        login_data = {
            "username": "admin",
            "password": "changeme"
        }
        async with session.post(f"{API_URL}/api/v1/auth/login", json=login_data) as resp:
            if resp.status == 200:
                result = await resp.json()
                TOKEN = result.get("access_token")
                return True
            return False

async def send_cot(cot_xml, description="TAK Object"):
    """Send CoT message to omniTAK API"""
    if not TOKEN:
        print("❌ Not authenticated. Please restart.")
        return False

    async with aiohttp.ClientSession() as session:
        headers = {
            "Authorization": f"Bearer {TOKEN}",
            "Content-Type": "application/json"
        }

        request_data = {
            "message": cot_xml,
            "apply_filters": False,
            "priority": 5
        }

        async with session.post(
            f"{API_URL}/api/v1/cot/send",
            json=request_data,
            headers=headers
        ) as resp:
            if resp.status == 200:
                result = await resp.json()
                print(f"✅ {description} sent successfully! (ID: {result['message_id'][:8]}...)")
                return True
            else:
                text = await resp.text()
                print(f"❌ Failed to send: {resp.status} - {text}")
                return False

def parse_coords(coord_str):
    """Parse coordinates from string like '34.0,-118.0' or '34.0 -118.0'"""
    parts = coord_str.replace(',', ' ').split()
    if len(parts) >= 2:
        return float(parts[0]), float(parts[1])
    return None, None

def print_help():
    """Print available commands"""
    print("\n📍 Available Commands:")
    print("=" * 80)
    print("  polygon <lat,lon> <lat,lon> <lat,lon> [name] [color]")
    print("    Example: polygon 34.0,-118.0 34.0,-117.0 33.5,-117.0 MyArea blue")
    print()
    print("  circle <lat,lon> <radius_km> [name] [color]")
    print("    Example: circle 34.0522,-118.2437 5 LAX_Zone red")
    print()
    print("  route <lat,lon:name> <lat,lon:name> <lat,lon:name> [route_name] [color]")
    print("    Example: route 34.0,-118.0:Start 34.1,-118.1:Mid 34.2,-118.2:End Patrol yellow")
    print()
    print("  help     - Show this help")
    print("  quit     - Exit program")
    print("=" * 80)
    print()

async def handle_polygon_command(parts):
    """Handle polygon creation command"""
    builder = CotMessageBuilder()

    # Parse vertices
    vertices = []
    name = "Interactive Polygon"
    color = "blue"

    i = 1
    while i < len(parts):
        lat, lon = parse_coords(parts[i])
        if lat is not None and lon is not None:
            vertices.append(LatLon(lat, lon))
            i += 1
        else:
            # Assume remaining are name and color
            if i < len(parts):
                name = parts[i]
                i += 1
            if i < len(parts):
                color = parts[i]
            break

    if len(vertices) < 3:
        print("❌ Need at least 3 coordinate pairs for a polygon")
        print("   Example: polygon 34.0,-118.0 34.0,-117.0 33.5,-117.0")
        return

    print(f"\n📐 Creating polygon with {len(vertices)} vertices:")
    for i, v in enumerate(vertices):
        print(f"   Point {i+1}: {v.lat}, {v.lon}")
    print(f"   Name: {name}")
    print(f"   Color: {color}\n")

    polygon = Polygon(vertices=vertices)
    cot_xml = builder.create_polygon_event(polygon, name, color)
    await send_cot(cot_xml, f"Polygon '{name}'")

async def handle_circle_command(parts):
    """Handle circle creation command"""
    if len(parts) < 3:
        print("❌ Usage: circle <lat,lon> <radius_km> [name] [color]")
        return

    builder = CotMessageBuilder()

    # Parse center coordinates
    lat, lon = parse_coords(parts[1])
    if lat is None or lon is None:
        print("❌ Invalid coordinates. Use format: 34.0,-118.0")
        return

    # Parse radius
    try:
        radius_km = float(parts[2])
    except ValueError:
        print("❌ Invalid radius. Use a number (kilometers)")
        return

    # Optional name and color
    name = parts[3] if len(parts) > 3 else "Interactive Circle"
    color = parts[4] if len(parts) > 4 else "red"

    print(f"\n⭕ Creating circle:")
    print(f"   Center: {lat}, {lon}")
    print(f"   Radius: {radius_km} km")
    print(f"   Name: {name}")
    print(f"   Color: {color}\n")

    circle = Circle(center=LatLon(lat, lon), radius_meters=radius_km * 1000)
    cot_xml = builder.create_circle_event(circle, name, color)
    await send_cot(cot_xml, f"Circle '{name}'")

async def handle_route_command(parts):
    """Handle route creation command"""
    if len(parts) < 3:
        print("❌ Usage: route <lat,lon:name> <lat,lon:name> ... [route_name] [color]")
        return

    builder = CotMessageBuilder()

    # Parse waypoints
    waypoints = []
    name = "Interactive Route"
    color = "yellow"

    i = 1
    while i < len(parts):
        part = parts[i]
        if ':' in part:
            # Format: lat,lon:name
            coord_part, wp_name = part.split(':', 1)
            lat, lon = parse_coords(coord_part)
            if lat is not None and lon is not None:
                waypoints.append((LatLon(lat, lon), wp_name))
                i += 1
            else:
                break
        else:
            # Try parsing as just coordinates
            lat, lon = parse_coords(part)
            if lat is not None and lon is not None:
                waypoints.append((LatLon(lat, lon), f"WP{len(waypoints)+1}"))
                i += 1
            else:
                # Assume remaining are name and color
                if i < len(parts):
                    name = parts[i]
                    i += 1
                if i < len(parts):
                    color = parts[i]
                break

    if len(waypoints) < 2:
        print("❌ Need at least 2 waypoints for a route")
        return

    print(f"\n🛣️  Creating route with {len(waypoints)} waypoints:")
    for i, (point, wp_name) in enumerate(waypoints):
        print(f"   {i+1}. {wp_name}: {point.lat}, {point.lon}")
    print(f"   Route Name: {name}")
    print(f"   Color: {color}\n")

    route = Route(waypoints=waypoints)
    route_msgs = builder.create_route_event(route, name, color)

    # Send all route messages (main route + waypoints)
    for idx, msg in enumerate(route_msgs):
        if idx == 0:
            await send_cot(msg, f"Route '{name}'")
        else:
            await send_cot(msg, f"  Waypoint {idx}")
            await asyncio.sleep(0.1)  # Small delay between waypoints

async def main():
    print("\n" + "=" * 80)
    print("🎯 OmniTAK Interactive Demo - Claude-Powered TAK Object Creator")
    print("=" * 80)
    print()
    print("Connecting to omniTAK API...")

    if not await login():
        print("❌ Failed to authenticate with omniTAK API")
        print("   Make sure the server is running:")
        print("   cd /Users/iesouskurios/omniTAK && target/release/omnitak --config config.yaml")
        return

    print("✅ Connected and authenticated!\n")
    print_help()

    print("Type your commands (or 'help' for examples, 'quit' to exit):")
    print("-" * 80)

    while True:
        try:
            # Get user input
            user_input = input("\n🎯 > ").strip()

            if not user_input:
                continue

            # Parse command
            parts = user_input.split()
            command = parts[0].lower()

            if command == 'quit' or command == 'exit':
                print("\n👋 Goodbye!\n")
                break

            elif command == 'help':
                print_help()

            elif command == 'polygon':
                await handle_polygon_command(parts)

            elif command == 'circle':
                await handle_circle_command(parts)

            elif command == 'route':
                await handle_route_command(parts)

            else:
                print(f"❌ Unknown command: {command}")
                print("   Type 'help' for available commands")

        except KeyboardInterrupt:
            print("\n\n👋 Interrupted. Goodbye!\n")
            break
        except Exception as e:
            print(f"❌ Error: {e}")
            print("   Type 'help' for correct command format")

if __name__ == "__main__":
    asyncio.run(main())
