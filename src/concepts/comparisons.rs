#[allow(unused_variables)]
#[allow(dead_code)]
#[test]
pub fn index() {

    traits::test_partial_eq_manual();
    traits::test_partial_eq_derive();
    traits::test_eq_manual();
    traits::test_eq_derive();
    traits::test_partial_ord_derive();
    traits::test_partial_ord_manual();
    traits::test_ord_derive();
    traits::test_ord_manual();

    primitive_types::test_bool();
    primitive_types::test_char();
    primitive_types::test_integers();
    primitive_types::test_floats();
    primitive_types::test_strings();

    references_vs_pointers::test_references_equal();
    references_vs_pointers::test_pointers_addresses();
    references_vs_pointers::test_pointers_heap();
    references_vs_pointers::test_references_vs_pointers();

    collections::test_arrays();
    collections::test_slices();
    collections::test_vectors();
    collections::test_collections_order();

    compound_types::test_enum_ord();
    compound_types::test_custom_impl();

    tuples::test_tuples_eq();
    tuples::test_tuples_ord();
    tuples::test_nested_tuples();
}

/*

// ═════════════════════════════════════════════════════════════════════════════
// TRAITS:
// ═════════════════════════════════════════════════════════════════════════════
*/

#[cfg(test)]
mod traits {
    /*
    ----------------------------------------------
    PartialEq
    ----------------------------------------------

        pub trait PartialEq<Rhs = Self>
        where
            Rhs: ?Sized,
        {
            fn eq(&self, other: &Rhs) -> bool;

            // Default implementations:
            fn ne(&self, other: &Rhs) -> bool {
                !self.eq(other)
            }
        }


    WHAT IT DOES:
    • Defines == and !=  operators
    • Does NOT require reflexivity (a == a can be false, e.g: NaN) <- important
    • There can be "incomparable" values, e.g NaN
    */

    // Manual implementation of PartialEq:
    #[test]
    pub fn test_partial_eq_manual() {
        #[derive(Debug)]
        struct Age(u8);

        impl PartialEq for Age {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        assert_eq!(Age(30) == Age(30), true);
        assert_eq!(Age(30).eq(&Age(25)), false);
        assert_eq!(Age(30), Age(30));
        assert_ne!(Age(30), Age(25));
        
    }

    // Automatic implementation with derive of PartialEq:
    #[test]
    pub fn test_partial_eq_derive() {
        #[derive(PartialEq, Debug)]
        struct Person {
            name: String,
            age: u8,
        }

        let p1 = Person {
            name: "Alice".into(),
            age: 30,
        };
        let p2 = Person {
            name: "Alice".into(),
            age: 30,
        };
        assert!(p1 == p2); // Compares: name == name AND age == age
    }

    /*
    ----------------------------------------------
    Eq
    ----------------------------------------------

        pub trait Eq: PartialEq<Self> {
            // No additional methods
            // Just marks that PartialEq is reflexive ( a == a is ALWAYS true )
        }

        WHAT IT DOES:
        • Extends PartialEq
        • Guarantees REFLEXIVITY: a == a is ALWAYS true
        • Used for types that do NOT have incomparable values
        • It's a "marker trait" (no methods, just mathematical properties)
    */

    // Manual implementation of Eq:
    #[test]
    pub fn test_eq_manual() {
        #[derive(Debug)]
        struct Point {
            x: i32,
            y: i32,
        }

        impl PartialEq for Point {
            fn eq(&self, other: &Self) -> bool {
                self.x == other.x && self.y == other.y
            }
        }

        impl Eq for Point {}

        let p = Point { x: 5, y: 10 };
        assert_eq!(p, p); // ✓ Reflexivity guaranteed
    }

    // Automatic implementation with derive of Eq:
    #[test]
    pub fn test_eq_derive() {
        #[derive(PartialEq, Eq, Debug)]
        struct UserId(u64);

        let id1 = UserId(123);
        assert_eq!(id1, id1); // Reflexivity: guaranteed by Eq
    }
    /*
    ----------------------------------------------
    PartialOrd
    ----------------------------------------------

        pub trait PartialOrd<Rhs = Self>
        where
            Rhs: ?Sized,   // allows fixed-size or dynamic types (known at runtime)
        {
            fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;

            // Default implementations:
            fn lt(&self, other: &Rhs) -> bool {
                matches!(self.partial_cmp(other), Some(Ordering::Less))
            }
            fn le(&self, other: &Rhs) -> bool {
                matches!(self.partial_cmp(other), Some(Ordering::Less | Ordering::Equal))
            }
            fn gt(&self, other: &Rhs) -> bool {
                matches!(self.partial_cmp(other), Some(Ordering::Greater))
            }
            fn ge(&self, other: &Rhs) -> bool {
                matches!(self.partial_cmp(other), Some(Ordering::Greater | Ordering::Equal))
            }
        }

        WHAT IT DOES:
        • Defines <, <=, >, >= operators
        • Returns Option<Ordering> (can be incomparable, e.g: NaN) <- important
        • REQUIRES implementing PartialEq first

        OPERATORS IT IMPLEMENTS:
        • <, <=, >, >=
        • partial_cmp() → Option<Ordering> (whether comparison succeeded or not)

    */

    // Automatic implementation with derive of PartialOrd:
    #[test]
    pub fn test_partial_ord_derive() {
        #[derive(PartialEq, PartialOrd)]
        struct Score(f64);

        let s1 = Score(85.5);
        let s2 = Score(90.0);
        // comparison operators
        assert_eq!(s1 < s2, true);
        assert_eq!(s1 <= s2, true);
        assert_eq!(s2 > s1, true);
        assert_eq!(s2 >= s1, true);

        // Allows knowing whether comparison is possible or not
        let nan_score = Score(f64::NAN);
        assert_eq!(nan_score < s1, false);
        assert_eq!(nan_score.partial_cmp(&s1), None); // Option<Ordering>
    }

    // Manual implementation of PartialOrd:
    #[test]
    pub fn test_partial_ord_manual() {
        use std::cmp::Ordering;

        struct Distance(f64);

        impl PartialEq for Distance {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl PartialOrd for Distance {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }

        let d1 = Distance(5.0);
        let d2 = Distance(10.0);
        assert_eq!(d1 < d2, true);
        assert_eq!(d1.partial_cmp(&d2), Some(Ordering::Less));
    }
    /*
    ----------------------------------------------
    4. Ord TRAIT
    ----------------------------------------------

        pub trait Ord: Eq + PartialOrd<Self> {
            fn cmp(&self, other: &Self) -> Ordering;
        }

        WHAT IT DOES:
        • Defines "total order": ALL elements are comparable <- important
        • Returns Ordering directly (NOT Option)
        • REQUIRES implementing Eq and PartialOrd first

        OPERATORS IT IMPLEMENTS:
        • <, <=, >, >= (inherited from PartialOrd)
        • cmp() → direct Ordering

    */

    // Automatic implementation with derive of Ord:
    #[test]
    pub fn test_ord_derive() {
        use std::cmp::Ordering;

        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct Priority {
            level: u8,
        }

        let p1 = Priority { level: 1 };
        let p2 = Priority { level: 5 };

        assert_eq!(p1 < p2, true);
        assert_eq!(p1.cmp(&p2), Ordering::Less);

        let mut levels = vec![p2, p1];
        levels.sort();
        assert_eq!(levels[0].level, 1);
    }

    // Manual implementation of Ord:
    #[test]
    pub fn test_ord_manual() {
        use std::cmp::Ordering;

        struct UserId(u64);

        impl PartialEq for UserId {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl Eq for UserId {}

        impl PartialOrd for UserId {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }

        impl Ord for UserId {
            fn cmp(&self, other: &Self) -> Ordering {
                self.0.cmp(&other.0)
            }
        }

        let id1 = UserId(100);
        let id2 = UserId(200);
        assert_eq!(id1.cmp(&id2), Ordering::Less);
    }

    /*

    ----------------------------------------------
    TRAIT HIERARCHY AND REQUIREMENTS
    ----------------------------------------------


                              PartialEq (==, !=) (eq, ne)
                             /          \
                            /            \
                           Eq         PartialOrd (<, <=, >, >=)
                            \            /       (partial_cmp -> Option<Ordering>)
                             \          /
                              \        /
                               \      /
                                 Ord (<, <=, >, >=), (cmp -> Ordering)
    */
}

// ═════════════════════════════════════════════════════════════════════════════
// MODULE 1: COMPARISON OF PRIMITIVE TYPES
// ═════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod primitive_types {
    /*
    COMPARISON OF PRIMITIVE TYPES

    bool    → TRUE/FALSE: equality and ordering (false < true)
    int     → i8, i16, i32, i64, isize: total comparison
    uint    → u8, u16, u32, u64, usize: total comparison
    float   → f32, f64: PartialOrd/PartialEq (NaN breaks Eq)
        NaN != float is always true
        NaN < <= > >= float, is always false
    char    → Compares by Unicode value
        '😀' = U+1F600 = 128512 (decimal)
        'a'  = U+0061  = 97 (decimal)
    String  → &str,  compares as if they were multiple chars, 
        Lexicographic by Unicode code value
        '😀' = U+1F600 = 128512 (decimal)
        'a'  = U+0061  = 97 (decimal)
    */

    #[test]
    pub fn test_bool() {
        assert_eq!(true == true, true);
        assert_eq!(true == false, false);
        assert_eq!(true > false, true); // false = 0, true = 1
        assert_eq!(false < true, true);
    }

    #[test]
    pub fn test_char() {
        assert_eq!('a' == 'a', true);
        assert_eq!('a' != 'b', true);
        assert_eq!('a' < 'b', true); // Compares Unicode code
        assert_eq!('0' < '9', true); // '0' = U+0030, '9' = U+0039
        assert_eq!('A' < 'a', true); // U+0041 < U+0061
        assert_eq!('a' < '😀', true); // 'a' = U+0061 (97), '😀' = U+1F600 (128512)
    }

    #[test]
    pub fn test_integers() {
        let a: i32 = -42;
        let b: i32 = 42;
        let c: i32 = 42;

        assert_eq!(a == b, false);
        assert_eq!(b == c, true);
        assert_eq!(a < b, true);
        assert_eq!(b > a, true);

        // Different types require casting
        let x: i8 = 10;
        let y: u32 = 10;
        assert_eq!(x as u32 == y, true);
    }

    #[test]
    pub fn test_floats() {
        let a: f64 = 3.14;
        let b: f64 = 3.14;
        let nan = f64::NAN;

        // Normal equality
        assert_eq!(a == b, true);
        assert_eq!(a != (a + 1.0), true);

        // NaN != NaN, (not reflexive) PartialEq
        // NaN != float (not comparable) PartialEq
        // NaN < <= > >= float, is always false (not orderable) PartialOrd

        // ⚠️ NaN breaks reflexivity
        assert_eq!(nan == nan, false); // ¡¡NaN ≠ NaN!!
        assert_eq!(nan < 0.0, false); // NaN < X always false
        assert_eq!(nan > 0.0, false); // NaN > X always false
        assert_eq!(nan == 0.0, false); // NaN == X always false
        assert!(nan != nan); // This is TRUE
    }

    #[test]
    pub fn test_strings() {
        let s1 = "apple";
        let s2 = "apple";
        let s3 = "banana";

        // Value comparison
        assert_eq!(s1 == s2, true);
        assert_eq!(s1 != s3, true);

        // Lexicographic order (alphabetical)
        assert_eq!(s1 < s3, true); // "apple" < "banana"
        assert_eq!("abc" < "abd", true); // Compares point by point
        assert_eq!("a" < "aa", true); // Prefix is less
        assert_eq!("hello_😀" > "hello_a", true); // '😀' = U+1F600 (128512) > 'a' = U+0061 (97)

        // String vs &str
        let owned = String::from("apple");
        assert_eq!(owned == s1, true); // Automatically dereferenced
    }
}

/*
Float

═════════════════════════════════════════════════════════════════════════════
NaN (Not a Number) IN FLOATS
═════════════════════════════════════════════════════════════════════════════

1. WHEN NaN APPEARS
─────────────────────────────────────────────────────────────────────────────

    NaN is the solution to the problem of representing a value that is an 
    indeterminate number. for example 0.0 / 0.0

    * Hardware natively supports it
    * This allows the calculation to continue without panicking (fault tolerance)
    * Easy detection: .is_nan() at the end instead of try/catch
    * Compatible with complex math libraries

  A) INDETERMINATE MATHEMATICAL OPERATIONS:
     0.0 / 0.0 = NaN
     Inf - Inf = NaN
     Inf / Inf = NaN
     Inf * 0.0 = NaN
     (-Inf) + Inf = NaN
     (-1.0).sqrt() = NaN
     (-5.0).ln() = NaN
     (-2.0).log10() = NaN

  C) OPERATIONS WITH NaN: (NaN propagation)
     NaN + 5.0               → NaN      (NaN propagates)
     NaN * 0.0               → NaN      (NaN propagates)
     NaN / 2.0               → NaN      (NaN propagates)
     (5.0).min(NaN)          → NaN      (min with NaN = NaN)

  D) DIRECT CONSTANT:
     f64::NAN                → NaN      (predefined constant)
     f32::NAN                → NaN      (in f32)

  E) PARSING "NaN" FROM STRING:
     "NaN".parse::<f64>()    → Ok(NaN)  (successful "NaN" parse)
     "nan".parse::<f64>()    → Error    (Rust is case-sensitive)
     "NAN".parse::<f64>()    → Error    (must be exactly "NaN")

  F) ERRONEOUS PARSING DOES NOT PRODUCE NaN: produces Err
     "abc".parse::<f64>()    → Err
     "12.34.56".parse()      → Err
     "".parse::<f64>()       → Err


2. COMPARISONS WITH NaN
─────────────────────────────────────────────────────────────────────────────

  A) BROKEN REFLEXIVITY (main problem):
     NaN == NaN  : false    ⚠️ (¡¡Not equal to itself!!)
     NaN == (any float) : false    (they are different)

  B) ORDERED COMPARISONS (all false):
     NaN < <= > >= (any float) : false
     (any float) < <= > >= NaN : false


═════════════════════════════════════════════════════════════════════════════
INFINITY (Inf) IN FLOATS
═════════════════════════════════════════════════════════════════════════════

1. WHEN INFINITY APPEARS
─────────────────────────────────────────────────────────────────────────────
    +Inf represents a numerical value that is larger than any other finite number.
    -Inf represents a numerical value that is smaller than any other finite number.
    f64::MAX < Inf

  A) DIVISION BY ZERO:
     1.0 / 0.0    → +Inf (positive infinity)
     -1.0 / 0.0   → -Inf (negative infinity)
     5.0 / 0.0    → +Inf

  B) OVERFLOW:
     f64::MAX + f64::MAX     → +Inf
     f64::MAX * 2.0          → +Inf
     10.0_f64.powi(400)      → +Inf (very large number)

  C) DIRECT CONSTANTS:
     f64::INFINITY           → +Inf
     f64::NEG_INFINITY       → -Inf
     f32::INFINITY           → +Inf (in f32)

  D) STRING PARSING:
     "inf".parse::<f64>()    → Ok(f64::INFINITY)
     "-inf".parse::<f64>()   → Ok(f64::NEG_INFINITY)
     "Infinity".parse()      → Error (not valid in Rust)


2. OPERATIONS WITH INFINITY
─────────────────────────────────────────────────────────────────────────────

  A) BASIC ARITHMETIC:
    Inf + - * / (finite float): Inf

  B) INDETERMINATE CASES (return NaN):
     Inf - Inf       → NaN         (indeterminate)
     Inf + (-Inf)    → NaN         (indeterminate)
     Inf / Inf       → NaN         (indeterminate)
     Inf * 0.0       → NaN         (indeterminate)
     Inf + - NaN     → NaN         (NaN propagates)

  C) OPERATIONS WITH ZERO:
     0.0 * Inf       → NaN
     0.0 / Inf       → 0.0         (zero is "small" compared to Inf)

  D) NEGATIVE INFINITY:
     -Inf + 100      → -Inf
     -Inf - 100      → -Inf
     -Inf * -1.0     → +Inf        (negative × negative = positive)


3. COMPARISONS WITH INFINITY
─────────────────────────────────────────────────────────────────────────────

  A) REFLEXIVITY (equal to itself): Eq
     Inf == Inf              → true   ✓ (unlike NaN)
     -Inf == -Inf            → true   ✓
     Inf == -Inf             → false  (opposite signs)

  B) ORDER COMPARISONS: Ord
     Inf > Inf              → false  (not greater than itself)
     Inf > 1e308            → true   (greater than any finite number)
     -Inf < -1e308          → true   (less than any finite number)
     Inf > -Inf             → true
     Inf >= > < <= NaN      → false  (NaN breaks comparisons)

*/

// ═════════════════════════════════════════════════════════════════════════════
// MODULE 2: REFERENCES VS RAW POINTERS
// ═════════════════════════════════════════════════════════════════════════════
/*
    CRITICAL DIFFERENCE:

    &T (reference)
    • compares the CONTENT (automatic dereference)
    • &5 == &5 → TRUE (compares values)

    *const T (raw pointer)
    • Compares the MEMORY ADDRESS (not the content)
    • 0x7fff1234 == 0x7fff5678 → FALSE (different addresses)
*/
#[cfg(test)]
mod references_vs_pointers {

    // references compare values
    #[test]
    pub fn test_references_equal() {
        let x = 5;
        let y = 5;

        // ✅ Reference compares values
        assert_eq!(&x, &y); // TRUE (both are worth 5)
    }

    // pointers compare addresses
    #[test]
    pub fn test_pointers_addresses() {
        println!("\n▶ RAW POINTERS - Compare ADDRESSES");
        let x = 5;
        let y = 5;

        // ❌ Pointer compares stack address (different variables)
        let ptr_x: *const i32 = &x as *const i32;
        let ptr_y: *const i32 = &y as *const i32;
        assert_ne!(ptr_x, ptr_y); // FALSE (different addresses)

        // ✅ The SAME pointer to itself is equal
        assert_eq!(ptr_x, ptr_x); // TRUE (same address number)
    }

    #[test]
    pub fn test_pointers_heap() {
        let vec1: Vec<i32> = vec![1, 2, 3];
        let ptr_before = vec1.as_ptr(); // Pointer to heap data

        let vec2 = vec1; // Move (ownership changed but heap data not copied)
        let ptr_after = vec2.as_ptr(); // Same pointer to heap

        // ✅ Both point to the SAME place in heap
        assert_eq!(ptr_before, ptr_after);
    }

    // pointer content against reference
    #[test]
    pub fn test_references_vs_pointers() {
        let x = 10;
        let ref_x: &i32 = &x; // Reference
        let ptr_x: *const i32 = &x; // Raw pointer

        // ✅ Reference compares value
        assert_eq!(ref_x, &x); // TRUE
                               // ✅ Raw pointer compares address
        assert_eq!(ptr_x, ref_x as *const i32); // TRUE (same address)
                                                // pointer content equals the value of x
        assert_eq!(unsafe { *ptr_x }, *ref_x); // Dereference raw pointer (unsafe)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// MODULE 3: ARRAYS, SLICES AND VECTORS
// ═════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod collections {
    /*
    COMPARISON IN COLLECTIONS: (Arrays, Slices, Vectors)

        • PartialEq/Eq: compares element by element by value, not by memory address.
            [1,2,3] == [1,2,3] → TRUE
            [1,2,3] == [1,2,4] → FALSE
    */

    // Arrays compare content, not address
    #[test]
    pub fn test_arrays() {
        let arr1 = [1, 2, 3];
        let arr2 = [1, 2, 3];
        let arr3 = [1, 2, 4];

        assert_eq!(arr1, arr2); // TRUE (same content)
        assert_ne!(arr1, arr3); // FALSE (different element)
        assert_eq!(arr1 < arr3, true); // Lexicographic order

        println!("✓ arrays: element by element comparison");
    }

    // Slices compare content, not address
    #[test]
    pub fn test_slices() {
        let arr = [1, 2, 3, 4, 5];
        let slice1 = &arr[0..3]; // [1, 2, 3]
        let slice2 = &arr[0..3];
        let slice3 = &arr[1..4]; // [2, 3, 4]

        assert_eq!(slice1, slice2); // TRUE (same content)
        assert_ne!(slice1, slice3); // FALSE (different content)
        assert_eq!(slice1.len(), 3);
    }

    // Vectors compare content, not address
    #[test]
    pub fn test_vectors() {
        let vec1 = vec![1, 2, 3];
        let vec2 = vec![1, 2, 3];
        let vec3 = vec![1, 2, 3, 4];

        // ✅ Compares content, NOT heap address
        assert_eq!(vec1, vec2); // TRUE (same content)
        assert_ne!(vec1, vec3); // FALSE (different size/content)

        // Heap addresses different
        assert_ne!(vec1.as_ptr(), vec2.as_ptr()); // Different places in heap
    }

    // Lexicographic order in collections
    #[test]
    pub fn test_collections_order() {
        let a = [1, 2, 3];
        let b = [1, 2, 4];

        assert_eq!(a < b, true); // [1,2,3] < [1,2,4] (at position 2: 3<4)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// ENUMS
// ═════════════════════════════════════════════════════════════════════════════
/*
 Enums are ordered according to the order of definition of their variants and 
 not by their associated content.

*/
#[cfg(test)]
mod compound_types {

    #[allow(unused_variables)]
    #[allow(dead_code)]
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Priority {
        Low,
        Medium,
        High,
    }

    #[test]
    pub fn test_enum_ord() {
        println!("\n▶ ENUM WITH #[derive(Ord)]");
        let low = Priority::Low;
        let high = Priority::High;

        assert_ne!(low, high);
        assert_eq!(low < high, true); // Order: Low < Medium < High

        // Order of definition in enum
        assert_eq!(Priority::Low < Priority::Medium, true);
        assert_eq!(Priority::Medium < Priority::High, true);
        println!("✓ Enums: order by position of definition (top < bottom)");
    }

    // Example with associated data
    // still compares by order and not by content

    #[allow(unused_variables)]
    #[allow(dead_code)]
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum PriorityComplex {
        Low(String),
        Medium(u8),
        High(bool),
    }

    #[test]
    pub fn test_custom_impl() {
        println!("\n▶ CUSTOM TYPE WITH DERIVED ORD");
        let p1 = PriorityComplex::Low("Task A".into());
        let p2 = PriorityComplex::Medium(5);
        let p3 = PriorityComplex::High(true);

        assert_eq!(p1 < p2, true);
        assert_eq!(p2 < p3, true);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// MODULE 6: TUPLES
// ═════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tuples {
    /*
    COMPARISON IN TUPLES:

    Tuples compare element by element, in order:
    (1, 'a') < (1, 'b') → TRUE (first element equal, second a<b)
    (1, 'b') < (2, 'a') → TRUE (first element 1<2)

    Require that ALL types implement the comparison trait.
    */

    #[test]
    pub fn test_tuples_eq() {
        println!("\n▶ TUPLE EQUALITY");
        let t1 = (1, "hello", 3.14);
        let t2 = (1, "hello", 3.14);
        let t3 = (1, "hello", 3.15);

        assert_eq!(t1, t2); // TRUE
        assert_ne!(t1, t3); // FALSE
        println!("✓ tuples: element by element comparison");
    }

    #[test]
    pub fn test_tuples_ord() {
        println!("\n▶ TUPLE ORDERING (lexicographic)");
        let t1 = (1, 2, 3);
        let t2 = (1, 2, 4);
        let t3 = (1, 3, 0);
        let t4 = (2, 0, 0);

        assert_eq!(t1 < t2, true); // Position 2: 3<4
        assert_eq!(t1 < t3, true); // Position 1: 2<3
        assert_eq!(t1 < t4, true); // Position 0: 1<2

        // Order by field: first → second → third
        println!("✓ tuples: lexicographic order (field by field)");
    }

    #[test]
    pub fn test_nested_tuples() {
        println!("\n▶ NESTED TUPLES");
        let nested1 = ((1, 2), (3, 4));
        let nested2 = ((1, 2), (3, 4));

        assert_eq!(nested1, nested2);
        assert_eq!(((1, 2), (3, 3)) < nested1, true);
        println!("✓ nested tuples: recursive order");
    }
}
