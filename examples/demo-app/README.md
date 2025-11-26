# Dioxus Iconify Demo App

This example demonstrates how to use `dioxus-iconify` to generate and use Iconify icons in a Dioxus application.

## Quick Start

1. **Build the CLI tool** (from the root of the repository):
   ```bash
   cargo build --release
   ```

2. **Generate some icons** for this demo:
   ```bash
   # From the root directory
   cargo run -- --output examples/demo-app/src/icons add mdi:home mdi:account heroicons:arrow-left

   # Or if you've installed the CLI:
   cd examples/demo-app
   dioxus-iconify add mdi:home mdi:account heroicons:arrow-left
   ```

3. **Uncomment the icon usage** in `src/main.rs`:
   - Uncomment the `mod icons;` and `use` statements at the top
   - Uncomment the icon examples section at the bottom

4. **Run the demo**:
   ```bash
   dx serve
   # or
   cargo run
   ```

## What Gets Generated

When you run `dioxus-iconify add mdi:home mdi:account heroicons:arrow-left`, it creates:

```
src/icons/
├── mod.rs         # Icon component + IconData struct
├── mdi.rs         # Material Design Icons: Home, Account
└── heroicons.rs   # Heroicons: ArrowLeft
```

## Using Icons in Your Code

```rust
use dioxus::prelude::*;

mod icons;
use icons::Icon;
use icons::{mdi, heroicons};

fn App() -> Element {
    rsx! {
        // Basic usage
        Icon { data: mdi::Home }

        // With custom size and color
        Icon {
            data: mdi::Account,
            width: "32",
            height: "32",
            fill: "blue",
        }

        // With CSS class
        Icon {
            data: heroicons::ArrowLeft,
            class: "icon-class",
        }

        // All SVG attributes work
        Icon {
            data: mdi::Home,
            width: "48",
            height: "48",
            fill: "#ff0000",
            stroke: "black",
            stroke_width: "2",
            class: "my-icon",
        }
    }
}
```

## Key Points

- **No runtime dependency**: The icons are generated as Rust source code in your project
- **Type-safe**: Each icon is a const that's checked at compile time
- **Organized**: Icons are grouped by collection (mdi, heroicons, etc.)
- **Customizable**: All SVG attributes can be overridden via props
- **Committed to git**: The generated code should be committed to version control
