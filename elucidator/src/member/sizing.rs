/// Represent array sizing for a Member.
/// Generally not useful except when constructing Members for users, though it is used in this
/// library.
/// ```
/// use elucidator::member::Sizing;
///
/// // Fixed Sizing of 10
/// let fixed_size = Sizing::Fixed(10 as u64);
/// // Dynamic Sizing based on the identifier "len"
/// let dynamic_size = Sizing::Dynamic;
/// assert_eq!(fixed_size, Sizing::Fixed(10));
/// assert_eq!(dynamic_size, Sizing::Dynamic);
/// ```
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Sizing {
    Singleton,
    Fixed(u64),
    Dynamic,
}
