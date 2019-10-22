pub enum MultiError<E> {
	One(E),
	All(E),
}
