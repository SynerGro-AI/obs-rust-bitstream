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
- [x] Gradient map

### Filters
- [x] Box blur
- [x] Sharpen
- [x] Gaussian blur
- [x] Motion blur
- [x] Radial blur
- [x] Add noise
- [x] Median
- [x] Emboss
- [x] Find edges
- [x] High pass

### Layer styles
- [x] Drop shadow (dark)
- [x] Bright shadow
- [x] Inner glow
- [x] Outer glow
- [x] Bevel and emboss
- [x] Stroke
- [x] Gradient overlay
- [x] Pattern overlay
- [x] Satin

### Selections
- [x] Rectangular / elliptical marquee
- [x] Lasso
- [x] Magic wand (tolerance)
- [x] Quick select / smart select
- [x] Invert selection
- [x] Feather
- [x] Expand / contract
- [x] Save / load selection

### Layers
- [x] Layer stack
- [x] Hide / show
- [x] Opacity
- [x] Blend modes
- [x] Layer mask
- [x] Clipping mask
- [x] Group

### Tools
- [x] Brush / eraser
- [x] Clone stamp
- [x] Healing
- [x] Dodge / burn / sponge
- [x] Gradient
- [x] Paint bucket
- [x] Text
- [x] Shape
- [x] Crop
- [x] Transform (scale, rotate, skew, warp)
- [x] Zoom in / out
- [x] Hand / pan

## After Effects

### Core
- [x] Timeline / clips
- [x] Compositor
- [x] Keyframes
- [x] Parenting
- [x] Nulls
- [x] Expressions (pure math)

### Effects categories
- [ ] Color correction (curves, color balance, levels — shared with Photoshop)
- [ ] Blur & sharpen (shared)
- [x] Distort (wave, bulge, twirl)
- [x] Generate (gradient, grid, lens flare)
- [x] Keying (chroma, luma)
- [x] Matte (track matte, set matte)
- [x] Noise & grain
- [x] Stylize (glow, find edges)
- [x] Time (echo, posterize time)
- [x] Transition (fade, wipe — partial)
- [x] 3D channel (depth, fog)
- [x] Audio (shared with existing filters)

### 3D
- [x] 3D layers
- [x] Camera
- [x] Lights

## Maya

### Modeling
- [x] Polygon mesh (verts, edges, faces)
- [x] Extrude, bevel, bridge
- [x] NURBS curve / surface
- [x] Subdivision

### Rigging
- [x] Joint hierarchy
- [x] IK / FK
- [x] Constraints
- [x] Skinning weights

### Animation
- [x] Keyframes / timeline (partial)
- [x] Graph editor (curves)
- [x] Motion paths
- [x] Blend shapes

### Dynamics
- [x] Particles
- [x] Rigid body
- [x] Soft body / cloth
- [x] Fluids (later)

### Look / render
- [x] Materials (partial)
- [x] Lights (partial)
- [x] Shaders (Lambert, Phong, PBR)
- [x] UV mapping
- [ ] Camera (perspective, orthographic)

## Live weights

AI writes a suggestion into LiveWeights once via WeightBridge.
Later frames read the weight. No new AI call is needed to re-apply
the same curve, balance, or effect.

Covered: glow, outer glow, shadow, bright, sharpen, hue/sat,
exposure, threshold, gauss, motion, noise, emboss, bevel, stroke,
gradient, pattern.

Also linked: light intensity, physics gravity/friction/mass,
bone translation. AI writes once; the scene reads.

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
