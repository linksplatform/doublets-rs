//! Zero-cost decorators that add policy to any [`Doublets`](crate::Doublets) store.
//!
//! This module is a port of the decorator layer of `Platform.Data.Doublets`. Each C#
//! decorator has a counterpart here with the same logic, but composition is static: a
//! decorator is a generic struct that owns the store it wraps, every forwarding method is
//! `#[inline]`, and the policy markers are zero-sized. A chosen stack is one concrete
//! type, so the optimiser can fuse the whole stack into a single `create` / `update` /
//! `delete` / `each` with no per-layer call.
//!
//! # Building a stack
//!
//! [`DecoratorsExt`] takes the store by value and returns the concrete composed type:
//!
//! ```rust
//! use doublets::{decorators::{DecoratorsExt, Resolve}, mem, unit, Doublets};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut store = unit::Store::<usize, _>::new(mem::Global::new())?
//!     .with_uniqueness(Resolve)
//!     .with_usages_validation();
//!
//! let a = store.create_point()?;
//! let b = store.create_point()?;
//! assert_eq!(store.create_link(a, b)?, store.create_link(a, b)?);
//! # Ok(())
//! # }
//! ```
//!
//! # The decorators
//!
//! | C# decorator | This module | Create | Update | Delete | Each |
//! |---|---|:--:|:--:|:--:|:--:|
//! | `LinksUniquenessValidator` | [`UniquenessValidator`] | | ● | | |
//! | `LinksUniquenessResolver` | [`UniquenessResolver`] | | ● | | |
//! | `LinksCascadeUniquenessAndUsagesResolver` | [`CascadeUniquenessAndUsagesResolver`] | | ● | ● | |
//! | `LinksUsagesValidator` | [`UsagesValidator`] | | ● | ● | |
//! | `LinksCascadeUsagesResolver` | [`CascadeUsagesResolver`] | | | ● | |
//! | `LinksInnerReferenceExistenceValidator` | [`InnerReferenceExistenceValidator`] | | ● | ● | ● |
//! | `LinksItselfConstantToSelfReferenceResolver` | [`ItselfConstantToSelfReferenceResolver`] | | ● | | ● |
//! | `LinksNullConstantToSelfReferenceResolver` | [`NullConstantToSelfReferenceResolver`] | ● | ● | | |
//! | `LinksNonExistentDependenciesCreator` | [`NonExistentDependenciesCreator`] | | ● | | |
//! | `NonNullContentsLinkDeletionResolver` | [`NonNullContentsLinkDeletionResolver`] | | | ● | |
//! | `LoggingDecorator` | [`LoggingDecorator`] | ● | ● | ● | |
//! | `NoExceptionsDecorator` | [`NoExceptionsDecorator`] | ● | ● | ● | ● |
//!
//! # Ordering
//!
//! C# propagates a `_facade` reference down the stack so that a decorator's recursive
//! calls — a cascading delete, a usage merge — re-enter at the *top* of the stack. A
//! statically composed stack has no such back-reference: a decorator only knows the
//! layers below it, so recursive calls re-enter at the layer that made them.
//!
//! For a stack whose outermost layer is the one doing the recursion the two are the same,
//! which is the case for every stack C# itself builds. Otherwise, put a decorator whose
//! behaviour the cascade must observe *below* the decorator that cascades.
//!
//! # Deviations from C\#
//!
//! Each is documented on the item it affects; in summary:
//!
//! - [`UniquenessResolver`] reports the surviving link to the handler, so
//!   [`create_link`](crate::Doublets::create_link) returns a valid address instead of the
//!   address of the link it just deleted.
//! - [`CascadeUsagesResolver`] tracks the links it has already visited, so a reference
//!   cycle terminates instead of overflowing the stack.
//! - `MergeUsages` re-points sources and targets correctly; the C# version writes null
//!   targets because of a `params` constructor mix-up.
//! - `EnsureCreated` actually calls the creator in its loop; the C# loop never terminates.
//! - [`InnerReferenceExistenceValidator::try_each_links`] exists because
//!   [`Links::each_links`](crate::Links::each_links) has no error channel.
//! - [`NoExceptionsDecorator`] maps `Err(_)` to [`Flow::Break`](data::Flow::Break), the
//!   closest thing Rust's [`Flow`](data::Flow) has to C#'s `Error` constant. It does not
//!   catch panics.
//! - [`LoggingDecorator`] surfaces the first I/O error from the log writer instead of
//!   discarding it.

mod macros;

mod builder;
mod constants;
mod deletion;
mod existence;
mod helpers;
mod logging;
mod no_exceptions;
mod policy;
mod uniqueness;
mod usages;

pub use builder::{AutomaticUniquenessAndUsagesResolution, DecoratorsExt};
pub use constants::{ItselfConstantToSelfReferenceResolver, NullConstantToSelfReferenceResolver};
pub use deletion::NonNullContentsLinkDeletionResolver;
pub use existence::{InnerReferenceExistenceValidator, NonExistentDependenciesCreator};
pub use logging::LoggingDecorator;
pub use no_exceptions::NoExceptionsDecorator;
pub use policy::{CascadeResolve, Resolve, UniquenessPolicy, UsagesPolicy, Validate};
pub use uniqueness::{CascadeUniquenessAndUsagesResolver, UniquenessResolver, UniquenessValidator};
pub use usages::{CascadeUsagesResolver, UsagesValidator};
