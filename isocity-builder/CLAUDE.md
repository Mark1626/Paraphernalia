# Isocity Builder

Isometric medieval city-building game rendered on HTML5 Canvas.

## Project structure

```
js/
  main.js      — Entry point, canvas setup, camera, game loop, input handling
  renderer.js  — Canvas rendering (ground tiles, objects, highlight, compass, debug labels)
  assets.js    — Sprite manifest (MANIFEST keys) and image loader
  iso.js       — Isometric math: toScreen/toGrid, tile constants (TILE_W=95, TILE_H=48)
  camera.js    — Pan/zoom camera with screenToWorld conversion
  world.js     — Map generation: ground tiles, river pathing, tree clusters, rock placement
  game.js      — Game state and turn logic (resources, AP, building placement, phases)
  hud.js       — Screen-space HUD (resources, turn/AP, phase prompts)
```

## Architecture

- **Two-grid map**: `ground[row][col]` (string: grass/ground/river sprite key) and `objects[row][col]` (string or null: trees/stones/buildings)
- **Two-pass rendering**: Ground grid drawn first, then objects grid on top (separate anchor math)
- **Asset keys** use directional naming: `river_2_n_s` means a river tile connecting North and South in our grid coordinate system
- **Grid directions**: N = row-1, S = row+1, E = col+1, W = col-1

## Game rules (see PLAN.md)

- Turn-based: 1 AP per turn
- Phase flow: place_house → play → turn_over → (Enter) next turn
- Workshops harvest adjacent tile resources (trees→wood, stones→stone)
- Houses produce 1 food per turn

## Key conventions

- Sprite keys in `assets.js` MANIFEST must match names used in `world.js` and `game.js`
- River/road sprites follow `{type}_{connection_count}_{sorted_directions}` naming (e.g. `river_2_n_e`)
- Debug labels and compass are active in renderer — remove `drawLabel` calls and `renderCompass` for production
- All draw coordinates use `Math.round()` to avoid sub-pixel rendering gaps
- `toScreen()` also rounds to prevent fractional positions from `TILE_W/2 = 47.5`

## Running

Open `index.html` in a browser (requires ES modules support). No build step.
