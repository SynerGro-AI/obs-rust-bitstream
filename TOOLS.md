# Full tool plan: Photoshop + After Effects + Maya → pure Rust

Goal: every capability of Photoshop, Adobe After Effects, and Maya that can be expressed as pure data/frame/math operations is added as a pure Rust module. Platform or GPU pieces come only when no pure path exists.

## Photoshop

### Adjustments
- [x] Levels
- [x] Curves
- [x] Color balance
- [x] Hue/saturation
- [x] Brightness/contrast
- [x] Exposure
- [x] Vibrance
- [x] Black & white
- [x] Photo filter
- [x] Channel mixer
- [x] Selective color
- [x] Invert
- [x] Posterize
- [x] Threshold
- [ ] Gradient map

### Filters
- [x] Box blur
- [x] Sharpen
- [x] Gaussian blur
- [x] Motion blur
- [ ] Radial blur
- [x] Add noise
- [ ] Median
- [x] Emboss
- [ ] Find edges
- [ ] High pass

### Layer styles
- [x] Drop shadow (dark)
- [x] Bright shadow
- [x] Inner glow
- [x] Outer glow
- [x] Bevel and emboss
- [x] Stroke
- [x] Gradient overlay
- [x] Pattern overlay
- [ ] Satin

### Selections
- [x] Rectangular / elliptical marquee
- [ ] Lasso
- [x] Magic wand (tolerance)
- [ ] Quick select / smart select
- [x] Invert selection
- [ ] Feather
- [ ] Expand / contract
- [ ] Save / load selection

### Layers
- [x] Layer stack
- [x] Hide / show
- [x] Opacity
- [x] Blend modes
- [ ] Layer mask
- [ ] Clipping mask
- [ ] Group

### Tools
- [ ] Brush / eraser
- [ ] Clone stamp
- [ ] Healing
- [ ] Dodge / burn / sponge
- [ ] Gradient
- [ ] Paint bucket
- [x] Text
- [ ] Shape
- [ ] Crop
- [ ] Transform (scale, rotate, skew, warp)
- [ ] Zoom in / out
- [ ] Hand / pan

## After Effects

### Core
- [x] Timeline / clips
- [x] Compositor
- [ ] Keyframes
- [ ] Parenting
- [ ] Nulls
- [ ] Expressions (pure math)

### Effects categories
- [ ] Color correction (curves, color balance, levels — shared with Photoshop)
- [ ] Blur & sharpen (shared)
- [ ] Distort (wave, bulge, twirl)
- [ ] Generate (gradient, grid, lens flare)
- [ ] Keying (chroma, luma)
- [ ] Matte (track matte, set matte)
- [ ] Noise & grain
- [ ] Stylize (glow, find edges)
- [ ] Time (echo, posterize time)
- [ ] Transition (fade, wipe — partial)
- [ ] 3D channel (depth, fog)
- [ ] Audio (shared with existing filters)

### 3D
- [ ] 3D layers
- [ ] Camera
- [ ] Lights

## Maya

### Modeling
- [ ] Polygon mesh (verts, edges, faces)
- [ ] Extrude, bevel, bridge
- [ ] NURBS curve / surface
- [ ] Subdivision

### Rigging
- [ ] Joint hierarchy
- [ ] IK / FK
- [ ] Constraints
- [ ] Skinning weights

### Animation
- [x] Keyframes / timeline (partial)
- [ ] Graph editor (curves)
- [ ] Motion paths
- [ ] Blend shapes

### Dynamics
- [ ] Particles
- [ ] Rigid body
- [ ] Soft body / cloth
- [ ] Fluids (later)

### Look / render
- [x] Materials (partial)
- [x] Lights (partial)
- [ ] Shaders (Lambert, Phong, PBR)
- [ ] UV mapping
- [ ] Camera (perspective, orthographic)

## Order of work

1. Curves, blend modes, color balance (this step)
2. Selection (rect, invert, magic wand) + layer stack with hide/mask
3. Remaining adjustments (hue/sat, exposure, invert, threshold)
4. Remaining filters (gaussian, motion, noise, emboss)
5. Remaining layer styles (bevel, stroke, gradient)
6. Tools (brush, clone, transform, zoom)
7. AE distort / generate / keying
8. Maya mesh + joint + skinning
9. Dynamics (particles, rigid)
10. Full 3D camera / lights / shaders

Each item is a pure module with a differential test, then wired into the library.
