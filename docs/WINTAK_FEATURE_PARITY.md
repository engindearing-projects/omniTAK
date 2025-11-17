# WinTAK Feature Parity Roadmap

This document tracks features that WinTAK has that OmniTAK currently lacks, prioritized for implementation.

## Priority 1: Rich Mapping & Visualization
**Status: IN PROGRESS**

- [ ] Full geospatial rendering engine with GPU acceleration
- [ ] Offline map support (works without internet)
- [ ] Multiple map formats: KML/KMZ, GeoTIFF, shapefiles, MBTiles
- [ ] Terrain analysis and elevation data
- [ ] High-res satellite/aerial imagery

## Priority 2: Interactive Drawing & Annotation
**Status: NOT STARTED**

- [ ] Freehand drawing on maps
- [ ] Shape tools: circles, polygons, routes
- [ ] Measurement tools (distance, area)
- [ ] Range rings and threat domes
- [ ] Geofences with alerts
- [ ] Tactical graphics (military symbols)

## Priority 3: Mission Planning Workflows
**Status: NOT STARTED**

- [ ] Route planning with waypoints
- [ ] Checkpoint management
- [ ] Mission briefing tools
- [ ] Team coordination features
- [ ] Time-on-target calculations
- [ ] Deconfliction tools

## Priority 4: Blue Force Tracking UI
**Status: NOT STARTED**

- [ ] Real-time friendly force visualization
- [ ] Track history trails
- [ ] Speed/heading indicators
- [ ] Callsign labels
- [ ] Quick-select by team/group
- [ ] Proximity alerts

## Priority 5: Mature Plugin Ecosystem
**Status: PARTIAL (WASM infrastructure exists)**

- [ ] Video streaming plugins (drone feeds)
- [ ] Sensor integration (weather, CBRN)
- [ ] UAS (drone) control plugins
- [ ] Data link integrations
- [ ] Plugin marketplace/registry

## Priority 6: Enterprise Integration
**Status: NOT STARTED**

- [ ] Active Directory/LDAP authentication
- [ ] Windows domain integration
- [ ] Group Policy support
- [ ] Corporate certificate enrollment
- [ ] Government PKI integration

## Priority 7: User Experience Polish
**Status: PARTIAL (basic GUI exists)**

- [ ] Context menus and shortcuts
- [ ] Customizable workspace layouts
- [ ] Quick action buttons
- [ ] Rich tooltips and help
- [ ] Keyboard shortcuts
- [ ] Workspace persistence

## Priority 8: TAK Data Package Support
**Status: NOT STARTED**

TAK Data Packages (.zip/.dpk) are essential for:
- Device staging and configuration
- Mission planning data distribution
- Cross-device data sharing

- [ ] Create TAK Data Package generator (.zip/.dpk format)
- [ ] Implement manifest.xml builder (MissionPackageManifest v2)
- [ ] Support Configuration parameters (uid, name, onReceiveDelete)
- [ ] Handle Contents with ignore flags
- [ ] Add data package import/export UI
- [ ] Device staging workflows
- [ ] Batch data package creation for multiple devices

**Data Package Structure:**
```
<root>
  |___ file1
  |___ file2
  |___ MANIFEST/
         |___ manifest.xml
```

**Manifest Format:**
```xml
<MissionPackageManifest version="2">
  <Configuration>
    <Parameter name="uid" value="uuid-here"/>
    <Parameter name="name" value="package-name.zip"/>
    <Parameter name="onReceiveDelete" value="true"/>
  </Configuration>
  <Contents>
    <Content ignore="false" zipEntry="file1"/>
    <Content ignore="false" zipEntry="file2"/>
  </Contents>
</MissionPackageManifest>
```

## Priority 9: Government Support & Certification
**Status: NOT APPLICABLE (community project)**

- [ ] Security audit documentation
- [ ] Compliance documentation
- [ ] Official training materials

---

## Implementation Notes

### Technology Choices for Mapping

**Recommended Stack:**
- **GPU Rendering**: `wgpu` (Rust WebGPU implementation)
- **Map Tiles**: OpenStreetMap, Mapbox, or custom tile server
- **Geospatial**: `geo` crate for geometry operations
- **Projections**: `proj` crate for coordinate transformations
- **File Formats**:
  - KML/KMZ: `kml` crate or custom XML parsing
  - GeoTIFF: `geotiff` or `gdal` bindings
  - Shapefiles: `shapefile` crate
  - MBTiles: SQLite-based (already have support)

### Performance Targets
- 60 FPS map rendering
- <16ms frame time
- Support for 10,000+ markers
- Smooth pan/zoom at any scale
- Efficient tile caching (LRU)
