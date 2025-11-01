# Claude AI Integration Demo

Create TAK objects using natural language commands through the Claude-powered interactive interface.

## Quick Start

**Terminal 1 - Start Server:**
```bash
target/release/omnitak --config config.yaml
```

**Terminal 2 - Interactive Demo:**
```bash
cd claude-interface
python3 interactive_demo.py
```

## Example Commands

```bash
# Create a polygon
polygon 34.0,-118.0 34.0,-117.0 33.5,-117.0 MyArea blue

# Create a circle (5km radius)
circle 34.0522,-118.2437 5 ExclusionZone red

# Create a route
route 34.0,-118.0:Start 34.1,-118.1:Mid 34.2,-118.2:End Patrol yellow
```

Objects appear instantly on all connected TAK clients!

## Architecture

```
Natural Language → Python CoT Builder → omniTAK API → TAK Servers → ATAK/WinTAK
```

See `/claude-interface/README.md` for full integration details.
