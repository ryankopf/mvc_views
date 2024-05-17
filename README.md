# mvc_views

A procedural macro to append rendering blocks to Actix Web handler functions in an MVC pattern.

## Features

- Automatically appends rendering blocks to functions within a module.
- Ensures functions return `HttpResponse` with rendered content from HTML templates.
- Simple to use: apply the macro to a module, and it processes all functions within.

## Installation

Add `mvc_views` to your `Cargo.toml`:

```toml
[dependencies]
mvc_views = "0.1.0"
