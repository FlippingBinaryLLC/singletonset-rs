use singletonset::SingletonSet;

fn main() {
    let mut set = SingletonSet::new();

    // The object is initially empty.
    assert!(set.is_empty());

    // A `SingletonSet` holds automatically calls `Default::default()` on
    // any type that implements it when an instance of the type is accessed
    // for the first time using a `_mut` method.
    assert_eq!(set.as_mut() as &u8, &0);

    // After that, the set holds an instance of that type.
    assert_eq!(set.len(), 1);

    // The instance of each type can be replaced or mutated as long as the
    // set is mutable.
    *set.as_mut() = 10u8;
    *set.as_mut() = 5u8;
    *set.as_mut() = 2u8;

    // At this point, the set still contains only one element, which is the
    // last one (2u8), because the previous values were replaced each time.
    // It can be retrieved with `as_ref()` if the desired value's type is
    // supplied with type coercion.
    assert_eq!(set.as_ref() as &u8, &2);

    // Each element in the set can be manipulated in place with either
    // `get_mut()` or `as_mut()`, but `get_mut()` can take a generic
    // parameter, which can be useful in some cases.
    *set.get_mut::<u8>() *= 2;
    *(set.get_mut() as &mut u8) *= 2;
    *(set.as_mut() as &mut u8) *= 2;

    assert_eq!(set.as_ref() as &u8, &16);

    // Alternatively, a default value can be specified, which the set uses to
    // infer the type.
    *set.get_or_insert_mut(14u8) *= 2;

    // The type is inferred from `14u8`, but its value was not used because
    // the slot in the set was not empty.
    assert_eq!(set.as_ref() as &u8, &32);

    // There isn't any u16 value in the set yet, so this call to
    // `as_mut_or_insert` first inserts `35` into the `u16` slot of the set,
    // then retrieves it so it can be multiplied by `2`.
    *set.as_mut_or_insert(35u16) *= 2;

    // This will print the `u16` value in the set, which is 35 * 2 = 70
    assert_eq!(set.as_ref() as &u16, &70);

    // More complex types can also be held in the set
    set.insert("Foo".to_string());

    // These more complex types can also be manipulated in place with a
    // generic type parameter.
    set.get_mut::<String>().push_str(", bar");

    // ... or type coercion
    (set.as_mut() as &mut String).push_str(", baz");

    // ... or scope the local variable using one of the `with_` methods.
    set.with_ref(|msg: &String| assert_eq!(msg, &"Foo, bar, baz".to_string()));

    // Notice the set now contains 3 elements. One is in the `u8` slot, one
    // in the `u16` slot, and one in the `String` slot.
    assert_eq!(set.len(), 3);

    // If you want to get a value from the set without causing one to be
    // created, use one of the `try_` methods.
    assert_eq!(set.try_get::<u8>(), Some(&32));

    // To simply test if a type is represented in the set, use one of the
    // `contains` methods.
    assert!(!set.contains::<f64>());

    // If you need to store objects of types that do not implement `Default`,
    // like this one, some methods won't be available.
    #[derive(Debug, PartialEq)]
    struct Foo(String);

    // One method uses two closures. This method calls the default closure
    // only if the value doesn't already exist in the set. It always calls
    // the other closure. That closure's return value gets passed through.
    let ret = set.with_mut_or_else(
        || Foo("Default".to_string()),
        |val| {
            *val = Foo("Wild!".to_string());
            42
        },
    );
    assert_eq!(ret, 42);

    // ... alternatively
    set.with_mut_or_else(
        || Foo("Default".to_string()),
        |val| *val = Foo("Wilder!".to_string()),
    );

    // Now the set holds a `Foo` containing the string `Wilder!`.
    assert_eq!(set.as_ref() as &Foo, &Foo("Wilder!".to_string()));

    // Finally, the types represented by the set can be iterated.
    for t in set.types() {
        match t.as_name() {
            "u8" => {
                println!("Set holds a u8: {}", set.as_ref() as &u8);
            }
            "u16" => {
                println!("Set holds a u16: {}", set.as_ref() as &u16);
            }
            "u32" => {
                println!("Set holds a u32: {}", set.as_ref() as &u32);
            }
            "str" => {
                println!("Set holds a &str: '{}'", set.as_ref() as &&str);
            }
            "String" => {
                println!("Set holds a String: '{}'", set.as_ref() as &String);
            }
            _ => {
                println!("Set holds another type of value, a {}", t);
            }
        }
    }
}
