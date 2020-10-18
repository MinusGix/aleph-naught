This is a document relating how the various platforms work, will work, and information about them  

Shortenings used in this document:
- NREQ: Not-Required. Usually means that such a feature isn't required for sane behavior.
- REQ: Required.
- WANT: Would be nice to have support for.
- NIMPL: Not implemented. Implies that it would be fine if it was implemented.
- IMPL: It is implemented.

# Features
This part of the document describes a variety of useful features.  
Each feature describes how required/desired it is for the engine. Sub-sections requirements depend on their parent requirements being met.  
## Basic  
These are basic features that should be supported:
- Rune's println should have some way to output to allow easier debugging of programs. If needed, you can have a specific flag that needs to be set for it to work.
## Text
Status: REQ.
The ability to display text in some form is obviously required.
Having some form of title is common in scenes and will be easily supported on anything that supports text, but some backends may have nicer support. If it does not support some form of title then it can simply display at as somewhat centered text at the start of the normal text.
### API
**Clearing**  
`state.clear_text_display();`  
Clears all text, including the title.
`state.clear_text();`  
Clears just the text.  
`state.clear_title();`  
Clears just the title.  

**Display Title**  
`state.set_title("The World");`  
This would set the current title to that text.

**Displaying Text**  
`state.set_text("Hello world.");`  
Sets the current displayed text to this.

**Continuing Text**  
`state.append_text("The army marches again.");`  
Add the text to the end of the last text, with no newline or spaces. Any newlines desired should be manually put into the string.

## Bars
Status: WANT  
Having the ability to have some information put *somewhere* that the user can access at all time without having to write a custom scene would be quite nice.  
This could range from a health bar, to a full inventory display.  
A bar would be updated whenever the Engine has free time to do so (most likely scene transitions).  
### Proposed API
## Images
Status: NREQ.  
Images are purely for providing more information about the scenes, and since this is a text adventure engine, then they are likely not essential, but they do provide good information.  
Having the ability to display images in a bar would be useful. (Character pictures, maps, etc).
### Proposed API
**Engine::Support::image()**  
(?) Returns a boolean for if images are supported.
### In-Text Images
Status: NREQ
Having images that are available to be displayed in the middle of text would be very useful.
### Proposed API
This could be a form of formatting in text strings?



# Platforms
## WASM
This is a web-backend that displays itself on the web.
This aims to be the most customizable due to the sheer ability you have on the web.
### Support:
- Basic: print goes to a thread local `OUT` variable, but it does not drain it automatically.
- Text: NIMPL.
    - Formatting: NIMPL.
- Images: NIMPL. Might want to think about how bars should be implemented first.
- Bar: NIMPL.
### Proposed Api
Reserved namespace: `wasm`  
`wasm::alert(string)`: Show alert dialogue.
`wasm::confirm(string)`: Show confirm dialogue.
`wasm::log1(string)`: Logs a single string to the console. Useful for checking.
`wasm::log_info1(string)`: Logs a single string to the console, tells that it came from a rune script.

## Terminal User Interface (Simple) (NIMPL)
This would be a really simple backend that simply takes in input from the user in the terminal without much fancy terminal shenanigans.



## Terminal User Interface (Complex) (NIMPL)
This is a full setup terminal using complex terminal user interface that allows a nicer view and more powerful features.

## Graphical User Interface (NIMPL)  
A GUI backend in some library. It would be nice to use a pure rust one.
