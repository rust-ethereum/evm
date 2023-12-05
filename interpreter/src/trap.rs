pub trait TrapConstruct<T> {
	fn construct(v: T) -> Self;
}

pub trait TrapConsume<T> {
	type Rest;

	fn consume(self) -> Result<T, Self::Rest>;
}
