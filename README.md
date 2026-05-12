# crucible

crucible is a custom engine designed for procedural magic systems. the goal is to move away from hardcoded skills and instead let player abilities emerge from the physics of the world.

### motivation
the core idea comes from the isekai trope where you pick a set of starter skills from a point pool. instead of those skills being static animations, they are elementary components. if you cast water and then apply high heat, the engine calculates the phase transition to steam and the resulting pressure. this lets players "invent" their own meta, like steam-powered jumps or flaming arrows, just by combining simple rules.

### technical approach
* **grid-based simulation**: the world is managed in a grid of cells, each with its own material properties and state.
* **material interactions**: physics rules define how different elements like water, fire, and air interact at the cell level.
* **rust engine**: built in rust for performance and safety, using a modular architecture for the renderer and simulation logic.

it is currently in early development and the name is just a placeholder.
