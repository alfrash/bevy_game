# Code Outline: [main.rs](file:///d:/temp/bevy_game/src/main.rs) and [player.rs](file:///d:/temp/bevy_game/src/player.rs)

## 1. [main.rs](file:///d:/temp/bevy_game/src/main.rs)
This is the entry point of the Bevy game, responsible for setting up the core application and basic environment.

- **App Initialization ([main](file:///d:/temp/bevy_game/src/main.rs#6-19) function):**
  - Sets the background clear color to white.
  - Adds `DefaultPlugins`, overridden slightly to set the asset directory path to `src/assets`.
  - Registers the `Startup` system `setup_camera` to spawn a 2D camera.
  - Adds the `PlayerPlugin` from `player.rs`.
  - Runs the Bevy `App`.
- **Systems:**
  - `setup_camera`: Spawns a standard Bevy `Camera2d`.

## 2. `player.rs`
This module handles all player-related logic, including spawning, movement, and sprite animation using a texture atlas.

### Constants
- Defines constants for tile size (`TILE_SIZE`), animation specifications (`WALK_FRAMES`, `ANIM_DT`), and movement mechanics (`MOVE_SPEED`).

### Components
- `Player`: A marker component to easily query the player entity.
- `Facing` (Enum): Represents the direction the player is looking (`UP`, `DOWN`, `LEFT`, `RIGHT`).
- `AnimationTimer`: A wrapper around Bevy's `Timer` to control animation frame speed.
- `AnimationState`: Tracks the current state of the player's animation:
  - `facing`: Current facing direction.
  - `moving`: Whether the player is currently moving.
  - `was_moving`: Previous frame's movement state (used to detect when movement starts/stops).

### Plugin Setup
- `PlayerPlugin`: Implements Bevy's `Plugin` trait to encapsulate player systems.
  - **Startup**: Adds `spawn_player`.
  - **Update**: Adds `move_player` and `animate_player` to run every frame.

### Core Systems
- **`spawn_player` (Startup System):**
  - Loads the player spritesheet (`male_spritesheet.png`).
  - Creates a `TextureAtlasLayout` based on grid properties (9 columns, 12 rows, 64x64 tiles).
  - Determines the initial frame index using helper functions.
  - Spawns the player entity with: `Sprite`, `Transform`, `Player` marker, `AnimationState`, and `AnimationTimer`.
- **`move_player` (Update System):**
  - Checks for WASD keyboard input.
  - Calculates a normalized movement direction and updates the player's `Transform.translation` according to `MOVE_SPEED` and `time.delta_secs()`.
  - Updates `AnimationState` based on dominant movement direction and sets `moving` to `true` or `false`.
- **`animate_player` (Update System):**
  - Handles the visual animation logic.
  - Computes the correct visual row and column within the texture atlas based on the `facing` state.
  - Snaps to new animation rows immediately when facing changes.
  - Steps through individual animation frames based on the `AnimationTimer` cadence if the player is moving.
  - Detects edge cases: instantly advancing on the first frame of movement (`just_started`) and holding the previous frame visually when stopping to prevent jarring snaps.

### Helper Functions
- `row_start_index`: Calculations to find the starting index of a given animation row based on `Facing`.
- `atlas_index_for`: Combines row start and a specified frame offset to yield an absolute atlas index.
- `row_zero_based`: Maps the `Facing` enum values to the specific 0-indexed rows in the generic template spritesheet (e.g., UP = 8, DOWN = 10).
