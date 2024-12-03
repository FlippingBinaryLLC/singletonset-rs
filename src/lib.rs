#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use std::{
    any::{Any, TypeId},
    fmt::{Display, Formatter},
    hash::Hasher,
};

use indexmap::IndexMap;
pub use indexmap::TryReserveError;

/// A hash map that uses the value's type as its key.
///
/// This data structure can be used to create a locally-scoped Singleton out
/// of any data type it holds. It ensures there is only one instance of any
/// type, similar to a Singleton, without requiring a global scope.
#[derive(Debug, Default)]
pub struct SingletonSet(IndexMap<Type, Box<dyn Any>>);

impl SingletonSet {
    /// Creates an empty `SingletonSet`.
    ///
    /// The set is initially created with a capacity of 0, so it will not
    /// allocate until an element is inserted.
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
        SingletonSet(IndexMap::new())
    }

    /// Creates an empty `SingletonSet` with at least the specified capacity.
    ///
    /// The set will be able to hold at least `capacity` elements without
    /// reallocating. There is no guarantee that more space won't be
    /// allocated.
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
        SingletonSet(IndexMap::with_capacity(capacity))
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

    /// Inserts a value into the inferred type's slot.
    pub fn insert<T>(&mut self, value: T) -> Option<T>
    where
        T: 'static,
    {
        self.0
            .insert(Type::of::<T>(), Box::new(value))
            .and_then(|boxed| boxed.downcast().ok().map(|boxed| *boxed))
    }

    /// Inserts the default value of a type in the set.
    pub fn insert_default<T>(&mut self) -> Option<T>
    where
        T: 'static + Default,
    {
        self.insert(T::default())
    }

    /// Inserts a value into the inferred type's slot.
    pub fn insert_with<T>(&mut self, f: impl FnOnce() -> T) -> Option<T>
    where
        T: 'static,
    {
        self.insert(f())
    }

    /// Returns true if the type is represented in the set.
    pub fn contains<T>(&self) -> bool
    where
        T: 'static,
    {
        self.0.contains_key(&Type::of::<T>())
    }

    /// Returns true if the type of the provided value is represented in the
    /// set.
    pub fn contains_type_of<T>(&self, value: &T) -> bool
    where
        T: 'static,
    {
        let _ = value;
        self.0.contains_key(&Type::of::<T>())
    }

    /// Returns true if the supplied [`Type`] is represented in the set.
    pub fn contains_type(&self, t: &Type) -> bool {
        self.0.contains_key(t)
    }

    /// Calls a closure with some value of the corresponding type's
    /// slot, returning the closure's return value.
    ///
    /// If the slot is empty, the closure is passed [`None`]. This method
    /// will not initialize an empty slot.
    pub fn try_with_ref<T, R>(&self, f: impl FnOnce(Option<&T>) -> R) -> R
    where
        T: 'static,
    {
        f(self.try_as_ref())
    }

    /// Calls a provided closure with the value of the corresponding type's
    /// slot, if it exists, returning its return value.
    ///
    /// # Safety
    ///
    /// This method panics if there is no existing value for the given type.
    /// If this is not acceptable, use [`.try_with_ref()`],
    /// [`.try_as_ref()`], or one of the `get_or` methods.
    ///
    /// [`.try_with_ref()`]: Self::try_with_ref()
    /// [`.try_as_ref()`]: Self::try_as_ref()
    pub fn with_ref<T, R>(&self, f: impl FnOnce(&T) -> R) -> R
    where
        T: 'static,
    {
        f(self.as_ref())
    }

    /// Inserts `default` if its type is not already represented, then calls
    /// `f` with a reference to the value from the set.
    ///
    /// This method also returns the closure's return value.
    pub fn with_ref_or<T, R>(&mut self, default: T, f: impl FnOnce(&T) -> R) -> R
    where
        T: 'static,
    {
        f(self.as_ref_or_insert::<T>(default))
    }

    /// Inserts the default value if its type is not already represented,
    /// then calls `f` with a reference to the value from the set.
    ///
    /// This method also returns the closure's return value.
    pub fn with_ref_or_default<T, R>(&mut self, f: impl FnOnce(&T) -> R) -> R
    where
        T: 'static + Default,
    {
        f(self.as_ref_or_insert::<T>(T::default()))
    }

    /// Applies a function to the immutable reference of a type in the set,
    /// initializing the value first with a default function result (if the
    /// type is not in the set), and returning the closure's return value.
    pub fn with_ref_or_else<T, R>(
        &mut self,
        default: impl FnOnce() -> T,
        f: impl FnOnce(&T) -> R,
    ) -> R
    where
        T: 'static,
    {
        f(self.as_ref_or_insert_with(default))
    }

    /// Calls a provided closure with some mutable reference to the
    /// corresponding type's slot, returning the closure's return value.
    ///
    /// If the slot is empty, the closure is passed [`None`].
    pub fn try_with_mut<T, R>(&mut self, f: impl FnOnce(Option<&mut T>) -> R) -> R
    where
        T: 'static,
    {
        f(self.try_as_mut::<T>())
    }

    /// Calls a provided closure with the value of the corresponding type's
    /// slot, if it exists, returning its return value.
    pub fn with_mut<T, R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R
    where
        T: 'static + Default,
    {
        f(self.as_mut())
    }

    /// Inserts `default` if its type is not already represented, then calls
    /// `f` with a mutable reference to the value from the set.
    ///
    /// This method returns the closure's return value.
    pub fn with_mut_or<T, R>(&mut self, default: T, f: impl FnOnce(&mut T) -> R) -> R
    where
        T: 'static,
    {
        f(self.as_mut_or_insert::<T>(default))
    }

    /// Applies a function to the mutable reference of a type in the set,
    /// initializing the value first with a default function result (if the
    /// type is not in the set), and returning the closure's return value.
    pub fn with_mut_or_else<T, R>(
        &mut self,
        default: impl FnOnce() -> T,
        f: impl FnOnce(&mut T) -> R,
    ) -> R
    where
        T: 'static,
    {
        f(self.as_mut_or_insert_with(default))
    }

    /// This is an alias for [`Self::as_ref()`]
    pub fn get<T>(&self) -> &T
    where
        T: 'static,
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
        T: 'static,
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

    /// This is an alias for [`Self::as_mut()`]
    pub fn get_mut<T>(&mut self) -> &mut T
    where
        T: 'static + Default,
    {
        self.as_mut()
    }

    /// Returns a mutable reference to the value of the specified type,
    /// if it exists.
    ///
    /// This method does not insert an element into the set, so it can be
    /// used with types that do not implement [`Default`].
    #[doc(alias = "try_get_mut()")]
    pub fn try_as_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static,
    {
        self.0
            .get_mut(&Type::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// This is an alias for [`Self::try_as_mut()`]
    pub fn try_get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static,
    {
        self.try_as_mut()
    }

    /// Returns an immutable reference to the value of the specified type,
    /// inserting the provided value if the type isn't already in the set.
    ///
    /// If the type is already represented in the set, the provided value
    /// is ignored.
    #[doc(alias = "get_or_insert()")]
    pub fn as_ref_or_insert<T>(&mut self, value: T) -> &T
    where
        T: 'static,
    {
        self.0
            .entry(Type::of::<T>())
            .or_insert(Box::new(value))
            .downcast_ref::<T>()
            // Safety: It should be safe to downcast a value back to its
            // original type, so this `unwrap()` will never panic.
            .unwrap()
    }

    /// This is an alias for [`Self::as_ref_or_insert()`]
    pub fn get_or_insert<T>(&mut self, value: T) -> &T
    where
        T: 'static,
    {
        self.as_ref_or_insert(value)
    }

    /// Returns a mutable reference to the value of the specified type,
    /// inserting the provided value if the type isn't already in the set.
    ///
    /// If the type is already represented in the set, the provided value
    /// is ignored. To avoid allocating memory for a default value that is
    /// discarded, use [`.get_or_insert_with_mut()`].
    ///
    /// [`.get_or_insert_with_mut()`]: Self::get_or_insert_with_mut()
    #[doc(alias = "get_or_insert_mut()")]
    pub fn as_mut_or_insert<T>(&mut self, value: T) -> &mut T
    where
        T: 'static,
    {
        self.0
            .entry(Type::of::<T>())
            .or_insert(Box::new(value))
            .downcast_mut::<T>()
            // Safety: The key exists and the type must be correct
            .unwrap()
    }

    /// This is an alias for [`.as_mut_or_insert(value)`]
    ///
    /// [`.as_mut_or_insert(value)`]: Self::as_mut_or_insert()
    pub fn get_or_insert_mut<T>(&mut self, value: T) -> &mut T
    where
        T: 'static,
    {
        self.as_mut_or_insert(value)
    }

    /// Returns an immutable reference to the value of the specified type,
    /// inserting the return value of the provided method if the type isn't
    /// already in the set.
    #[doc(alias = "get_or_insert_mut()")]
    pub fn as_ref_or_insert_with<T>(&mut self, default: impl FnOnce() -> T) -> &T
    where
        T: 'static,
    {
        self.0
            .entry(Type::of::<T>())
            .or_insert_with(|| Box::new(default()))
            .downcast_ref::<T>()
            // Safety: The key exists and the type must be correct
            .unwrap()
    }

    /// This is an alias for [`Self::as_ref_or_insert_with()`]
    pub fn get_or_insert_with<T>(&mut self, default: impl FnOnce() -> T) -> &T
    where
        T: 'static,
    {
        self.as_ref_or_insert_with(default)
    }

    /// Returns a mutable reference to the value of the specified type,
    /// inserting the return value of the provided method if the type isn't
    /// already in the set.
    #[doc(alias = "get_or_insert_with_mut()")]
    pub fn as_mut_or_insert_with<T>(&mut self, default: impl FnOnce() -> T) -> &mut T
    where
        T: 'static,
    {
        self.0
            .entry(Type::of::<T>())
            .or_insert_with(|| Box::new(default()))
            .downcast_mut::<T>()
            // Safety: The key exists and the type must be correct
            .unwrap()
    }

    /// This is an alias for [`.as_mut_or_insert_with(default)`]
    ///
    /// [`.as_mut_or_insert_with(default)`]: Self::as_mut_or_insert_with()
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
    /// the set to be mutable.
    ///
    /// # Safety
    ///
    /// This method panics if there is no existing value for the given type.
    /// If this is not acceptable, use methods like [`.try_with_ref()`],
    /// [`.try_as_ref()`], or a `_mut` method.
    ///
    /// [`.try_with_ref()`]: Self::try_with_ref()
    /// [`.try_as_ref()`]: Self::try_as_ref()
    #[doc(alias = "get_mut()")]
    fn as_ref(&self) -> &T {
        self.try_as_ref()
            .expect(".try_as_ref() or .as_mut() should be used if the slot might be empty")
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
    #[doc(alias = "get_mut()")]
    fn as_mut(&mut self) -> &mut T {
        self.as_mut_or_insert_with(|| T::default())
    }
}

/// An iterator of the [`Type`]s in a [`SingletonSet`].
pub struct Types<'a>(indexmap::map::Keys<'a, Type, Box<dyn Any>>);

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
/// name according to the compiler can be obtained with [`.as_str()`].
/// This may be the full path of the type, such as `"core::option::Option"`,
/// but it comes with no guarantees. A shortened version holding the last
/// segment of the type name can be obtained by calling [`.as_name()`],
/// also with no guarantees (arguably fewer guarantees). Currently,
/// `as_name()` returns the string slice within `as_str()` located between
/// the first open angle bracket (`<`) and the nearest colon (`:`) to the
/// left of it.
///
/// [`.as_str()`]: Self::as_str()
/// [`.as_name()`]: Self::as_name()
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
    pub fn as_id(&self) -> &TypeId {
        &self.0
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

    // NOTE: `to_str` is not implemented as a convenience method because the
    // return value would have to be `String`, so the name `to_string` would
    // be more appropriate, but that's already implemented via the `Display`
    // implementation.

    /// Returns a short name of the type as a string.
    ///
    /// The short type name is not guaranteed to be consistent across
    /// multiple builds, or unique among available types.
    pub fn as_name(&self) -> &str {
        let to_index = self.1.find('<').unwrap_or(self.1.len());

        let from_index = self.1[..to_index].rfind(':').map_or(0, |i| i + 1);

        &self.1[from_index..to_index]
    }

    /// Returns a short name of the type as a string.
    ///
    /// The short type name is not guaranteed to be consistent across
    /// multiple builds, or unique among available types.
    pub fn to_name(&self) -> String {
        let to_index = self.1.find('<').unwrap_or(self.1.len());

        let from_index = self.1[..to_index].rfind(':').map_or(0, |i| i + 1);

        self.1[from_index..to_index].to_string()
    }
}

impl AsRef<str> for Type {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<TypeId> for Type {
    fn as_ref(&self) -> &TypeId {
        self.as_id()
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

        *set.as_mut() = 1u8;
        *set.as_mut() = 2u8;
        *set.as_mut() = 3u8;
        *set.as_mut() = 1u16;
        *set.as_mut() = 2u16;
        *set.as_mut() = 3u32;
        *set.as_mut() = 2u32;
        *set.as_mut() = "foo";
        *set.as_mut() = "bar";
        *set.as_mut() = "baz".to_string();

        assert_eq!(set.len(), 5);
        assert_ne!(set.as_ref() as &u8, &1u8);
        assert_ne!(set.get::<u8>(), &1u8);
        assert_eq!(set.as_ref() as &u8, &3u8);
        assert_ne!(set.get::<u16>(), &1u16);
        assert_eq!(set.as_ref() as &u16, &2u16);
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

        set.with_mut(|val: &mut u32| *val += 2);
        set.with_mut::<u32, _>(|val| *val *= 3);
        set.with_mut(|val: &mut String| *val += "baz");

        // The "Hello, World!" string should be gone, replaced by the longer
        // one, which can be retrieved by accessing the `&str` element.
        assert_ne!(set.get::<String>(), &"foo".to_string());
        assert_ne!(set.get::<String>(), &"foobar".to_string());
        assert_eq!(set.get::<String>(), &"foobarbaz".to_string());
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

    #[test]
    fn singletonset_can_be_iterated() {
        let mut set = SingletonSet::new();

        set.get_mut::<u8>();
        set.get_mut::<u16>();
        set.get_mut::<u32>();

        let mut iter = set.types();

        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert_eq!(iter.next(), None);
    }
}
