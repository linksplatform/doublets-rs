use doublets::{
  data::LinkType, mem, mem::RawMem, unit, unit::LinkPart, Doublets, DoubletsExt, Error, Link, Links,
};
use itertools::Itertools;
use std::{collections::HashSet, fmt::Write};
use tap::Pipe;
use std::io::Read;

#[rustfmt::skip]
const CATALAN_NUMBERS: [u64; 25] = [
  1, 1, 2, 5, 14, 42, 132, 429, 1430, 4862, 16796, 58786, 208012,
  742900, 2674440, 9694845, 35357670, 129644790,  477638700, 1767263190,
  6564120420, 24466267020, 91482563640, 343059613650, 1289904147324,
];

const fn catalan(n: usize) -> u64 {
  CATALAN_NUMBERS[n]
}

fn spec_all_variants<T, S>(store: &mut S, seq: &[T]) -> Result<Vec<T>, Error<T>>
where
  T: LinkType,
  S: Doublets<T>,
{
  assert!(seq.len() > 2);

  let mut variants = Vec::with_capacity(catalan(seq.len() - 1) as usize);
  for splitter in 1..seq.len() {
    let (left, right) = seq.split_at(splitter);
    let (left, right) = (
      all_seq_variants(store, left)?,
      all_seq_variants(store, right)?,
    );
    for from in left {
      for &to in &right {
        variants.push(store.get_or_create(from, to)?);
      }
    }
  }
  Ok(variants)
}

fn all_seq_variants<T, S>(store: &mut S, seq: &[T]) -> Result<Vec<T>, Error<T>>
where
  T: LinkType,
  S: Doublets<T>,
{
  match seq {
    &[single] => {
      vec![single]
    }
    &[from, to] => {
      vec![store.get_or_create(from, to)?]
    }
    seq => spec_all_variants(store, seq)?,
  }
  .pipe(Ok)
}

/// Performs a NAND operation on two boolean inputs.
///
/// # Arguments
///
/// * `a` - A boolean input.
/// * `b` - A boolean input.
///
/// # Returns
///
/// * A boolean output representing the NAND operation on the inputs.
fn nand(a: bool, b: bool) -> bool {
  !(a && b)
}

// Fixed function signature: replaced `_` with generic parameter `T` and corrected the return type
fn get_link_by_id<T>(
  store: &mut unit::Store<usize, T>,
  id: usize,
) -> Result<Link<usize>, Error<usize>>
where
  T: RawMem<LinkPart<usize>>,
{
  // `any` constant denotes any link
  let any = store.constants().any;
  let mut link_result = Err(Error::NotExists(id));

  store.each_iter([id, any, any]).for_each(|link| {
    if link.index == id {
      link_result = Ok(link);
    }
  });

  link_result
}

pub fn deep_format<T, S>(
  store: &mut S,
  link_index: T,
  is_element: impl Fn(&Link<T>) -> bool,
  render_visited: bool,
  render_index: bool,
  render_debug: bool,
) -> Result<String, Error<T>>
where
  T: LinkType,
  S: Doublets<T>,
{
  let mut sb = String::new();
  let mut visited = HashSet::new();
  append_structure(
    store,
    &mut sb,
    &mut visited,
    link_index,
    &is_element,
    &append_index,
    render_visited,
    render_index,
    render_debug,
  )?;
  Ok(sb)
}

fn append_structure<T, S>(
  store: &mut S,
  sb: &mut String,
  visited: &mut HashSet<T>,
  link_index: T,
  is_element: &impl Fn(&Link<T>) -> bool,
  append_index: &impl Fn(&mut String, T, bool, bool, bool),
  render_visited: bool,
  render_index: bool,
  render_debug: bool,
) -> Result<(), Error<T>>
where
  T: LinkType,
  S: Doublets<T>,
{
  let constants = store.constants();
  if [constants.null, constants.any, constants.itself].contains(&link_index) {
    return Ok(());
  }

  let mut is_missing = !store.exist(link_index);
  let is_visited = !visited.insert(link_index);

  // Skip fetching the link if it's missing or visited
  if is_missing || (is_visited && !render_visited) {
    append_index(sb, link_index, is_missing, is_visited, render_debug);
    return Ok(());
  }

  // Call get_link to check if the link exists
  let link = store.get_link(link_index);
  is_missing = link.is_none();

  if is_missing {
    append_index(sb, link_index, is_missing, is_visited, render_debug);
    return Ok(());
  }

  let link = link.unwrap();

  // Check if the link is an element after unwrapping
  if is_element(&link) {
    append_index(sb, link_index, is_missing, is_visited, render_debug);
    return Ok(());
  }

  // Open the structure with '('
  sb.push('(');

  // Render index if required
  if render_index {
    append_index(sb, link_index, is_missing, is_visited, render_debug);
    sb.push(':');
    sb.push(' ');
  }

  // Recur for source and target
  append_structure(
    store,
    sb,
    visited,
    link.source,
    is_element,
    append_index,
    render_visited,
    render_index,
    render_debug,
  )?;
  sb.push(' ');
  append_structure(
    store,
    sb,
    visited,
    link.target,
    is_element,
    append_index,
    render_visited,
    render_index,
    render_debug,
  )?;

  // Close the structure with ')'
  sb.push(')');

  Ok(())
}

fn append_index<T>(
  sb: &mut String,
  index: T,
  is_missing: bool,
  is_visited: bool,
  render_debug: bool,
) where
  T: LinkType,
{
  if render_debug {
    if is_missing {
      sb.push('~');
    } else if is_visited {
      sb.push('*');
    }
  }

  // Always render the index at the end
  write!(sb, "{}", index).unwrap();
}

fn apply_nand_to_structure<T, S>(
  store: &mut S,
  link_index: T,
  x_placeholder_link: T,
  y_placeholder_link: T,
  x_value: bool,
  y_value: bool,
) -> Result<bool, Error<T>>
where
  T: LinkType,
  S: Doublets<T>,
{
  // Fetch the link (assume it's guaranteed to exist)
  let link = store
    .get_link(link_index)
    .ok_or(Error::NotExists(link_index))?;

  if link.index == x_placeholder_link {
    return Ok(x_value);
  } else if link.index == y_placeholder_link {
    return Ok(y_value);
  } else {
  }

  // If the link source is the x or y placeholder, substitute the values
  let lhs = if link.source == x_placeholder_link {
    x_value
  } else if link.source == y_placeholder_link {
    y_value
  } else {
    // Recursively apply NAND on the source link
    apply_nand_to_structure(
      store,
      link.source,
      x_placeholder_link,
      y_placeholder_link,
      x_value,
      y_value,
    )?
  };

  // If the link target is the x or y placeholder, substitute the values
  let rhs = if link.target == x_placeholder_link {
    x_value
  } else if link.target == y_placeholder_link {
    y_value
  } else {
    // Recursively apply NAND on the target link
    apply_nand_to_structure(
      store,
      link.target,
      x_placeholder_link,
      y_placeholder_link,
      x_value,
      y_value,
    )?
  };

  // Return the result of the NAND operation on the left-hand side and right-hand side
  Ok(nand(lhs, rhs))
}

fn main() -> Result<(), Error<usize>> {
  let mem = mem::Global::new();
  // Fixed: replaced `_` placeholder with explicit type parameter
  let mut store = unit::Store::<usize, mem::Global<unit::LinkPart<usize>>>::new(mem)?;

  let link_type = store.create_point()?;

  let x = store.create_point()?;
  store.update(x, x, link_type)?;
  let y = store.create_point()?;
  store.update(y, y, link_type)?;

  // Define the two links
  let args = vec![x, y];

  // Specify the length of the sequences you want (e.g., 1 to 16)
  let max_seq_length = 8; // Change this as needed

  // Generate all possible sequences of `1` and `2` with the specified length
  let sequences: Vec<Vec<usize>> = (1..=max_seq_length)
  .flat_map(|length| {
      let pools = vec![args.iter().cloned(); length];
      pools.into_iter().multi_cartesian_product()
  })
  .collect();

  println!("Total sequences: {}", sequences.len());
  for seq in &sequences {
    let mut seq_string = format!("{:?}", seq);
    seq_string = seq_string.replace(&x.to_string(), "x");
    seq_string = seq_string.replace(&y.to_string(), "y");
    println!("{}", seq_string);
  }

  // ask user to continue
  println!("Press any key to continue...");
  let _ = std::io::stdin().read(&mut [0u8]).unwrap();

  // Use the generated sequences to create variants
  for seq in sequences {
    let result = all_seq_variants(&mut store, &seq)?;

    println!("Total variants: {}", result.len());
    for variant in &result {
      let mut deep_structure = deep_format(
        &mut store,
        *variant,
        |link| link.is_partial(),
        true,
        false,
        false,
      )?;
      deep_structure = deep_structure.replace(&x.to_string(), "x");
      deep_structure = deep_structure.replace(&y.to_string(), "y");
      println!("({variant}: {deep_structure})");

      let base_expression: String = deep_structure.replace(" ", " ↑ ");

      println!("expression: {base_expression}");

      // Define all possible combinations of x_value and y_value
      let combinations = [
        (false, false),
        (false, true),
        (true, false),
        (true, true),
      ];

      let mut result_vec = vec![];

      // Loop through each combination
      for &(x_value, y_value) in &combinations {
        // Compute the final NAND result by traversing the entire expression tree for the current combination
        let nand_result = apply_nand_to_structure(
            &mut store,
            *variant,
            x,       // x_placeholder_link
            y,       // y_placeholder_link
            x_value, // current x_value
            y_value, // current y_value
        )?;

        // Replace placeholders in the expression with the current x_value and y_value
        let mut expression = base_expression.replace("x", &x_value.to_string());
        expression = expression.replace("y", &y_value.to_string());

        // Print the final expression and the result of NAND evaluation
        println!("{expression} = {nand_result}");

        // Store the result in a vector
        result_vec.push(nand_result);
      }

      // print expression and result_vec
      println!("{base_expression} = {result_vec:?}");

      println!();
    }
  }

  // `any` constant denotes any link
  let any = store.constants().any;

  println!("Total links: {}", store.count());
  store.each_iter([any, any, any]).for_each(|link| {
    println!("{link:?}");
  });

  // println!("Check for full points:");
  // // Iterate over all links and check if they are full points
  // store.each_iter([any, any, any]).for_each(|link| {
  //   println!("{:?} is a full point: {}", link, link.is_full());
  // });

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_nand() {

    assert_eq!(nand(false, false), true);
    assert_eq!(nand(false, true), true);
    assert_eq!(nand(true, false), true);
    assert_eq!(nand(true, true), false);
  }
}