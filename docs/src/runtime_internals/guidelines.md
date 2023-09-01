# Guidelines

This document contains guidelines for contributing to Ambient. These will be updated as the project evolves.

## Message style (errors, information)

When composing text that is shown to the user (including error messages and log messages), follow these guidelines:

- Use American English, as opposed to British English. This is the default for technical writing.
  - **Example**: "color" is preferable to "colour".
- Use the Oxford comma.
  - **Example**: "red, white, and blue" is preferable to "red, white and blue".
- Fully capitalize acronyms.
  - **Example**: "HTTP" is preferable to "http" or "Http".
  - Other common examples: "HTTP", "URL", "JSON", "TOML", "ECS"
- Use sentence case (i.e. capitalize the first word, and any proper nouns). Your errors could be at any layer of the stack, so they should read as complete sentences.
  - **Example**: "Server running" is preferable to "server running".
- Use the present tense if the message describes the current state of the system, or the past tense if it describes a past state.
  - **Example**: "Server running" when the server's started vs. "Server was running" when the server's stopped.
- Use the imperative mood for commands.
  - **Example**: "Run `cargo build` to build the project." is preferable to "You can run `cargo build` to build the project." as it is shorter and easier to read.
- Avoid being overly verbose, but don't be terse to the point of confusion.
  - **Example**: "Server running" is preferable to "The server is running" as it conveys the same amount of information, but is shorter and easier to read.
  - **Example**: "Error while processing the frobnicator stack" is preferable to "Frobnicator stack processing error" as it provides specific information about the why ("while") and the what ("the frobnicator stack"), while the latter is ambiguous and could be interpreted in multiple ways.
- Object IDs (packages, components, etc) should be referred to with surrounding backticks - except where already surrounded by parentheses - while names should be referred to with quotation marks.
  - **Example**: `` The package "Party Starter" (party_starter) does not have the component `boombox`. ``
- Paths should be referred to with quotation marks surrounding them. (The Rust debug implementation for paths does this automatically.)
  - **Example**: `Your file is located at "/home/user/file.txt".`
- The additional context (e.g. `anyhow`, but this applies to anything where errors are being nested) for an error should be a complete message, not a fragment, as errors at any level of the stack may be displayed.
  - **Example**: `Error while processing single pipeline in "lol.toml": No such file or directory (os error 2)` is preferable to `In pipeline "lol.toml": No such file or directory (os error 2)`.
- In general, try to optimize for easy copy-ability / clicking. If it's a link, you should be able to click it in your terminal without having to manually select it.
  - **Example**: `` Visit `https://example.com/` for more information. `` is preferable to `Visit https://example.com/ for more information.`, as the former is more likely to be clickable in a terminal.
- Use logging instead of `println`. This is because many events within Ambient have a time component to them, and the output should convey that to ensure the user is aware of the time that the event occurred.
  - **Example**: `log::info!("Server running")` is preferable to `println!("Server running")` as the latter does not convey the time that the event occurred.

## Performance

### Error handling

- Be careful with the use of `anyhow`, especially `context` and `with_context`. The context methods capture a backtrace for their error case, which can be expensive, especially if done in aggregate (i.e. in a hot loop). If your code is likely to discard the error, consider using a dedicated error type or `Option` instead.
