// Bring in the core Bevy parts. `prelude` has all the essential types.
use bevy::prelude::*;

// --- Constants ---
// Size of a single tile/frame in our spritesheet in pixels.
const TILE_SIZE: u32 = 64;
// Number of animation frames per row (walking animation has 9 frames).
const WALK_FRAMES: usize = 9; 
// Player movement speed in pixels per second.
const MOVE_SPEED: f32 = 140.0; 
// How much time (in seconds) between each animation frame. 0.1s means 10 FPS.
const ANIM_DT: f32 = 0.1; 

// --- Components ---
// In the Entity Component System (ECS), Components are just plain data attached to an Entity.

// A "marker" component. We attach this to the player entity so we can easily find the player 
// later when querying the active entities.
#[derive(Component)]
pub struct Player;

// An enum representing which direction the player is looking.
// We derive common traits like Debug, Clone, Copy, and PartialEq so we can compare and copy it easily.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Facing {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

// A component that wraps a generic Bevy `Timer`.
// Deref and DerefMut allow us to use this struct exactly as if it were the inner `Timer`.
#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

// Groups together the state needed to control the player's animation.
#[derive(Component)]
struct AnimationState {
    facing: Facing,
    // Is the player currently moving?
    moving: bool,
    // Was the player moving in the previous frame? Used to detect when they just started or stopped.
    was_moving: bool,
}

// --- Plugin ---
// A Plugin is a way to group related systems, resources, and configuration together.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // We register `spawn_player` to run once at startup.
        app.add_systems(Startup, spawn_player)
            // We register `move_player` and `animate_player` to run every single frame (Update schedule).
            .add_systems(Update, (move_player, animate_player));
    }
}

// --- Systems ---

// Spawning the Player
// This system takes `Commands` (to spawn the entity), `AssetServer` (to load the image),
// and `Assets<TextureAtlasLayout>` (to register the layout of our spritesheet).
fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load the image file from the disk (from the src/assets folder as configured in main.rs).
    let texture = asset_server.load("male_spritesheet.png");
    
    // Create a layout describing how the image is divided into a grid.
    // We tell it the tile size (64x64), the number of columns (9), and rows (12).
    let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        // `UVec2::splat` creates a 2D vector where X and Y are identical.
        UVec2::splat(TILE_SIZE),
        WALK_FRAMES as u32,
        12,
        None,
        None,
    ));

    // Start facing down (towards user), idle on first frame of that row.
    let facing = Facing::DOWN;
    let start_index = atlas_index_for(facing, 0);

    // Spawning an entity creates it in the game world. We give it a tuple of components.
    commands.spawn((
        // Sprite component configured to use the texture atlas (spritesheet) we just created.
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout,
                index: start_index, // Which tile in the grid to render right now.
            },
        ),
        // Transform controls position, rotation, and scale. We start at the origin (0,0).
        Transform::from_translation(Vec3::ZERO),
        // Our marker component
        Player,
        // The initial animation state
        AnimationState { facing: Facing::DOWN, moving: false, was_moving: false },
        // The timer that controls when the next animation frame happens (0.1s repeating).
        AnimationTimer(Timer::from_seconds(ANIM_DT, TimerMode::Repeating)),
    ));
}

// Movement System
// Runs every frame to check input and move the player.
fn move_player(
    // Read keyboard input. `Res` means we are accessing a global Resource.
    input: Res<ButtonInput<KeyCode>>,
    // Access delta time (time since the last frame) to make movement framerate-independent.
    time: Res<Time>,
    // This `Query` finds exactly one entity that has `Transform`, `AnimationState`, and the `Player` marker.
    // The `With<Player>` filter means we ONLY want the player, not any other entity with a Transform.
    mut player: Query<(&mut Transform, &mut AnimationState), With<Player>>
){
    // standard Bevy pattern: try to get the single matching entity. If none exist (or more than 1), return early.
    let Ok((mut transform, mut anim)) = player.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    // Check which keys are currently being pressed.
    if input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        // `normalize()` ensures diagonal movement isn't faster than horizontal/vertical movement.
        // We multiply by our speed constant and `time.delta_secs()` so it scales with time, not frames.
        let delta = direction.normalize() * MOVE_SPEED * time.delta_secs();
        
        // Update the player's physical position on screen.
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
        
        // Mark that the player is indeed moving this frame.
        anim.moving = true;

        // Update facing dir based on dominant direction (which axis had more movement).
        if direction.x.abs() > direction.y.abs() {
            anim.facing = if direction.x > 0.0 { Facing::RIGHT } else { Facing::LEFT };
        } else {
            anim.facing = if direction.y > 0.0 { Facing::UP } else { Facing::DOWN };
        }
        
    } else {
        // If there was no input, they aren't moving.
        anim.moving = false;
    }
}

// Animation Implementation
// Runs every frame to update which part of the spritesheet we draw.
fn animate_player(
    time: Res<Time>,
    mut query: Query<(&mut AnimationState, &mut AnimationTimer, &mut Sprite), With<Player>>,
) {
    let Ok((mut anim, mut timer, mut sprite)) = query.single_mut() else {
        return;
    };

    // Grab mutable access to the texture atlas to change the visible `index`.
    let atlas = match sprite.texture_atlas.as_mut() {
        Some(a) => a,
        None => return,
    };

    // Compute what row of the spritesheet we SHOULD be on based on where we are facing.
    let target_row = row_zero_based(anim.facing);
    let mut current_col = atlas.index % WALK_FRAMES;
    let mut current_row = atlas.index / WALK_FRAMES;

    // If the facing direction changed, snap immediately to the start of the new row.
    if current_row != target_row {
        atlas.index = row_start_index(anim.facing);
        current_col = 0;
        // Reset the timer so the new animation cleanly starts from 0 time.
        timer.reset();
    }

    // Detect state transitions
    let just_started = anim.moving && !anim.was_moving;
    let just_stopped = !anim.moving && anim.was_moving;

    if anim.moving {
        if just_started {
            // "Juice" / Responsiveness: As soon as movement starts, immediately advance one visual frame 
            // so the player feels it instantly, rather than waiting 0.1s for the timer to tick.
            let row_start = row_start_index(anim.facing);
            let next_col = (current_col + 1) % WALK_FRAMES;
            atlas.index = row_start + next_col;
            // Restart the timer so the *next* advance takes the full 0.1s interval.
            timer.reset();
        } else {
            // Continuous movement: tick the timer with the time passed since last frame.
            timer.tick(time.delta());
            
            // If 0.1s has elapsed...
            if timer.just_finished() {
                let row_start = row_start_index(anim.facing);
                let next_col = (current_col + 1) % WALK_FRAMES;
                // Move the atlas index to the next column.
                atlas.index = row_start + next_col;
            }
        }
    } else if just_stopped {
        // If they just stopped moving, reset the timer so it's fresh for next time they start.
        timer.reset();
        // Notice we don't automatically snap back to column 0 here; 
        // we leave them on the frame they stopped at to prevent artificial snapping.
    }

    // Remember our state for the next frame.
    anim.was_moving = anim.moving;
}

// --- Helper Functions ---

// Returns the total, straight numerical starting index for a specific row.
// e.g., if row_zero_based is 8, and there are 9 frames per row, the start is index 72.
fn row_start_index(facing: Facing) -> usize {
    row_zero_based(facing) * WALK_FRAMES
}

// Gives us the exact atlas index for a given direction and column number.
fn atlas_index_for(facing: Facing, frame_in_row: usize) -> usize {
    // `.min(WALK_FRAMES - 1)` prevents us from accidentally asking for column 10 in a 9 column row.
    row_start_index(facing) + frame_in_row.min(WALK_FRAMES - 1)
}

// Maps our Facing enum to specific Y-rows in the generic 12-row spritesheet graphic.
fn row_zero_based(facing: Facing) -> usize {
    match facing {
        // The character in our spritesheet faces UP on the 9th row (index 8).
        Facing::UP => 8,
        // Character faces LEFT on 10th row (index 9).
        Facing::LEFT => 9,
        // Character faces DOWN on 11th row (index 10).
        Facing::DOWN => 10,
        // Character faces RIGHT on 12th row (index 11).
        Facing::RIGHT => 11,
    }
}