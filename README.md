# Aleph: Text Adventure Engine
This is a text adventure engine made to run on the web using wasm, as well as potentially more backends.  
It uses the [Rune](https://github.com/rune-rs/rune) scripting language for the writing of scenes, giving full logic capabilities.  
  
Example engine script (more complete example available in `examples/demon.rune`):  
```rust
pub fn entry(state) {
    state.add_scenes([
        ("town", town),
        ("forest", forest),
        ("tavern", tavern),
        ("demon_conv", demon_conv),
    ]);
    state.info.shoe_quest = false;
    state.goto("town");
}
fn town(state) {
    state.set_title("Town of Plenty");
    state.set_text("You stand in the middle of the town as people bustle around you in their daily life. At the far end of the town lies the gate, and from there adventure. Nearer is the tavern, where stories of valor and poor luck can be heard throughout the day.");

    // Displays the buttons with the text.
    // The callback given to the button is passed to the callback
    // given to `ask_choice`.
    state.ask_choice([
        Button::new("Leave Town", || "forest"),
        Button::new("Enter Tavern", || "tavern"),
    ], |choice| state.goto(choice()));
}
fn forest(state) {
    state.set_title("Forest of Darkness");
    state.set_text("In the forest of darkness it is quite dark, but unlike your expectations it is also quite loud. Birds chirp, fallen leaves crunch as animals pass by, and the occasional spontaneous explosion.");
    let choices = [
        Button::new("Flee to Town", || state.goto("town")),
        Button::new("Stay", || state.set_text("You died to a spontaneous explosion.")),
    ];
    if state.info.shoe_quest {
        choices.push(Button::new("Search for Shoes", || state.set_text("You died to a carnivorous shoe, and an explosion")));
    }
    state.ask_choice(choices, |func| func());
}
fn tavern(state) {
    state.set_title("Cheap Tavern");
    state.set_text("The room smells of cheap alcohol, vomit, and an excessive amount of cleaning magicks. In the corner there is a hooded man, but the clothing barely covers the obvious appearance of a demon of the ninth circle.");
    // Doesn't check if you've already talked with the demon.
    state.ask_choice([
        Button::new("Talk to the Demon", || "demon_conv"),
        Button::new("Leave Tavern", || "town"),
    ], |func| state.goto(func()));
}
fn demon_conv(state) {
    state.set_text("You sit down to speak to the demon, and have a surprisingly amiable conversation. He gives you a quest to go to the forest and look for his lost pair of shoes.");
    state.info.shoe_quest = true;
    state.ask_choice([
        Button::new("Continue", || "tavern"),
    ], |func| state.goto(func()));
}
```

# Building
Requires: Rust, and https://rustwasm.github.io/wasm-pack/installer/  
`wasm-pack build --dev --target web`  
This deliberately does not use the default bundler (webpack) as it was horrendusly slow (took ten seconds for every small change), and so does the without-a-bundler setup method.  
Then, once you've built that - though you only have to do this specific step once - you do `npm install` in the `www/` folder. This simply makes a symlink to the pkg folder inside the `node_modules` folder so that the webpage can access it.  
You could avoid using `npm` by making a symlink yourself to the `pkg/` folder in the `www/node_modules/aleph-naught`.  
Now that you have the built wasm accessible by the webpage, you can `cd www` and use any simple http-server you want. I personally use https://crates.io/crates/https.