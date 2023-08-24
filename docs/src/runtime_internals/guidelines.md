# Guidelines

This document contains guidelines for contributing to Ambient. These will be updated as the project evolves.

## Performance

### Error handling

- Be careful with the use of `anyhow`, especially `context` and `with_context`. The context methods capture a backtrace for their error case, which can be expensive, especially if done in aggregate (i.e. in a hot loop). If your code is likely to discard the error, consider using a dedicated error type or `Option` instead.
