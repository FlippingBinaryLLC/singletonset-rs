#![doc = include_str!("../README.md")]

use std::{
    any::{Any, TypeId},
    fmt::{Display, Formatter},
    hash::Hasher,
};

use hashbrown::{HashMap, TryReserveError};

/// A hash map that uses the value's type as its key.
///
/// This data structure can be used to create a locally-scoped Singleton out
/// of any data type it holds. It ensures there is only one instance of any
/// type, similar to a Singleton, without requiring a global scope.
#[derive(Debug, Default)]
pub struct SingletonSet(HashMap<Type, Box<dyn Any>>);

impl SingletonSet {
    /// Creates an empty `SingletonSet`.
    ///
    /// The set is initially created with a capacity of 0, so it will not
    /// allocate until an element is inserted. This behavior is inherited
    /// from the internal `HashMap` that is used to stored the elements.
    ///
    /// # Example
    ///
    /// ```
    /// use singletonset::SingletonSet;
    /// let mut set = SingletonSet::new();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        SingletonSet(HashMap::new())
    }

    /// Creates an empty `SingletonSet` with at least the specified capacity.
    ///
    /// The set will be able to hold at least `capacity` elements without
    /// reallocating. The hash map that stores the elements internally does
    /// not provide any guarantee that more space won't be allocated.
    ///
    /// # Example
    ///
    /// ```
    /// use singletonset::SingletonSet;
    /// let mut set: SingletonSet = SingletonSet::with_capacity(10);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        SingletonSet(HashMap::with_capacity(capacity))
    }

    /// Returns the number of elements the set can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Returns the number of elements the set currently holds.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the set contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Clears the set, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Reserves capacity for at least `additional` more values.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Tries to reserve capacity for at least `additional` more values.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve(additional)
    }

    /// Shrinks the capacity of the set as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Shrinks the capacity of the set as much as possible, but not less than
    /// `min_capacity`.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity)
    }

    /// Returns an immutable reference to the value of the specified type.
    ///
    /// This method does not insert an element into the set, so it can be
    /// used with types that do not implement [`Default`] and does not need
    /// the `SimpletonSet` to be mutable.
    ///
    /// # Safety
    ///
    /// This method panics if there is no existing value for the given type.
    /// If this is not acceptable, use [`SingletonSet::try_get()`],
    /// [`SingletonSet::get_mut()`], or one of the `get_or` methods.
    pub fn get<T>(&self) -> &T
    where
        T: Any,
    {
        self.as_ref()
    }

    /// Returns an immutable reference to the value of the specified type,
    /// if it exists.
    ///
    /// This method does not insert an element into the set, so it can be
    /// used with types that do not implement [`Default`] and does not need
    /// the set to be mutable.
    #[doc(alias = "try_get()")]
    pub fn try_as_ref<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        self.0
            .get(&Type::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// This is an alias for [`Self::try_as_ref()`]
    pub fn try_get<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.try_as_ref()
    }

    pub fn get_mut<T>(&mut self) -> &mut T
    where
        T: Any + Default,
    {
        self.as_mut()
    }

    /// Returns a mutable reference to the value of the specified type,
    /// if it exists.
    ///
    /// This method does not insert an element into the set, so it can be
    /// used with types that do not implement [`Default`].
    pub fn try_get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Any,
    {
        self.0
            .get_mut(&Type::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Returns an immutable reference to the value of the specified type,
    /// inserting the provided value if the type isn't already in the set.
    ///
    /// If the type is already represented in the set, the provided value
    /// is ignored.
    pub fn get_or_insert<T>(&mut self, value: T) -> &T
    where
        T: Any,
    {
        self.0
            .entry(Type::of::<T>())
            .or_insert(Box::new(value))
            .downcast_ref::<T>()
            // Safety: It should be safe to downcast a value back to its
            // original type, so this `unwrap()` will never panic.
            .unwrap()
    }

    /// Returns a mutable reference to the value of the specified type,
    /// inserting the provided value if the type isn't already in the set.
    ///
    /// If the type is already represented in the set, the provided value
    /// is ignored. To avoid allocating memory for a default value that is
    /// discarded, use [`SingletonSet::get_or_insert_with_mut()`].
    pub fn get_or_insert_mut<T>(&mut self, value: T) -> &mut T
    where
        T: Any,
    {
        self.0
            .entry(Type::of::<T>())
            .or_insert(Box::new(value))
            .downcast_mut::<T>()
            // Safety: The key exists and the type must be correct
            .unwrap()
    }

    /// Returns an immutable reference to the value of the specified type,
    /// inserting the return value of the provided method if the type isn't
    /// already in the set.
    pub fn get_or_insert_with<F, T>(&mut self, default: F) -> &T
    where
        F: FnOnce() -> T,
        T: Any,
    {
        self.0
            .entry(Type::of::<T>())
            .or_insert_with(|| Box::new(default()))
            .downcast_ref::<T>()
            // Safety: The key exists and the type must be correct
            .unwrap()
    }

    /// Returns a mutable reference to the value of the specified type,
    /// inserting the return value of the provided method if the type isn't
    /// already in the set.
    #[doc(alias = "get_or_insert_with_mut()")]
    pub fn as_mut_or_insert_with<T>(&mut self, default: impl FnOnce() -> T) -> &mut T
    where
        T: Any,
    {
        self.0
            .entry(Type::of::<T>())
            .or_insert_with(|| Box::new(default()))
            .downcast_mut::<T>()
            // Safety: The key exists and the type must be correct
            .unwrap()
    }

    /// This is an alias for [`Self::as_mut_or_insert_with()`]
    pub fn get_or_insert_with_mut<T>(&mut self, default: impl FnOnce() -> T) -> &mut T
    where
        T: 'static,
    {
        self.as_mut_or_insert_with(default)
    }

    /// Returns an iterator that visits each [`Type`] in the set in random order.
    ///
    /// The random order is inherited from the internal hash map used to
    /// store the elements, but may change in the future.
    pub fn types(&self) -> Types<'_> {
        Types(self.0.keys())
    }
}

impl<T> AsRef<T> for SingletonSet
where
    T: 'static,
{
    /// Returns an immutable reference to the value of the inferred type.
    ///
    /// This method does not insert an element into the set, so it can be
    /// used with types that do not implement [`Default`] and does not need
    /// the `SimpletonSet` to be mutable.
    ///
    /// # Safety
    ///
    /// This method panics if there is no existing value for the given type.
    /// If this is not acceptable, use [`Self::try_with_ref()`],
    /// [`Self::try_as_ref()`], or one of the `get_or` methods.
    #[doc(alias = "get_mut()")]
    fn as_ref(&self) -> &T {
        self.try_as_ref()
            .expect("try_as_ref() or as_mut() should be used if the slot might be empty")
    }
}

impl<T> AsMut<T> for SingletonSet
where
    T: 'static + Default,
{
    /// Returns a mutable reference to the value of the specified type.
    ///
    /// This method inserts an element into the set if the type is not
    /// already represented, so the type must implement [`Default`].
    #[doc(alias = "get()")]
    fn as_mut(&mut self) -> &mut T {
        self.as_mut_or_insert_with(|| T::default())
    }
}

/// An iterator of the [`Type`]s in a [`SingletonSet`].
pub struct Types<'a>(hashbrown::hash_map::Keys<'a, Type, Box<dyn Any>>);

impl<'a> Iterator for Types<'a> {
    type Item = &'a Type;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// A `Type` represents a globally unique identifier for a type.
///
/// The properties of each `Type` come from the compiler, which are currently
/// only available for types with a static lifetime.
///
/// For a string representation of the type, there are two options. The full
/// name according to the compiler can be obtained with [`Type::as_str()`].
/// This may be the full path of the type, such as `"core::option::Option"`,
/// but it comes with no guarantees. A shortened version holding the last
/// segment of the type name can be obtained by calling [`Type::as_name()`].
///
/// The [`TypeId`] can be obtained by calling [`Type::to_id()`]
#[derive(Clone, Copy, Debug, Eq)]
pub struct Type(TypeId, &'static str);

impl Type {
    /// Creates a new `Type`
    pub fn of<T>() -> Self
    where
        T: 'static,
    {
        Type(TypeId::of::<T>(), std::any::type_name::<T>())
    }

    /// Returns a [`TypeId`] representing the type uniquely among all other
    /// types available to the compiler.
    pub fn to_id(&self) -> TypeId {
        self.0
    }

    /// Returns a name of the type as a string, as reported by the compiler.
    ///
    /// Type names are not unique, and there may be multiple type names that
    /// all refer to the same type.
    pub fn as_str(&self) -> &str {
        self.1
    }

    /// Returns a short name of the type as a string.
    ///
    /// The short type name is not guaranteed to be consistent across
    /// multiple builds, or unique among available types.
    pub fn as_name(&self) -> &str {
        let to_index = self.1.find('<').unwrap_or(self.1.len());

        let from_index = self.1[..to_index].rfind(':').map_or(0, |i| i + 1);

        &self.1[from_index..to_index]
    }
}

impl AsRef<str> for Type {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<TypeId> for Type {
    fn as_ref(&self) -> &TypeId {
        &self.0
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::hash::Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // The TypeId is guaranteed to be unique, so that's all that should
        // be hashed. The name has weaker guarantees and comes from the same
        // compiler at the same time.
        self.0.hash(state)
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        // The TypeId is guaranteed to be unique, so that's all that should
        // be hashed. The name has weaker guarantees and comes from the same
        // compiler at the same time.
        self.0 == other.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singletonset_retains_last_element_of_type() {
        let mut set = SingletonSet::new();

        *set.get_mut() = 1u8;
        *set.get_mut() = 2u8;
        *set.get_mut() = 3u8;
        *set.get_mut() = 1u16;
        *set.get_mut() = 2u16;
        *set.get_mut() = 3u32;
        *set.get_mut() = 2u32;
        *set.get_mut() = "foo";
        *set.get_mut() = "bar";
        *set.get_mut() = "baz".to_string();

        assert_eq!(set.len(), 5);
        assert_ne!(set.get::<u8>(), &1u8);
        assert_ne!(set.get::<u8>(), &1u8);
        assert_eq!(set.get::<u8>(), &3u8);
        assert_ne!(set.get::<u16>(), &1u16);
        assert_eq!(set.get::<u16>(), &2u16);
        assert_ne!(set.get::<u32>(), &3u32);
        assert_eq!(set.get::<u32>(), &2u32);
        assert_ne!(set.get::<&str>(), &"foo");
        assert_eq!(set.get::<&str>(), &"bar");
        assert_eq!(set.get::<String>(), &"baz".to_string());
    }

    #[test]
    fn test_singletonset_mutations() {
        let mut set = SingletonSet::new();

        *set.get_mut() = "foo".to_string();
        (*set.get_mut::<String>()).push_str("bar");

        *set.get_mut::<u8>() += 2;
        *set.get_mut::<u8>() *= 2;

        // The "Hello, World!" string should be gone, replaced by the longer
        // one, which can be retrieved by accessing the `&str` element.
        assert_ne!(set.get::<String>(), &"foo".to_string());
        assert_eq!(set.get::<String>(), &"foobar".to_string());
        assert_ne!(set.get::<u8>(), &2);
        assert_eq!(set.get::<u8>(), &4);
    }

    #[test]
    fn singletonset_works_without_default() {
        let mut set = SingletonSet::new();

        #[derive(Debug, PartialEq)]
        struct Foo(&'static str);

        assert_eq!(set.try_get::<Foo>(), None);

        set.get_or_insert(Foo("bar"));

        assert_eq!(set.try_get::<Foo>(), Some(&Foo("bar")));
    }

    #[test]
    fn singletonset_works_with_custom_defaults() {
        let mut set = SingletonSet::new();

        #[derive(Debug, PartialEq)]
        struct Foo(&'static str);

        impl Default for Foo {
            fn default() -> Self {
                Self("foo")
            }
        }

        set.get_mut::<Foo>();

        assert_eq!(set.try_get::<Foo>(), Some(&Foo("foo")));

        *set.get_mut() = Foo("bar");

        assert_eq!(set.try_get::<Foo>(), Some(&Foo("bar")));
    }
}
