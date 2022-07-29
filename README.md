# Doublets

A library that represents database engine that uses doublets.

## [Overview](https://github.com/linksplatform)

## Example

A basic CRUD in doublets

```rust
use doublets::{
    data::Flow::Continue,
    mem::FileMapped,
    unit, Doublets,
};

fn main() -> Result<(), doublets::Error<usize>> {
    // use file as memory for doublets
    let mem = FileMapped::from_path("db.links")?;
    let mut links = united::Store::<usize, _>::new(mem)?;

    // Create empty doublet in tiny style
    let mut point = links.create()?;

    // Update doublet in handler style
    // The link is updated to reference itself twice (as source and target):
    links.update_with(point, point, point, |_, after| {
        // link is { index, source, target }
        point = after.index;
        // give handler state (any ops::Try)
        Continue
    })?;

    // print all links from store
    links.each(|link| {
        println!("{link:?}");
        Continue
    });

    // The link deletion in full style:
    // `any` constant denotes any link
    let any = links.constants().any;
    // query in [index source target] style
    // delete all links with index = point
    links.delete_by_with([point, any, any], |before, _| {
        println!("Goodbye {}", before);
        Continue
    })
}
```
