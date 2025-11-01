#!/usr/bin/env python3
"""
Demo script to create and send a polygon to TAK via omniTAK API
"""

import asyncio
import aiohttp
from claude_tools.tak_geometry import CotMessageBuilder, Polygon, LatLon

async def send_cot_message(api_url: str, cot_xml: str):
    """Send a CoT message to omniTAK API"""
    async with aiohttp.ClientSession() as session:
        # Login first to get token
        login_data = {
            "username": "admin",
            "password": "changeme"
        }

        async with session.post(f"{api_url}/api/v1/auth/login", json=login_data) as resp:
            if resp.status != 200:
                print(f"Login failed: {resp.status}")
                return False

            result = await resp.json()
            token = result.get("access_token")

        if not token:
            print("Failed to get authentication token")
            return False

        print(f"✅ Authenticated successfully")

        # Send CoT message using JSON request format
        headers = {
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json"
        }

        # Format as SendCotRequest JSON
        request_data = {
            "message": cot_xml,
            "apply_filters": False,  # Bypass filters for demo
            "priority": 5
        }

        async with session.post(
            f"{api_url}/api/v1/cot/send",
            json=request_data,
            headers=headers
        ) as resp:
            if resp.status == 200:
                print(f"✅ Polygon sent successfully!")
                return True
            else:
                print(f"❌ Failed to send polygon: {resp.status}")
                text = await resp.text()
                print(f"Response: {text}")
                return False

async def main():
    print("=" * 80)
    print("OmniTAK Claude Demo - Creating TAK Polygon")
    print("=" * 80)
    print()

    # Create geometry builder
    builder = CotMessageBuilder()

    # Create a test polygon (Area of Operations)
    print("📍 Creating polygon with coordinates:")
    vertices = [
        LatLon(34.0, -118.0),
        LatLon(34.0, -117.0),
        LatLon(33.5, -117.0),
        LatLon(33.5, -118.0),
    ]

    for i, vertex in enumerate(vertices):
        print(f"   Point {i+1}: {vertex.lat}, {vertex.lon}")

    polygon = Polygon(vertices=vertices)

    # Generate CoT XML
    cot_xml = builder.create_polygon_event(
        polygon,
        "Demo Area - Claude Generated",
        "blue"
    )

    print()
    print("📄 Generated CoT XML:")
    print("-" * 80)
    print(cot_xml)
    print("-" * 80)
    print()

    # Send to omniTAK API
    api_url = "http://127.0.0.1:9443"
    print(f"🚀 Sending polygon to omniTAK API at {api_url}")
    print()

    success = await send_cot_message(api_url, cot_xml)

    if success:
        print()
        print("=" * 80)
        print("✅ SUCCESS! Check your TAK map - you should see the blue polygon!")
        print("=" * 80)
    else:
        print()
        print("=" * 80)
        print("❌ Failed to send polygon. Check if omniTAK server is running.")
        print("   Run: ../target/release/omnitak --config ../config.yaml")
        print("=" * 80)

if __name__ == "__main__":
    asyncio.run(main())
