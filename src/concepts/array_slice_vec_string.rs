#[allow(unused_variables)]
#[allow(dead_code)]
#[test]
fn index() {
    arrays::arrays();
    arrays::array_iteration();

    vectors::vectors();
    vectors::vector_growth();
    vectors::vector_move();

    array_vs_vec::comparison();
    array_vs_vec::performance_characteristics();

    slices::slices();
    slices::slice_ranges();
    slices::slice_from_vec();
    slices::slice_operations();

    mutable_slices::mutable_slices();
    mutable_slices::function_with_mut_slice();
    mutable_slices::mut_str_limited();

    vector_slice::vector_slice();

    strings::strings();
    strings::string_mutation();
    strings::string_is_move();

    string_slices::string_slices();
    string_slices::str_from_string();

    string_literals::string_literals();

    utf8_slicing::utf8_slicing();
    utf8_slicing::safe_slicing_with_get();
    utf8_slicing::char_iteration();
    utf8_slicing::invalid_slice_panics();
}

/*
========================================================================
ARRAYS
========================================================================

    Fixed size on the stack

    ARRAYS [T; N] - FIXED SIZE ON STACK:
    --------------------------------------------
        let arr: [i32; 4] = [10, 20, 30, 40];

        STACK (16 bytes, all inline):
        ┌─────┬─────┬─────┬─────┐
        │  10 │  20 │  30 │  40 │  ← direct data
        └─────┴─────┴─────┴─────┘
          [0]   [1]   [2]   [3]

        Characteristics:
        ✓ Size known at compile time
        ✓ No heap allocation
        ✓ Copy if T:Copy

    CREATION:
    --------------------------------------------
        let arr1: [i32; 4] = [10, 20, 30, 40];  // explicit

        let arr2 = [1; 4];                      // initialize all with 1

        let arr4: [i32; 4];                     // uninitialized
                                                // unsafe to access (garbage values)

        let arr3: [i32; 0] = [];                // empty array, 0 elements, 0 bytes,
                                                // safe to access, nothing to read
                                                // useful for generics [u8; N]
                                                // where N can be 0
*/
#[cfg(test)]
mod arrays {

    #[test]
    pub fn arrays() {
        use std::mem;

        let arr: [i32; 4] = [10, 20, 30, 40];
        let _arr2: [i32; 4] = [1; 4]; // initialize all with 1
        let _arr4: [i32; 4]; // uninitialized (garbage values)
        let _arr3: [i32; 0] = []; // empty array

        // Stack size = N * size_of::<T>()
        assert_eq!(mem::size_of::<[i32; 4]>(), 16); // 4 * 4 bytes

        // Index access
        assert_eq!(arr[0], 10);
        assert_eq!(arr[3], 40);

        // Is Copy if T is Copy
        let arr2 = arr; // copy, not move
        assert_eq!(arr[0], arr2[0]); // arr still valid

        // Initialization with repeated value
        let zeros: [i32; 100] = [0; 100];
        assert_eq!(zeros[100 - 1], 0);
    }

    #[test]
    pub fn array_iteration() {
        let arr: [i32; 4] = [1, 2, 3, 4];

        // Iteration by reference
        let sum: i32 = arr.iter().sum();
        assert_eq!(sum, 10);

        // Iteration with index
        for (i, &val) in arr.iter().enumerate() {
            assert_eq!(val, (i + 1) as i32);
        }
    }
}

/*
========================================================================
VECTORS
========================================================================

    VECTORS Vec<T> - DYNAMIC SIZE ON HEAP:
    --------------------------------------------
        let vec: Vec<i32> = vec![10, 20, 30, 40];

        STACK (24 bytes):                      HEAP:
        ┌─────────────────────┐               ┌─────┬─────┬─────┬─────┬─────┬─────┐
        │ ptr ────────────────┼──────────────▶│  10 │  20 │  30 │  40 │  ?  │  ?  │
        ├─────────────────────┤               └─────┴─────┴─────┴─────┴─────┴─────┘
        │ len: 4              │                 [0]   [1]   [2]   [3]  (extra capacity)
        ├─────────────────────┤
        │ cap: 6              │  ← may have extra capacity
        └─────────────────────┘

        Characteristics:
        ✓ Dynamic size (push/pop)
        ✓ Heap allocation
        ✗ NOT Copy (has Drop)

    CAPACITY AND GROWTH:
    --------------------------------------------
        When capacity is reached, it doubles:
        4, 8, 16, 32, 64, 128
        If initial capacity n was assigned, it would double each time: n*2, n*4, n*8, n*16...
*/
#[cfg(test)]
mod vectors {

    #[test]
    pub fn vectors() {
        use std::mem;
        let vec: Vec<i32> = vec![10, 20, 30, 40];

        // Stack size always 24 bytes (ptr + len + cap)
        assert_eq!(mem::size_of::<Vec<i32>>(), 24);

        // len and capacity
        assert_eq!(vec.len(), 4);
        assert!(vec.capacity() >= 4);

        // Index access
        assert_eq!(vec[0], 10);
        assert_eq!(vec[3], 40);
    }

    #[test]
    pub fn vector_growth() {
        let mut vec: Vec<i32> = Vec::new();
        assert_eq!(vec.capacity(), 0);

        // Push increases capacity automatically
        vec.push(1);
        let cap1 = vec.capacity();
        assert!(cap1 >= 4);

        // Capacity grows exponentially
        for i in 2..=100 {
            vec.push(i);
            // dbg!(&vec.capacity()); // 4, 8, 16, 32, 64, 128
            // If initial capacity n was assigned, it would double each time
        }
        assert!(vec.capacity() >= 100);

        // with_capacity pre-allocates
        let vec2: Vec<i32> = Vec::with_capacity(1000);
        assert_eq!(vec2.len(), 0);
        assert!(vec2.capacity() >= 1000);
    }

    #[test]
    pub fn vector_move() {
        let vec1: Vec<i32> = vec![1, 2, 3];
        let ptr_before = vec1.as_ptr();

        let vec2 = vec1; // move, not copy
        let ptr_after = vec2.as_ptr();

        // The heap pointer is the same
        assert_eq!(ptr_before, ptr_after);
        // vec1 is no longer valid
    }
}

/*
========================================================================
ARRAY_VS_VEC
========================================================================

    COMPARISON:
    --------------------------------------------
        ┌────────────────────┬────────────────────┬────────────────────────────────┐
        │ Aspect             │ [T; N] (Array)     │ Vec<T>                         │
        ├────────────────────┼────────────────────┼────────────────────────────────┤
        │ Allocation         │ Stack              │ Heap                           │
        │ Size               │ Fixed (compile)    │ Dynamic (runtime)              │
        │ Overhead           │ 0 bytes            │ 24 bytes (ptr+len+cap)         │
        │ Copy               │ ✓ (if T: Copy)     │ ✗ (move or clone)              │
        │ Cache locality     │ Excellent          │ Good                           │
        │ Grows/shrinks      │ ✗                  │ ✓                              │
        │ Max size           │ ~MB (stack limit)  │ ~GB (heap)                     │
        │ Alloc speed        │ Instant            │ Slower (syscall)               │
        └────────────────────┴────────────────────┴────────────────────────────────┘

    WHY CAN ARRAY BE FASTER?:
    --------------------------------------------
        1. STACK vs HEAP:
           Array: instant allocation (just moves stack pointer)
           Vec: syscall to OS for heap memory (slower)

        2. NO INDIRECTION:
           Array: data inline, direct access
           Vec: ptr → heap, one extra level of indirection

        3. COMPILER OPTIMIZATION:
           Array: size known → loop unrolling, SIMD
           Vec: dynamic size → fewer optimizations possible

    LOOP UNROLLING:
    --------------------------------------------
        Original code:
          for i in 0..4 {
              result[i] = arr[i] * 2;
          }

        After unrolling:
          result[0] = arr[0] * 2;
          result[1] = arr[1] * 2;
          result[2] = arr[2] * 2;
          result[3] = arr[3] * 2;

        ✓ No loop jump overhead
        ✓ CPU can execute in parallel (ILP)
        ✗ Only possible if size known at compile time

    SIMD (SINGLE INSTRUCTION MULTIPLE DATA):
    --------------------------------------------
        Modern CPU has SIMD registers (SSE, AVX, NEON):

        Scalar processing (no SIMD):
          result[0] = arr[0] * 2;
          result[1] = arr[1] * 2;
          result[2] = arr[2] * 2;
          result[3] = arr[3] * 2;
          ✗ 4 instructions, 4 cycles

        SIMD processing (AVX-256: 256 bits = 4 x i32):
          result[0..4] = arr[0..4] * 2;   (all in parallel!)
          ✓ 1 instruction, 1 cycle

        Compiler can use SIMD only if:
          ✓ Size known at compile time
          ✓ Sequential memory access
          ✓ No dependencies between iterations
          ✗ Vec dynamic size → harder to vectorize

    WHEN TO USE EACH ONE:
    --------------------------------------------
        USE ARRAY [T; N]:
          • Size known at compile time
          • Small data (< 1KB typically)
          • Maximum performance needed
          • Examples: coordinates [f32; 3], matrix [f64; 16], buffer [u8; 256]

        USE VEC<T>:
          • Size dynamic or unknown at compile time
          • Large data (> several KB)
          • Need push/pop/insert/remove
          • Examples: user list, file content, network input
*/
#[cfg(test)]
mod array_vs_vec {
    #[test]
    pub fn comparison() {
        // Array: Copy if T is Copy
        let arr: [i32; 4] = [1, 2, 3, 4];
        let arr2 = arr; // copy
        assert_eq!(arr[0], arr2[0]); // both valid

        // Vec: Move, not Copy
        let vec: Vec<i32> = vec![1, 2, 3, 4];
        let vec2 = vec; // move
        // vec is no longer valid
        assert_eq!(vec2[0], 1);

        // Clone to copy Vec
        let vec3 = vec2.clone();
        assert_eq!(vec2[0], vec3[0]); // both valid

        println!("  ✅ array_vs_vec::comparison");
    }

    #[test]
    pub fn performance_characteristics() {
        use std::mem;

        // Array: no overhead
        let arr: [i32; 1000] = [0; 1000];
        assert_eq!(mem::size_of_val(&arr), 4000); // exactly 1000 * 4 bytes

        // Vec: 24 bytes overhead on stack
        let vec: Vec<i32> = vec![0; 1000];
        assert_eq!(mem::size_of_val(&vec), 24); // only ptr+len+cap

        // Vec data on heap
        assert!(vec.capacity() >= 1000);
    }
}

/*
========================================================================
SLICES
========================================================================

    SLICES &[T]:
    --------------------------------------------
        Array, Vector, String...
        ┌─────┬─────┬─────┬─────┐
        │  10 │  20 │  30 │  40 │
        └─────┴──▲──┴──▲──┴─────┘
                 │     │
                 │     └─────────────────┐
                 │                       │
        slice: &[i32] (16 bytes, fat pointer)
        ┌─────────────────────┐          │
        │ ptr ────────────────┼──────────┘  (points to arr[1])
        ├─────────────────────┤
        │ len: 2 (Fixed)      │
        └─────────────────────┘

    CHARACTERISTICS:
    --------------------------------------------
        • Fixed len: Cannot change the size. Must create a new one.
          If it changed, you'd point beyond valid data.

        • Len calculated at runtime when creating the slice:
          let slice: &[i32] = &vec![1, 2, 3][..];  // vec.len() unknown at compile time

        • Immutable: Cannot change ptr or len.
          let slice: &[i32] = &arr[1..3];
          let slice: &[i32] = &vec[1..4];
          let slice: &str = &s[0..4];  // access to UTF-8 bytes (may not be valid chars)

        • is Copy (just ptr + len)
*/
#[cfg(test)]
mod slices {

    #[test]
    pub fn slices() {
        use std::mem;
        let arr: [i32; 5] = [10, 20, 30, 40, 50];
        let slice: &[i32] = &arr[1..3]; // [20, 30]

        // Fat pointer: ptr + len = 16 bytes
        assert_eq!(mem::size_of::<&[i32]>(), 16);

        // Slice contents
        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0], 20);
        assert_eq!(slice[1], 30);

        // Slice is Copy
        let slice2 = slice;
        assert_eq!(slice[0], slice2[0]); // both valid
    }

    #[test]
    pub fn slice_ranges() {
        let _arr: [i32; 5] = [10, 20, 30, 40, 50];

        // Different ranges:
        // &arr[1..3]      // [20, 30]      (excludes index 3)
        // &arr[1..=3]     // [20, 30, 40]  (includes index 3)
        // &arr[1..]       // [20, 30, 40, 50]
        // &arr[..3]       // [10, 20, 30]
        // &arr[..=3]      // [10, 20, 30, 40]
        // &arr[..]        // [10, 20, 30, 40, 50]
    }

    #[test]
    pub fn slice_from_vec() {
        let vec: Vec<i32> = vec![10, 20, 30, 40, 50];
        let slice: &[i32] = &vec[1..4]; // [20, 30, 40]

        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], 20);

        // The slice points inside the Vec's heap
        assert!(slice.as_ptr() > vec.as_ptr()); // slice points to vec[1]
    }

    #[test]
    pub fn slice_operations() {
        let arr: [i32; 5] = [10, 20, 30, 40, 50];

        // slice1: Slice is Copy, duplicating doesn't consume original
        let slice1: &[i32] = &arr[1..4]; // [20, 30, 40]
        let slice2 = slice1;
        assert_eq!(slice1.as_ptr(), slice2.as_ptr());

        // slice2: Trim slice with subrange
        let slice: &[i32] = &arr[..];
        let trimmed1 = &slice[1..4]; // [20, 30, 40]
        let trimmed2 = &slice[..3]; // [10, 20, 30]
        assert_eq!(trimmed1, &[20, 30, 40]);
        assert_eq!(trimmed2, &[10, 20, 30]);

        // slice3: Create Vec from slice copies data to heap
        let vec: Vec<i32> = slice1.to_vec();
        assert_ne!(vec.as_ptr(), slice1.as_ptr()); // different memory

        // slice4: Multiple ways to copy slice to Vec
        let v1: Vec<i32> = slice1.to_vec();
        let v2: Vec<i32> = Vec::from(slice1);
        let v3: Vec<i32> = slice1.iter().copied().collect();
        assert_eq!(v1, v2);
        assert_eq!(v2, v3);
    }
}

/*
========================================================================
MUTABLE SLICES
========================================================================

    MUTABLE SLICES &mut [T]:
    --------------------------------------------
        ┌──────────────────────┬──────────────────┬──────────────────────────┐
        │ Operation            │ &[T] (immutable) │ &mut [T] (mutable)       │
        ├──────────────────────┼──────────────────┼──────────────────────────┤
        │ Read values          │ ✓                │ ✓                        │
        │ Edit values          │ ✗                │ ✓                        │
        │ Multiple refs        │ ✓ (many)         │ ✗ (only 1)               │
        │ Edit vec/array       │ ✓ (no borrow)    │ ✗ (while it exists)      │
        │ is Copy (ptr + len)  │ ✓                │ ✗                        │
        └──────────────────────┴──────────────────┴──────────────────────────┘

    WHY IS &mut str NOT COMMON?:
    --------------------------------------------
        FIXED-SIZE TYPES (i32, f64, etc.):
            • Each element occupies exactly N bytes
            • Modifying one element does NOT affect others
            ✓ &mut [i32] works perfectly

        UTF-8 STRINGS:
            • Each character occupies 1-4 bytes (variable)
            • Changing 'a' (1 byte) to '🦀' (4 bytes) would shift everything
            ✗ &mut str very limited (only same-size character changes)

    MUTABLE REFERENCE RESTRICTIONS:
    --------------------------------------------
        1. Only ONE mutable reference at a time:
            let mut arr = [1, 2, 3, 4];
            let mut_slice1 = &mut arr[0..2];
            let mut_slice2 = &mut arr[2..4];  // ✗ ERROR: mut_slice1 already exists

        2. Cannot mutate the vec/array while mutable slice exists:
            let mut vec = vec![1, 2, 3, 4, 5];
            let mut_slice = &mut vec[1..4];
            vec.push(6);  // ✗ ERROR: cannot mutate vec while mut_slice exists
*/
#[cfg(test)]
mod mutable_slices {
    #[test]
    pub fn mutable_slices() {
        let mut arr: [i32; 4] = [10, 20, 30, 40];
        let slice_mut: &mut [i32] = &mut arr[1..3];

        // Modify elements
        slice_mut[0] = 200;
        slice_mut[1] *= 10;

        assert_eq!(slice_mut[0], 200);
        assert_eq!(slice_mut[1], 300);
        assert_eq!(arr, [10, 200, 300, 40]);
    }

    #[test]
    pub fn function_with_mut_slice() {
        fn double_values(data: &mut [i32]) {
            for x in data.iter_mut() {
                *x *= 2;
            }
        }

        let mut vec = vec![1, 2, 3, 4, 5];
        double_values(&mut vec[1..4]); // Only modifies [1], [2], [3]

        assert_eq!(vec, [1, 4, 6, 8, 5]);
    }

    #[test]
    pub fn mut_str_limited() {
        let mut s = String::from("hello");

        // Only operations that do NOT change length
        s.make_ascii_uppercase();
        assert_eq!(s, "HELLO");

        // This works because 'H' and 'h' occupy the same byte
    }
}

/*
========================================================================
SLICES: FLEXIBILITY: MULTIPLE SOURCES
========================================================================

    DEREF COERCION (Deref() and DerefMut()):
    --------------------------------------------
        A reference of Array|Vec|String, can be used as an slice.
        unifying the types, allowing to use the same function for multiple sources.

        • &[T; N]  →  &[T]
        • &Vec<T>  →  &[T]
        • Box<[T]>  →  &[T]
        • Struct (with Deref -> &[T]) →  &[T]
        • &String  →  &str (&str is an slice of characters, valid utf-8 parts, not &[u8] slice)

        Same for mutable references:

        • &mut [T; N]  →  &mut [T]
        • &mut Vec<T>  →  &mut [T]
        • Box<mut [T]>  →  &mut [T]
        • &mut String  →  &mut str (while respect the same length of the slice (utf-8 bytes))

    FLEXIBLE vs RESTRICTIVE:
    --------------------------------------------
        It is better to use slices instead of owning types
        Slices are flexible, because they can be created from multiple sources:

        FLEXIBLE - Accepts multiple sources:

            fn process_slice(data: &[i32]) {       // ← &[T] is flexible
                println!("{:?}", data);
            }

            let arr = [1, 2, 3];
            let vec = vec![1, 2, 3];

            process_slice(&arr);        // ✓ Array → &[i32] (Deref coercion)
            process_slice(&vec);        // ✓ Vec → &[i32]   (Deref coercion)
            process_slice(&vec[1..3]);  // ✓ slice

        RESTRICTIVE - Only one source:

            fn process_vec(data: &Vec<i32>) {
                println!("{:?}", data);
            }

            process_vec(&vec);           // ✓ Only works with Vec
*/

/*
========================================================================
VECTOR SLICE
========================================================================

    VECTOR SLICE:
    --------------------------------------------
        let vec: Vec<i32> = vec![10, 20, 30, 40, 50];
        let slice: &[i32] = &vec[1..4];  // [20, 30, 40]

        STACK                                 HEAP
        vec: Vec<i32> (24 bytes)
        ┌─────────────────────┐               ┌─────┬─────┬─────┬─────┬─────┐
        │ ptr ────────────────┼──────────────▶│  10 │  20 │  30 │  40 │  50 │
        ├─────────────────────┤               └─────┴──▲──┴─────┴──▲──┴─────┘
        │ len: 5              │                        │           │
        ├─────────────────────┤                        │           │
        │ cap: 5              │                        │           │
        └─────────────────────┘                        │           │
                                                       │           │
        slice: &[i32] (16 bytes)                       │           │
        ┌─────────────────────┐                        │           │
        │ ptr ────────────────┼────────────────────────┘           │
        ├─────────────────────┤         (points to vec[1])          │
        │ len: 3              │  ──────────────────────────────────┘
        └─────────────────────┘         (covers up to vec[3])

        ✓ slice points WITHIN vec's heap
        ✓ No data copy
        ✓ slice must live less than vec (lifetime)
*/
#[cfg(test)]
mod vector_slice {
    #[test]
    pub fn vector_slice() {
        let vec: Vec<i32> = vec![10, 20, 30, 40, 50];
        let slice: &[i32] = &vec[1..4];

        // Slice points inside the heap
        assert_eq!(slice.len(), 3);
        assert_eq!(slice, &[20, 30, 40]);

        // Verify that it points to the same heap
        let vec_ptr = vec.as_ptr();
        let slice_ptr = slice.as_ptr();

        // slice_ptr should be vec_ptr + 4 bytes (offset of 1 i32)
        unsafe {
            assert_eq!(slice_ptr, vec_ptr.add(1));
        }
    }
}

/*
========================================================================
STRINGS
========================================================================

    STRINGS String - UTF-8 on heap:
    --------------------------------------------
        let s = String::from("Hello 🦀");

        STACK (24 bytes):                      HEAP:
        ┌─────────────────────┐               ┌───┬───┬───┬───┬───┬────┬────┬────┬────┐
        │ ptr ────────────────┼──────────────▶│ H │ e │ l │ l │ o │0xF0│0x9F│0xA6│0x80│
        ├─────────────────────┤               └───┴───┴───┴───┴───┴────┴────┴────┴────┘
        │ len: 9              │                 UTF-8 bytes (🦀 = 4 bytes)
        ├─────────────────────┤
        │ cap: 9              │
        └─────────────────────┘

    CHARACTERISTICS:
    --------------------------------------------
        ✓ Same as Vec<u8> but guarantees valid UTF-8
        ✗ NOT Copy
*/
#[cfg(test)]
mod strings {
    #[test]
    pub fn strings() {
        use std::mem;
        let s = String::from("Hello 🦀");

        // Stack size always 24 bytes
        assert_eq!(mem::size_of::<String>(), 24);

        // len is in bytes, not characters
        assert_eq!(s.len(), 10); // "Hello " (6 bytes) + 🦀 (4 bytes)
        assert_eq!(s.chars().count(), 7); // 7 characters
    }

    #[test]
    pub fn string_mutation() {
        let mut s = String::from("Hello");

        s.push(' ');
        s.push_str("world");

        assert_eq!(s, "Hello world");
        assert!(s.capacity() >= s.len());
    }

    #[test]
    pub fn string_is_move() {
        let s1 = String::from("test");
        let ptr_before = s1.as_ptr();

        let s2 = s1; // move
        let ptr_after = s2.as_ptr();

        // The heap pointer is the same
        assert_eq!(ptr_before, ptr_after);
        // s1 is no longer valid
    }
}

/*
========================================================================
STRING_SLICES
========================================================================

    STRING SLICES &str:
    --------------------------------------------
        let s = String::from("Hello world");
        let slice: &str = &s[0..5];  // "Hello"

        STACK                                 HEAP
        s: String (24 bytes)
        ┌─────────────────────┐               ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
        │ ptr ────────────────┼──────────────▶│ H │ e │ l │ l │ o │   │ w │ o │ r │ l │ d
        ├─────────────────────┤               └─▲─┴───┴───┴─▲─┴───┴───┴───┴───┴───┴───┘
        │ len: 11             │                 │           │
        ├─────────────────────┤                 │           │
        │ cap: 11             │                 │           │
        └─────────────────────┘                 │           │
                                                │           │
        slice: &str (16 bytes)                  │           │
        ┌─────────────────────┐                 │           │
        │ ptr ────────────────┼─────────────────┘           │
        ├─────────────────────┤    (points to s[0])         │
        │ len: 5              │  ───────────────────────────┘
        └─────────────────────┘    (covers up to s[4])

    CHARACTERISTICS:
    --------------------------------------------
        ✓ View of UTF-8 bytes (no copy)
        ✓ Copy (just ptr + len)
        ✓ Can point to String, literal, or another &str
*/
#[cfg(test)]
mod string_slices {
    #[test]
    pub fn string_slices() {
        use std::mem;
        let s = String::from("Hello world");
        let slice: &str = &s[0..5];

        // Fat pointer: 16 bytes
        assert_eq!(mem::size_of::<&str>(), 16);

        assert_eq!(slice, "Hello");
        assert_eq!(slice.len(), 5);

        // &str is Copy
        let slice2 = slice;
        assert_eq!(slice, slice2);
    }

    #[test]
    pub fn str_from_string() {
        let s = String::from("hello");

        // Multiple ways to get &str
        let slice1: &str = &s; // Deref coercion
        let slice2: &str = s.as_str(); // Explicit
        let slice3: &str = &s[..]; // Full slice

        assert_eq!(slice1, slice2);
        assert_eq!(slice2, slice3);
    }
}

/*
========================================================================
STRING_LITERALS
========================================================================

    String literals are fat pointers to the binary data section.

    STRING LITERALS &'static str:
    --------------------------------------------
        let literal: &'static str = "Hello 🦀";

        STACK (16 bytes):                      BINARY (.rodata):
        ┌─────────────────────┐               ┌───┬───┬───┬───┬───┬────┬────┬────┬────┐
        │ ptr ────────────────┼──────────────▶│ H │ e │ l │ l │ o │0xF0│0x9F│0xA6│0x80│
        ├─────────────────────┤               └───┴───┴───┴───┴───┴────┴────┴────┴────┘
        │ len: 9              │                 Embedded in the executable
        └─────────────────────┘

    CHARACTERISTICS:
    --------------------------------------------
        ✓ Data in .rodata (read-only data section)
        ✓ Lives for the entire program ('static)
        ✓ NO heap allocation
        ✓ Copy
*/
#[cfg(test)]
mod string_literals {
    #[test]
    pub fn string_literals() {
        let literal: &'static str = "Hello 🦀";

        // No heap allocation
        assert_eq!(literal.len(), 10);
        assert_eq!(literal.chars().count(), 7);

        // Is Copy
        let literal2 = literal;
        assert_eq!(literal, literal2);

        // Lives forever ('static)
        fn get_static() -> &'static str {
            "this lives forever"
        }
        let s = get_static();
        assert!(!s.is_empty());
    }
}

/*
========================================================================
UTF8_SLICING
========================================================================

    UTF-8 SLICING - Dangers:
    --------------------------------------------
        let s = String::from("Hello 🦀 rustaceans");

        Byte map:
        ┌───┬───┬───┬───┬───┬────┬────┬────┬────┬───┬───┬───┬...┐
        │ H │ e │ l │ l │ o │0xF0│0x9F│0xA6│0x80│   │ r │ u │...│
        └───┴───┴───┴───┴───┴────┴────┴────┴────┴───┴───┴───┴...┘
          0   1   2   3   4   5    6    7    8    9  10  11  ...
                          ◄──────── 🦀 ────────►
                          │    │    │    │
                          ✓    ✗    ✗    ✗    ✓  ← char boundaries
                         [5]  [6]  [7]  [8]  [9]

        ┌────────────────────┬─────────────────────────────────────────────┐
        │ Operation          │ Result                                      │
        ├────────────────────┼─────────────────────────────────────────────┤
        │ &s[0..5]           │ ✓ "Hello " (ends before emoji)              │
        │ &s[5..9]           │ ✓ "🦀" (full emoji, 4 bytes)                │
        │ &s[9..20]          │ ✓ " rustaceans" (after emoji)               │
        ├────────────────────┼─────────────────────────────────────────────┤
        │ &s[0..6]           │ ✗ PANIC! cuts inside emoji                  │
        │ &s[6..9]           │ ✗ PANIC! starts inside emoji                │
        └────────────────────┴─────────────────────────────────────────────┘

    HOW TO AVOID PANIC:
    --------------------------------------------
        1. Check first: s.is_char_boundary(idx)
        2. Use chars(): s.chars().take(n).collect::<String>()
        3. Use s.get(start..end) which returns Option<&str>
*/
#[cfg(test)]
mod utf8_slicing {
    #[test]
    pub fn utf8_slicing() {
        let s = String::from("Hello 🦀 rustaceans");

        // Check char boundaries
        assert!(s.is_char_boundary(0));
        assert!(s.is_char_boundary(6)); // start of 🦀
        assert!(!s.is_char_boundary(7)); // inside 🦀
        assert!(!s.is_char_boundary(8)); // inside 🦀
        assert!(!s.is_char_boundary(9)); // inside 🦀
        assert!(s.is_char_boundary(10)); // after 🦀

        // Valid slicing
        assert_eq!(&s[0..6], "Hello ");
        assert_eq!(&s[6..10], "🦀");
        assert_eq!(&s[10..], " rustaceans");
    }

    #[test]
    pub fn safe_slicing_with_get() {
        let s = String::from("Hello 🦀");

        // .get() returns Option instead of panic
        assert!(s.get(0..7).is_none()); // invalid (cuts in the middle of emoji)
        assert!(s.get(0..6).is_some()); // valid
        assert_eq!(s.get(6..10), Some("🦀"));
    }

    #[test]
    pub fn char_iteration() {
        let s = String::from("Hello 🦀");

        // Iterate by characters (not bytes)
        let chars: Vec<char> = s.chars().collect();
        assert_eq!(chars.len(), 7);
        assert_eq!(chars[6], '🦀');

        // char_indices gives byte index + character
        let indices: Vec<(usize, char)> = s.char_indices().collect();
        assert_eq!(indices[6], (6, '🦀'));
    }

    #[test]
    pub fn invalid_slice_panics() {
        let _s = String::from("Hello 🦀");
        // let _ = &s[6..7]; // PANIC! cuts inside emoji
    }
}
    