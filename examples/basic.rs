use singletonset::SingletonSet;

fn main() {
    let mut set = SingletonSet::new();

    // The object is initially empty.
    println!("The set is empty: {}", set.is_empty());

    // A `SingletonSet` holds automatically calls `Default::default()` on
    // any type that implements it when an instance of the type is requested
    // for the first time.
    println!("First mutable access of u8: {}", set.get_mut::<u8>());

    // After that, the set holds an instance of that type.
    println!("The set now contains {} element", set.len());

    // The instance can be replaced or mutated, but only the latest instance
    // is the one stored in the set.
    *set.get_mut() = 10u8;
    *set.get_mut() = 5u8;
    *set.get_mut() = 2u8;

    // The set still contains only one element, which is the last one (2u8)
    // because the previous values were replaced each time.
    println!(
        "The set still contains {} element, which is {}",
        set.len(),
        set.get::<u8>()
    );

    // Each element in the set can be manipulated in place, but the type must
    // be specified as a generic parameter of `get_mut()` in situations when
    // the compiler can't infer it correctly.
    *set.get_mut::<u8>() *= 2;

    println!(
        "Now the {} set element has been updated to {}",
        set.len(),
        set.get::<u8>()
    );

    // Alternatively, a default value can be specified, which provides the
    // type key
    *set.get_or_insert_mut(14u8) *= 2;

    println!("Now the u8 slot holds {}", set.get::<u8>());

    // Now the u8 slot contains 2 * 2 * 2 = 8. The `14u8` is used for its
    // type, but not its value because a value already exists in the slot.
    // If a value hadn't existed in the slot, the default value would have
    // been used.
    *set.get_or_insert_mut(14u16) *= 2;

    // This should print 14 * 2 = 28 because there wasn't any u16 value in
    // the set until `get_or_insert_mut` was called.
    println!("The new u16 slot holds {}", set.get::<u16>());

    // More complex types can also be held in the set

    *set.get_mut() = "A static str";
    *set.get_mut() = "A dynamic String".to_string();

    // These more complex types can also be manipulated in place, but it gets
    // a bit messy.
    (*set.get_mut::<String>()).insert_str(1, " single");

    // The complexity can be reduce a little by using local variables
    let s = set.get_mut::<String>();
    s.push_str(" easily");

    // Notice the set now contains 4 elements. One is in the `u8` slot, one
    // in the `u16` slot, one in the `&'static str` slot, and one in the
    // `String` slot.
    println!(
        "'{}' is in the set of {} total elements",
        set.get::<String>(),
        set.len()
    );

    // You might want to check if a type slot holds a value without actually
    // creating one. For that, use `try_get()`
    if let Some(val) = set.try_get::<u8>() {
        println!("A u8 containing {} was found", val);
    } else {
        println!("This will not print because the set holds a u8");
    }
    if let Some(val) = set.try_get::<f64>() {
        println!(
            "There is no f64 value in the set, so {} will not print.",
            val
        );
    } else {
        println!("There is no f64 value in the set.");
    }

    // If you need to store objects that do not implement `Default`, you must
    // supply an object or closure each time an element is accessed. Both
    // `get()` and `get_mut()` will not work with types that do not implement
    // `Default`.
    struct Foo(&'static str);
    *set.get_or_insert_mut(Foo("default str")) = Foo("Hello, World!");
    println!(
        "The message is: {}",
        set.get_or_insert(Foo("different default str")).0
    );

    // When the default object is not trivial to construct, it may be more
    // efficient to supply a closure instead. The closure is lazily-called,
    // so no memory is allocated if the type is already represented in the
    // `SingletonSet`.
    struct Bar(String);
    *set.get_or_insert_with_mut(|| Bar("default string".to_string())) =
        Bar("Hello, World again!".to_string());
    println!(
        "The message is: {}",
        set.get_or_insert_with(|| Bar("different default string".to_string()))
            .0
    );

    // That gets messy again, but a method can be used instead of a closure.
    fn bar() -> Bar {
        Bar("default string from method".to_string())
    }
    *set.get_or_insert_with_mut(bar) = Bar("Wild!".to_string());

    // Now the set holds a `Bar` containing the string `Wild!`.
    println!("This is {}", set.try_get::<Bar>().unwrap().0);
}
