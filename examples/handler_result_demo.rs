//! Demonstration of the HandlerResult trait solving issue #4
//!
//! This example shows how the HandlerResult trait eliminates the need for 
//! unnecessary trailing `Flow::Continue` returns in handlers.
//!
//! ## Before (Issue #4):
//! ```rust
//! links.each(|link| {
//!     worker.work(link);
//!     Flow::Continue // <- Unnecessary boilerplate :(
//! });
//! ```
//!
//! ## After (This solution):
//! ```rust
//! links.each(|link| {
//!     worker.work(link);
//!     // No trailing Flow::Continue needed! :)
//! });
//! ```

#![feature(try_trait_v2)]

use std::ops::{ControlFlow, Try, FromResidual};

// Mock types for demonstration
#[derive(Debug, PartialEq)]
pub enum Flow {
    Continue,
    Break,
}

impl Try for Flow {
    type Output = ();
    type Residual = ();

    fn from_output(_: ()) -> Self {
        Flow::Continue
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Flow::Continue => ControlFlow::Continue(()),
            Flow::Break => ControlFlow::Break(()),
        }
    }
}

impl FromResidual<()> for Flow {
    fn from_residual(_: ()) -> Self {
        Flow::Break
    }
}

/// The new HandlerResult trait that solves issue #4
pub trait HandlerResult {
    type Try: Try<Output = ()>;

    fn try_it(self) -> Self::Try;
}

impl HandlerResult for () {
    type Try = Flow;

    fn try_it(self) -> Self::Try {
        Flow::Continue
    }
}

impl HandlerResult for Flow {
    type Try = Flow;

    fn try_it(self) -> Self::Try {
        self
    }
}

#[derive(Debug)]
pub struct Link {
    pub index: u32,
}

struct Worker;

impl Worker {
    fn work(&mut self, link: Link) {
        println!("Processing link {}", link.index);
    }
}

struct MockLinks {
    links: Vec<Link>,
}

impl MockLinks {
    fn new() -> Self {
        Self {
            links: vec![
                Link { index: 1 },
                Link { index: 2 },
                Link { index: 3 },
            ],
        }
    }

    /// New each method using HandlerResult trait
    fn each<F, R>(&self, mut handler: F) -> R::Try
    where
        F: FnMut(Link) -> R,
        R: HandlerResult,
    {
        let mut output = R::Try::from_output(());
        
        for link in self.links.iter() {
            let result = handler(Link { index: link.index }).try_it();
            match result.branch() {
                ControlFlow::Continue(_) => continue,
                ControlFlow::Break(residual) => {
                    output = R::Try::from_residual(residual);
                    break;
                }
            }
        }
        
        output
    }
}

fn main() {
    println!("🚀 HandlerResult trait demo - solving issue #4\n");
    
    let links = MockLinks::new();
    let mut worker = Worker;
    
    println!("=== Old way (what users had to write before): ===");
    println!("links.each(|link| {{");
    println!("    worker.work(link);");
    println!("    Flow::Continue // <- Annoying boilerplate! 😞");
    println!("}});");
    println!();
    
    println!("=== New way (what users can write now): ===");
    println!("links.each(|link| {{");
    println!("    worker.work(link);");
    println!("    // No trailing Flow::Continue needed! 😍");
    println!("}});");
    println!();
    
    println!("=== Running the new way: ===");
    links.each(|link| {
        worker.work(link);
        // No trailing Flow::Continue needed! This is the solution to issue #4!
    });
    
    println!();
    println!("✅ SUCCESS: HandlerResult trait eliminates unnecessary Flow::Continue!");
    println!("🎉 Issue #4 solved!");
}