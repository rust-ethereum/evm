//! A module containing the `MaybeBorrowed` enum. See its documentation for details.

/// Similar to `Cow` from the standard library, but without requiring `T: Clone`.
/// Instead of "copy on write", this data structure represents a type that can create
/// `&mut T`, either because it is `&mut T`, or because it is an owned `T`.
/// This is also distinct from the `BorrowMut` trait in the standard library because
/// you can have a single collection mix both borrowed and owned data (e.g.
/// `let xs: Vec<MaybeBorrowed<'_, T>> = vec![&mut t1, t2]` would be possible whereas
/// `Vec<B> where B: BorrowMut<T>` would need to consist of all owned or all borrowed data).
#[derive(Debug)]
pub enum MaybeBorrowed<'a, T> {
	Borrowed(&'a mut T),
	Owned(T),
}

impl<'a, T> core::ops::Deref for MaybeBorrowed<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Borrowed(x) => x,
			Self::Owned(x) => x,
		}
	}
}

impl<'a, T> core::ops::DerefMut for MaybeBorrowed<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Self::Borrowed(x) => x,
			Self::Owned(x) => x,
		}
	}
}
