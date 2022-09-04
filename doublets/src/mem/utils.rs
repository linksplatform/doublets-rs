pub fn detach_query<T, const N: usize>(query: &[T]) -> [T; N] {
    query[..]
        .try_into()
        .unwrap_or_else(|_| panic!("expected `query` with {N} len"))
}
