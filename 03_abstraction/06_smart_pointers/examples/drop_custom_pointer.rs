//! Book 15.3: the `Drop` trait runs code when a value goes away.
//!
//! Run: cargo run -p smart_pointers --example drop_custom_pointer

struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping CustomSmartPointer with data `{}`!", self.data);
    }
}

fn main() {
    // 1. Values are dropped at the end of their scope, in REVERSE order of
    //    creation: `d` was made last, so it goes first.
    {
        let _c = CustomSmartPointer {
            data: String::from("my stuff"),
        };
        let _d = CustomSmartPointer {
            data: String::from("other stuff"),
        };
        println!("CustomSmartPointers created.");
    } // <- d dropped, then c

    // 2. Dropping early. You may NOT call the method yourself:
    //
    //     c.drop();   // error[E0040]: explicit use of destructor method
    //
    //    (Rust would drop it again at the end of the scope: a double free.)
    //    Call std::mem::drop instead. It's in the prelude, so just `drop`.
    let c = CustomSmartPointer {
        data: String::from("some data"),
    };
    println!("CustomSmartPointer created.");
    drop(c); // takes ownership, so c can't be used (or dropped) again
    println!("CustomSmartPointer dropped before the end of main.");

    // 3. A value moved somewhere else is dropped THERE, not here.
    let e = CustomSmartPointer {
        data: String::from("moved into a function"),
    };
    take(e);
    println!("end of main");
}

fn take(_value: CustomSmartPointer) {
    println!("take() is running");
} // <- _value dropped here
