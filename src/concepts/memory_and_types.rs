/*
BYTE:
  Basic unit of memory (8 bits on modern systems)

WORD:
  Word: "Natural" size with which the CPU operates and moves data efficiently;
  matches the width of general purpose registers and the data bus.
  Pointers also have the size of a word.
  In 16-bit: 1 word = 2 bytes
  In 32-bit: 1 word = 4 bytes
  In 64-bit: 1 word = 8 bytes

ADDRESS:
  The number that identifies each byte in memory

USIZE:
  Unsigned integer type whose size depends on architecture = 1 word

ISIZE:
  Signed integer type whose size depends on architecture = 1 word

STACK:
  Memory region for variables with known size and short lifetime
  Variables are stored in a contiguous block of memory
  scoped to the function or block they are declared in

HEAP:
  Memory region for data with variable size and long lifetime
  Data is stored in a scattered block of memory (not contiguous)
  not scoped to the function or block they are declared in

PADDING:
  Bytes automatically added between variables or struct fields to satisfy
  alignment rules and optimize access, padding is filled until filling the word.

MEMORY ADDRESSES:
  Address    │  Content (byte)
  ───────────┼──────────────────
  0x1000     │  10101101  (byte 0) (values: 0-255)
  0x1001     │  11010010  (byte 1)
  0x1002     │  01001110  (byte 2)
  0x1003     │  00110101  (byte 3)
...

*/
/*
STACK LAYOUT - 64-BIT SYSTEM WITH DETAILED ALIGNMENT AND PADDING
═════════════════════════════════════════════════════════════════════════════

ALIGNMENT:
  Each data type must start at an address that is a multiple of its size,
  so the CPU can access it efficiently.
  Padding (empty bytes) is added between variables/fields if necessary,
  to satisfy this rule.

ALIGNMENT RULE:
  - bool/u8:   must be at address multiple of 1 (0.1000, 0x1001, 0x1002, ...)
  - u32:       must be at address multiple of 4 (0x1000, 0x1004, 0x1008, ...)
  - u64:       must be at address multiple of 8 (0x1000, 0x1008, 0x1010, ...)
  - &T, *T:    must be at address multiple of 8 (0x1000, 0x1008, 0x1010, ...)
  - struct:    alignment = largest alignment of its fields

INDIVIDUAL VARIABLES IN STACK:
  When you have variables, each one is aligned to a full word to optimize access:
  let a: bool = true;  // 1 byte + 7 padding = 8 bytes (1 word)
  let b: bool = false; // 1 byte + 7 padding = 8 bytes (1 word)
  let c: bool = true;  // 1 byte + 7 padding = 8 bytes (1 word)

  in a struct, they are together and compact:
  struct ThreeBools { a: bool, b: bool, c: bool }
  let s = ThreeBools { a: true, b: false, c: true }; // Total: 3 bytes (not 24)
═════════════════════════════════════════════════════════════════════════════


Word │ Address           │ Byte │ Content        │ Variable
─────┼───────────────────┼──────┼────────────────┼──────────────────────────

bool:
  1  │ 0x7fff...632      │ 0    │ 0x01           │ bool flag = true
     │ 0x7fff...633      │ 1    │ 0x00 (padding) │
     │ 0x7fff...634      │ 2    │ 0x00 (padding) │ [7 bytes of padding
     │ 0x7fff...635      │ 3    │ 0x00 (padding) │  for alignment]
     │ 0x7fff...636      │ 4    │ 0x00 (padding) │
     │ 0x7fff...637      │ 5    │ 0x00 (padding) │
     │ 0x7fff...638      │ 6    │ 0x00 (padding) │
     │ 0x7fff...639      │ 7    │ 0x00 (padding) │
     │                   │      │                │
     │                   │      │ ─── ALIGNMENT 8 ──│ (bool occupies 1, padding 7 → total 1 word)

u32:
  3  │ 0x7fff...648      │ 0    │ 0x0C           │ u32 small_num = 300
     │ 0x7fff...649      │ 1    │ 0x01           │ (little-endian: 0x0000012C)
     │ 0x7fff...650      │ 2    │ 0x00           │
     │ 0x7fff...651      │ 3    │ 0x00           │
     │ 0x7fff...652      │ 4    │ 0x00 (padding) │ [4 bytes padding]
     │ 0x7fff...653      │ 5    │ 0x00 (padding) │
     │ 0x7fff...654      │ 6    │ 0x00 (padding) │
     │ 0x7fff...655      │ 7    │ 0x00 (padding) │

// with OPTIMIZATION: u8 + u32 Compiler optimizes (packs multiple variables)
u8 + u32 packed:
  2  │ 0x7fff...640      │ 0    │ 0xFF           │ u8 byte_val = 255
     │ 0x7fff...641      │ 1    │ 0x00 (padding) │ [3 bytes padding because
     │ 0x7fff...642      │ 2    │ 0x00 (padding) │  u32 needs alignment 4]
     │ 0x7fff...643      │ 3    │ 0x00 (padding) │
     │ 0x7fff...644      │ 4    │ 0x0C           │ u32 small_num = 300 (fits in same word)
     │ 0x7fff...645      │ 5    │ 0x01           │
     │ 0x7fff...646      │ 6    │ 0x00           │
     │ 0x7fff...647      │ 7    │ 0x00           │
     │                   │      │ ─── OPTIMIZATION ──│ 2 variables in 1 word

u64:
  4  │ 0x7fff...656      │ 0    │ 0xFF           │ u64 big_num = u64::MAX
     │ 0x7fff...657      │ 1    │ 0xFF           │ (8 consecutive bytes
     │ 0x7fff...658      │ 2    │ 0xFF           │  without padding)
     │ 0x7fff...659      │ 3    │ 0xFF           │
     │ 0x7fff...660      │ 4    │ 0xFF           │
     │ 0x7fff...661      │ 5    │ 0xFF           │
     │ 0x7fff...662      │ 6    │ 0xFF           │
     │ 0x7fff...663      │ 7    │ 0xFF           │
     │                   │      │                │
     │                   │      │ ─── ALIGNMENT 8 ──│ (u64 occupies exactly 8 bytes = 1 word)

[u32; 2]:
  5  │ 0x7fff...664      │ 0    │ 0x04           │ array[0]: u32 = 16909060
     │ 0x7fff...665      │ 1    │ 0x03           │ (little-endian: 0x01020304)
     │ 0x7fff...666      │ 2    │ 0x02           │
     │ 0x7fff...667      │ 3    │ 0x01           │
     │ 0x7fff...668      │ 4    │ 0x08           │ array[1]: u32 = 84281096
     │ 0x7fff...669      │ 5    │ 0x07           │ (little-endian: 0x05060708)
     │ 0x7fff...670      │ 6    │ 0x06           │
     │ 0x7fff...671      │ 7    │ 0x05           │
     │                   │      │                │
     │                   │      │ ─── ALIGNMENT 4 ──│ (each u32 aligned to 4 = 1 word)

Box<i32>:
  6  │ 0x7fff...672      │ 0    │ 0xCD           │ Box<i32> ptr: 4328730445
     │ 0x7fff...673      │ 1    │ 0xAB           │ (little-endian: 0x000000010203ABCD)
     │ 0x7fff...674      │ 2    │ 0x03           │ NO padding because it starts at
     │ 0x7fff...675      │ 3    │ 0x01           │ multiple of 8 (byte 0 of word 6)
     │ 0x7fff...676      │ 4    │ 0x00           │
     │ 0x7fff...677      │ 5    │ 0x00           │
     │ 0x7fff...678      │ 6    │ 0x00           │
     │ 0x7fff...679      │ 7    │ 0x00           │
     │                   │      │                │
     │                   │      │ ─── ALIGNMENT 8 ──│ (pointer occupies 8 bytes = 1 word)

String:
  7  │ 0x7fff...680      │ 0    │ 0xFF           │ String.ptr
     │ 0x7fff...681      │ 1    │ 0x04           │ 4328766719
     │ 0x7fff...682      │ 2    │ 0x02           │ (little-endian)

     │ 0x7fff...683      │ 3    │ 0x01           │
     │ 0x7fff...684      │ 4    │ 0x00           │
     │ 0x7fff...685      │ 5    │ 0x00           │
     │ 0x7fff...686      │ 6    │ 0x00           │
     │ 0x7fff...687      │ 7    │ 0x00           │
     │                   │      │                │
     │                   │      │ ─── ALIGNMENT 8 ──│ (pointer occupies 8 bytes = 1 word)
  8  │ 0x7fff...688      │ 0    │ 0x05           │ String.len: 5
     │ 0x7fff...689      │ 1    │ 0x00           │ (little-endian: 0x0000000000000005)
     │ 0x7fff...690      │ 2    │ 0x00           │
     │ 0x7fff...691      │ 3    │ 0x00           │
     │ 0x7fff...692      │ 4    │ 0x00           │
     │ 0x7fff...693      │ 5    │ 0x00           │
     │ 0x7fff...694      │ 6    │ 0x00           │
     │ 0x7fff...695      │ 7    │ 0x00           │
     │                   │      │                │
     │                   │      │ ─── ALIGNMENT 8 ──│  (String.len occupies 8 bytes = 1 word)
  9  │ 0x7fff...696      │ 0    │ 0x05           │ String.cap: 5
     │ 0x7fff...697      │ 1    │ 0x00           │ (little-endian: 0x0000000000000005)
     │ 0x7fff...698      │ 2    │ 0x00           │ (3 fields × 8 bytes = 24 bytes = 3 words)
     │ 0x7fff...699      │ 3    │ 0x00           │
     │ 0x7fff...700      │ 4    │ 0x00           │
     │ 0x7fff...701      │ 5    │ 0x00           │
     │ 0x7fff...702      │ 6    │ 0x00           │
     │ 0x7fff...703      │ 7    │ 0x00           │
     │                   │      │                │
     │                   │      │ ─── ALIGNMENT 8 ──│ (String.cap occupies 8 bytes = 1 word)

&bool:
 10  │ 0x7fff...704      │ 0    │ 0x56           │ &bool ref_to_bool
     │ 0x7fff...705      │ 1    │ 0x06           │ Address: 0x7fff0656
     │ 0x7fff...706      │ 2    │ 0x7f           │ (little-endian: 0x00007fff0656)
     │ 0x7fff...707      │ 3    │ 0x00           │
     │ 0x7fff...708      │ 4    │ 0x00           │
     │ 0x7fff...709      │ 5    │ 0x00           │
     │ 0x7fff...710      │ 6    │ 0x00           │
     │ 0x7fff...711      │ 7    │ 0x00           │
     │                   │      │                │
     │                   │      │ ─── ALIGNMENT 8 ──│ (reference occupies 8 bytes = 1 word)

ptr_next:
 11  │ 0x7fff...712      │ 0    │ 0xAB           │ ptr_next: *const str
     │ 0x7fff...713      │ 1    │ 0xCD           │ (little-endian: 0x00007fffABCD)
     │ 0x7fff...714      │ 2    │ 0x7f           │
     │ 0x7fff...715      │ 3    │ 0x00           │
     │ 0x7fff...716      │ 4    │ 0x00           │
     │ 0x7fff...717      │ 5    │ 0x00           │
     │ 0x7fff...718      │ 6    │ 0x00           │
     │ 0x7fff...719      │ 7    │ 0x00           │
     │                   │      │                │
     │                   │      │ ─── ALIGNMENT 8 ──│ (pointer occupies 8 bytes = 1 word)

&T reference:
 12  │ 0x7fff...720      │ 0    │ 0x9A           │ &T ref_to_data
     │ 0x7fff...721      │ 1    │ 0x12           │ Address: 0x7fff129A
     │ 0x7fff...722      │ 2    │ 0x7f           │ (little-endian: 0x00007fff129A)
     │ 0x7fff...723      │ 3    │ 0x00           │
     │ 0x7fff...724      │ 4    │ 0x00           │
     │ 0x7fff...725      │ 5    │ 0x00           │
     │ 0x7fff...726      │ 6    │ 0x00           │
     │ 0x7fff...727      │ 7    │ 0x00           │
     │                   │      │                │
     │                   │      │ ─── ALIGNMENT 8 ──│ (reference occupies 8 bytes = 1 word)

PEFORMANCE - STRUCT PADDING — REORDERING SAVES A WORD
═════════════════════════════════════════════════════════════════════════════

Each field starts at an offset multiple of its alignment.
The struct size is rounded up to a multiple of its largest alignment,
so the next element in an array stays aligned.

A field is loaded by its offset. Padding bytes are not read.
Reordering saves memory only when it removes a whole word (8 bytes).
#[repr(C)] keeps this order. Without it, Rust may reorder fields.

#[repr(C)]
struct Person {
    active: bool, // offset 0,  align 1
    id: u64,      // offset 8,  align 8
    flag: bool,   // offset 16, align 1
}
// [bool 1][pad 7][u64 8][bool 1][pad 7] = 24 bytes = 3 words

┌──────┬──────────┬─────────────────┬──────┬──────────┐
│act(1)│ pad (7)  │     id (8)      │flg(1)│ pad (7)  │
└──────┴──────────┴─────────────────┴──────┴──────────┘
0      1          8                16     17         24

#[repr(C)]
struct PersonOptimized {
    id: u64,      // offset 0, align 8
    active: bool, // offset 8, align 1
    flag: bool,   // offset 9, align 1
}
// [u64 8][bool 1][bool 1][pad 6] = 16 bytes = 2 words

┌─────────────────┬──────┬──────┬────────────┐
│     id (8)      │act(1)│flg(1)│  pad (6)   │
└─────────────────┴──────┴──────┴────────────┘
0                8      9     10            16

A u64 at a multiple of 8 is one load. One byte later, the CPU
does two loads and merges them. Put the largest alignment first:
pointer / u64, then u32, u16, and u8 / bool last.

*/

/*
═════════════════════════════════════════════════════════════════════════════
HEAP
═════════════════════════════════════════════════════════════════════════════


STACK (Contiguous, ordered):
┌──────┬──────┬──────┬──────┬──────┐
│ var1 │ var2 │ var3 │ var4 │ var5 │ ← Variables attached
└──────┴──────┴──────┴──────┴──────┘
0x1000 0x1008 0x1010 0x1018 0x1020  ← Consecutive addresses

HEAP (Scattered, fragmented):
┌──────┐         ┌──────┐               ┌──────┐
│ Box1 │         │ Vec1 │               │ Str1 │
└──────┘         └──────┘               └──────┘
0x2000           0x4000                 0x7000
       ↓ 8 KB ↑          ↓ 12 KB ↑             ↓ 4 KB ↑
              [FREE]              [FREE]              [FREE]
*/
#[allow(dead_code)]
#[allow(unused_variables)]
fn example() {
    // STACK: contiguous variables
    let a: u64 = 100; // address: 0x7fff0100
    let b: u32 = 200; // address: 0x7fff0108 (8 bytes later)
    let c: bool = true; // address: 0x7fff010C (4 bytes later)

    // HEAP: scattered
    let box1 = Box::new(42); // heap @ 0x1234567 (random)
    let string = String::from("hello"); // heap @ 0xABCDEF (random, far)
    let vec = vec![1, 2, 3]; // heap @ 0x98765 (random, between the previous ones)

    // The heap allocator chooses where to put each data
    // depending on what memory is available
}

/*
Example of fragmentation and allocation:

Initial (empty):
┌──────────────────────────────────┐
│ [FREE]                           │
└──────────────────────────────────┘

After: let a = Box::new(10); (4 bytes)
┌─────┬──────────────────────────┐
│ [a] │ [FREE]                   │
└─────┴──────────────────────────┘

After: let b = Box::new(20); (4 bytes)
┌─────┬─────┬──────────────────────┐
│ [a] │ [b] │ [FREE]               │
└─────┴─────┴──────────────────────┘

After: drop(a); (frees 'a')
┌─────┬─────┬──────────────────────┐
│[FREE]│ [b] │ [FREE]              │
└─────┴─────┴──────────────────────┘
      ↑ Gap available for new memory

After: let c = Box::new(30); (4 bytes)
┌─────┬─────┬──────────────────────┐
│ [c] │ [b] │ [FREE]               │
└─────┴─────┴──────────────────────┘
      ↑ 'c' occupies the previous gap


The heap uses an allocator (memory manager) that:

Keeps track of free and used blocks
When you request memory, it searches for the first free block large enough
There can be "gaps" if you freed previous memory
New memory goes to the first available gap (first-fit) or the best gap (best-fit)
*/

/*
═════════════════════════════════════════════════════════════════════════════
MEMORY PREALLOCATION
═════════════════════════════════════════════════════════════════════════════

There are 3 scenarios where memory is preallocated:

1. STATIC ARRAYS [T; N] - Compiletime, Stack
   ───────────────────────────────────────────────

   let arr: [u32; 5] = [1, 2, 3, 4, 5];

   ┌─────┬─────┬─────┬─────┬─────┐
   │  1  │  2  │  3  │  4  │  5  │  ← 5 elements always
   └─────┴─────┴─────┴─────┴─────┘
   Address: 0x7fff0100 (stack)
   Size: 5 × 4 bytes = 20 bytes (fixed, known at compiletime)

   CHARACTERISTICS:
   - FIXED size at compiletime [u32; 5] is always 5 elements
   - Location: STACK (contiguous, fast)
   - No dynamic padding
   - No "capacity" vs "length" (both are 5)


2. VEC WITHOUT PREALLOCATION - Runtime, Heap (grows on demand)
   ────────────────────────────────────────────────────────────

   let mut vec = Vec::new();  // capacity = 0, length = 0
   vec.push(10);              // capacity = 1, length = 1
   vec.push(20);              // capacity = 2, length = 2
   vec.push(30);              // capacity = 4, length = 3 ← REALLOCATION

   STEP 1 (after new):
   Stack:
   ┌───────────────────┐
   │ ptr: null         │  ← no heap allocated
   │ len: 0            │
   │ cap: 0            │
   └───────────────────┘

   STEP 2 (after push(10)):
   Stack:
   ┌───────────────────┐
   │ ptr: 0x1000       │  ─────┐
   │ len: 1            │       │
   │ cap: 1            │       │
   └───────────────────┘       │
                               ↓ Heap (0x1000):
                               ┌──────┐
                               │ [10] │  1 element allocated
                               └──────┘

   STEP 3 (after push(20)):
   Stack:
   ┌───────────────────┐
   │ ptr: 0x1000       │  ─────┐
   │ len: 2            │       │
   │ cap: 2            │       │
   └───────────────────┘       │
                               ↓ Heap (0x1000):
                               ┌──────┬──────┐
                               │ [10] │ [20] │  2 elements allocated
                               └──────┴──────┘

   STEP 4 (after push(30)):
   The vec needs to grow: capacity < length
   → REALLOCATE to new block (0x2000):

   Stack:
   ┌───────────────────┐
   │ ptr: 0x2000       │  ─────┐ (changed!)
   │ len: 3            │       │
   │ cap: 4            │       │  IMPORTANT:
   └───────────────────┘       │  - Copies data from 0x1000 to 0x2000
                               │  - Frees 0x1000
   OLD (0x1000):              │  - New block for 4 elements
   ┌──────┬──────┐            │
   │[10]  │ [20] │ [FREED]    │
   └──────┴──────┘            │
                               ↓ Heap (0x2000):
                               ┌──────┬──────┬──────┬──────┐
                               │ [10] │ [20] │ [30] │ [??] │  4 elements capacity
                               └──────┴──────┴──────┴──────┘

   PROBLEM: Each reallocation is expensive (copies everything + frees previous)


3. VEC WITH PREALLOCATION - Runtime, Heap (avoids reallocations)
   ─────────────────────────────────────────────────────────────

   let mut vec = Vec::with_capacity(10);
   // capacity = 10, length = 0 (PREALLOCATED)

   Stack:
   ┌───────────────────┐
   │ ptr: 0x1000       │  ─────┐
   │ len: 0            │       │
   │ cap: 10           │       │
   └───────────────────┘       │
                               ↓ Heap (0x1000):
                               ┌────────────────────────────┐
                               │ [??] [??] [??] ... [??]    │  10 elements UNINITIALIZED
                               │ len=0       capacity=10    │
                               └────────────────────────────┘

   NOW we can push without reallocations:

   vec.push(10);
   vec.push(20);
   vec.push(30);

   Stack:
   ┌───────────────────┐
   │ ptr: 0x1000       │  ─────┐
   │ len: 3            │       │
   │ cap: 10           │       │
   └───────────────────┘       │
                               ↓ Heap (0x1000):
                               ┌────────────────────────────┐
                               │ [10] [20] [30] [??] ... [??]
                               │ len=3       capacity=10    │
                               └────────────────────────────┘

   ✓ NO reallocations (space was already reserved)
   ✓ Data in order
   ✓ Only 3/10 of preallocated memory used


4. STRING WITH PREALLOCATION - Runtime, Heap
   ──────────────────────────────────────────

   let mut s = String::with_capacity(100);
   // capacity = 100 bytes, length = 0 (empty string)

   Stack:
   ┌───────────────────┐
   │ ptr: 0x3000       │  ─────┐
   │ len: 0            │       │
   │ cap: 100          │       │
   └───────────────────┘       │
                               ↓ Heap (0x3000):
                               ┌──────────────────────────────┐
                               │ [??] [??] ... [??]           │  100 bytes UNINITIALIZED
                               │ len=0       capacity=100      │
                               └──────────────────────────────┘

   s.push_str("Hello");  // len = 5
   s.push_str(" World"); // len = 11

   Stack:
   ┌───────────────────┐
   │ ptr: 0x3000       │  ─────┐
   │ len: 11           │       │
   │ cap: 100          │       │
   └───────────────────┘       │
                               ↓ Heap (0x3000):
                               ┌──────────────────────────────┐
                               │ H e l l o   W o r l d [??] ... │  11 bytes used
                               │ len=11      capacity=100      │
                               └──────────────────────────────┘

   ✓ No reallocations
   ✓ Space reserved to grow
   ✓ Only 11 of 100 bytes used


DIFFERENCE: CAPACITY vs LENGTH
═════════════════════════════════════════════════════════════════════════════

Vec<T>:

  length (len):     How many INITIALIZED elements you have
                    Only these are accessible: vec[0..len]

  capacity (cap):   How much TOTAL memory is allocated in heap
                    Can be >= length
                    Reserve to grow without reallocation

┌─────────────────────────────────────────┐
│ Vec { ptr, len, cap }                   │
└─────────────────────────────────────────┘

Example:
  let mut vec: Vec<u32> = Vec::with_capacity(100);
  vec.push(42);

  vec.len()      // 1 (one initialized element)
  vec.capacity() // 100 (total preallocated space)

  println!("{:?}", vec);     // [42]
  println!("{}", vec[0]);    // 42 ✓
  println!("{}", vec[1]);    // ✗ PANIC (out of bounds)
                             // length is 1, you cannot access vec[1]


USED MEMORY vs ALLOCATED MEMORY
═════════════════════════════════════════════════════════════════════════════

Without preallocation:
  let mut vec = Vec::new();
  vec.push(1); vec.push(2); vec.push(3); vec.push(4); vec.push(5);

  Reallocations: 0→1→2→4→8
  TOTAL ALLOCATED MEMORY (over time): 1+2+4+8 = 15 bytes
  FINAL USED MEMORY: 5×4 = 20 bytes (in the 32-byte block)
  WASTE: 12 bytes (32-20)

With preallocation:
  let mut vec = Vec::with_capacity(10);
  vec.push(1); vec.push(2); vec.push(3); vec.push(4); vec.push(5);

  Reallocations: 0
  ALLOCATED MEMORY: 10×4 = 40 bytes (once)
  USED MEMORY: 5×4 = 20 bytes
  WASTE: 20 bytes (40-20)

  ✓ But ZERO reallocations (much faster)


USE CASES
═════════════════════════════════════════════════════════════════════════════

USE Vec::with_capacity() WHEN:
  ✓ You know approximately how much data you expect
  ✓ Reallocation is expensive (lots of data)
  ✓ You want predictable performance
  ✓ Loop that will do many push()

  Example:
    let mut vec = Vec::with_capacity(1_000_000);
    for i in 0..1_000_000 {
      vec.push(i);  // No reallocations
    }

USE Vec::new() WHEN:
  ✓ You don't know how much data you'll have
  ✓ Data is small
  ✓ Performance is not critical

  Example:
    let mut errors = Vec::new();
    if condition { errors.push("error1"); }
    if other_condition { errors.push("error2"); }

*/
