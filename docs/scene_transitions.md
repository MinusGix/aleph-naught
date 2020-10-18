## Scene transitions
This document describes methods for transitioning between scenes.
Goals:
- Knowing which scene we are in.
    - Essential for saving in the scene we are in.
    - As well as being able to potentialyl 'go back' a scene.  
      Even if it is re-running the scene.
- Passing around at minimum the `state` and `info`
    - Potentially passing around info directly associated with the scene, but that can be ignored in favor of storing it all in info if that is required.
        - State info would also be nice to support using structures with (de)serialization, as it allows member functions and updating old saves in the actual data.


### Simple goto methods
These goto methods would be performed at the end

### State director
`state.goto("aurum", info);`  
    Status: Negative.  
- This requires state to own the scenes, and also call the function while it is borrowed, and the function would use state.
- Grows stack

### Engine director (No Scenes)
`Engine::goto(state, info, "aurum");`  
Status: Possible.  
- This has the downside that we would need some form of global state (so essentially Engine being a singleton), which isn't awful, though it'd be a Arc<RefCell<Engine>>.
- The engine would own the scenes, likely you'd do: `Engine::start(state, scenes, info, "first_scene");` and it would steal the scenes.
    - You know, if we did this, then the goto can just become `Engine::goto("aurum");` as it stores the `state` and `info` on it and they are auto-passed!
- This also allows the pattern of having utility functions.
- grows stack

### Engine director (scenes)
`Engine::goto(state, scenes, info, "aurum");`  
Status: Possible, but funky.  
If this was done it would have to take the given data by-value.
- This would require you to borrow part of the scene structure (if it was a `HashMap<String, SceneInfo>`), get its function, and then call that.. with the `scenes`. Then that scene would use goto as well, thus creating multiple borrows.
    - Actually, this could potentially work as calling the `rune` functions does not require a mutable reference, but that might change in the future which makes this shaky.
