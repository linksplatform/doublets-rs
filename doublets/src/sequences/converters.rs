//! Balanced variants, C# hybrid raw numbers and reusable conversion caches.

use std::{collections::HashMap, hash::Hash};

use data::LinkReference;

use super::SequenceError;
use crate::{Doublets, Error};

/// Encodes an unsigned magnitude as a C# hybrid external reference.
#[derive(Clone, Copy, Debug, Default)]
pub struct AddressToRawNumberConverter;

impl AddressToRawNumberConverter {
    /// Creates the stateless converter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Encodes zero as `MAX / 2 + 1`, or a positive value as `MAX - value + 1`.
    /// Returns an error when `address > MAX / 2` would alias an internal address.
    pub fn convert<T: LinkReference>(&self, address: T) -> Result<T, SequenceError<T>> {
        let half = T::MAX / T::from_byte(2);
        if address > half {
            return Err(SequenceError::RawNumberOutOfRange(address));
        }
        Ok(if address == T::from_byte(0) {
            half + T::from_byte(1)
        } else {
            T::MAX - address + T::from_byte(1)
        })
    }
}

/// Decodes the absolute magnitude of a C# hybrid reference.
#[derive(Clone, Copy, Debug, Default)]
pub struct RawNumberToAddressConverter;

impl RawNumberToAddressConverter {
    /// Creates the stateless converter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Decodes external zero and negative encodings; internal addresses pass through.
    pub fn convert<T: LinkReference>(&self, raw_number: T) -> T {
        let external_zero = T::MAX / T::from_byte(2) + T::from_byte(1);
        if raw_number == external_zero {
            T::from_byte(0)
        } else if raw_number > external_zero {
            T::MAX - raw_number + T::from_byte(1)
        } else {
            raw_number
        }
    }
}

/// Builds the same pairwise balanced variant as the C# converter.
#[derive(Clone, Copy, Debug, Default)]
pub struct BalancedVariantConverter;

impl BalancedVariantConverter {
    /// Creates the stateless converter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns null for an empty list, its element for a singleton, or a balanced root.
    /// Adjacent elements are paired from left to right; an odd tail advances unchanged.
    /// Existing pairs are reused and storage failures propagate.
    /// Choose terminal elements that cannot be mistaken for pairs, such as marked
    /// Unicode symbols: a self-point leaf makes `(leaf, leaf)` ambiguous.
    pub fn convert<T: LinkReference, L: Doublets<T>>(
        &self,
        links: &mut L,
        elements: &[T],
    ) -> Result<T, Error<T>> {
        if elements.is_empty() {
            return Ok(links.constants().null);
        }
        let mut layer = elements.to_vec();
        while layer.len() > 1 {
            let length = layer.len();
            for i in (0..length - 1).step_by(2) {
                layer[i / 2] = links.get_or_create(layer[i], layer[i + 1])?;
            }
            if length % 2 == 1 {
                layer[length / 2] = layer[length - 1];
            }
            layer.truncate(length.div_ceil(2));
        }
        Ok(layer[0])
    }
}

/// Memoizes successful conversions. Errors are never cached.
///
/// Supply the wrapped operation to [`Self::convert_with`] so it can borrow a store.
/// Caches belong to one store/marker configuration. Call [`Self::clear`] after
/// deleting or changing links that could invalidate cached results.
#[derive(Clone, Debug)]
pub struct CachingConverterDecorator<K, V> {
    cache: HashMap<K, V>,
}

impl<K, V> Default for CachingConverterDecorator<K, V> {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl<K: Eq + Hash, V: Clone> CachingConverterDecorator<K, V> {
    /// Creates an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a cloned cached result, if present.
    pub fn get(&self, input: &K) -> Option<V> {
        self.cache.get(input).cloned()
    }

    /// Stores a result and returns it.
    pub fn insert(&mut self, input: K, output: V) -> V {
        self.cache.insert(input, output.clone());
        output
    }

    /// Converts only on a cache miss; a failed conversion can be retried.
    pub fn convert_with<F, E>(&mut self, input: K, convert: F) -> Result<V, E>
    where
        F: FnOnce(&K) -> Result<V, E>,
    {
        if let Some(output) = self.get(&input) {
            return Ok(output);
        }
        let output = convert(&input)?;
        Ok(self.insert(input, output))
    }

    /// Invalidates all cached conversions.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Returns the number of cached inputs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns whether no inputs are cached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}
