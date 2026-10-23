#![allow(dead_code)]
#![allow(unused_variables)]

#[test]
fn indice() {
    value_ownership::value_ownership();

    id_reference::id_reference();

    rc_shared::rc_shared();

    arc_shared::arc_shared();

    lifetimes_ref::lifetimes_ref();

    weak_references::weak_references();

    arena_allocation::arena_allocation();

    ecs_pattern::ecs_hashmap::ecs_pattern();
    ecs_pattern::ecs_contiguous::ecs_contiguous_demo();
    ecs_pattern::ecs_sparse_set::test_sparse_set();
}

/*
========================================================================
RELATED ENTITIES: id vs value vs Rc vs Arc
========================================================================

    How to handle entities that can belong to multiple owners?

    SCENARIO: A Product can be in multiple places:
    --------------------------------------------
        - In a Cart
        - In an Order
        - In a Wishlist

    How do we model this relationship?
*/

#[derive(Debug, Clone)]
struct Product {
    id: u64,
    name: String,
    price: f64,
}

impl Product {
    fn new(id: u64, name: &str, price: f64) -> Self {
        Product {
            id,
            name: name.to_string(),
            price,
        }
    }
}

/*
========================================================================
VALUE_OWNERSHIP
========================================================================

    VALUE OWNERSHIP (Complete copy):
    --------------------------------------------
        Each entity OWNS its own copy of the product.
        Simple but duplicates data in memory.

        DIAGRAM:
        ┌─────────────────────────────────────────────────────────────┐
        │  (stack)                (heap 1)              (heap 2)      │
        │  product (original)     cart.items[0]       order.items[0]   │
        │  ┌──────────────┐      ┌──────────────┐   ┌──────────────┐  │
        │  │ id: 1        │      │ id: 1        │   │ id: 1        │  │
        │  │ name: ───────┼──┐   │ name: ───────┼─┐ │ name: ───────┼┐ │
        │  │ price: 999.99│  │   │ price: 999.99│ │ │ price: 999.99││ │
        │  └──────────────┘  │   └──────────────┘ │ └──────────────┘│ │
        │                    ▼                    ▼                 ▼ │
        │              ┌────────┐          ┌────────┐         ┌────────┐
        │              │"Laptop"│          │"Laptop"│         │"Laptop"│
        │              └────────┘          └────────┘         └────────┘
        │              (heap 3)            (heap 4)           (heap 5) │
        │                                                             │
        │  → 3 independent copies in memory                           │
        └─────────────────────────────────────────────────────────────┘

    CHARACTERISTICS:
    --------------------------------------------
        ✓ Advantages: Simple, no lifetimes, each entity is independent
        ✗ Disadvantages: Duplicates memory, changes don't propagate
*/
#[cfg(test)]
mod value_ownership {
    use super::Product;

    #[derive(Debug, Clone)]
    pub struct CartItem {
        pub product: Product, // Ownership: Cart OWNS the product
        pub quantity: u32,
    }

    #[derive(Debug, Clone)]
    pub struct OrderItem {
        pub product: Product, // Ownership: Order OWNS the product
        pub quantity: u32,
    }

    #[derive(Debug)]
    pub struct Cart {
        pub items: Vec<CartItem>,
    }

    #[derive(Debug)]
    pub struct Order {
        pub id: u64,
        pub items: Vec<OrderItem>,
    }

    #[test]
    pub fn value_ownership() {
        let product = Product::new(1, "Laptop", 999.99);

        // Each entity has its OWN COPY
        let cart = Cart {
            items: vec![CartItem {
                product: product.clone(), // COPY
                quantity: 1,
            }],
        };

        let order = Order {
            id: 1,
            items: vec![OrderItem {
                product: product.clone(), // ANOTHER COPY
                quantity: 1,
            }],
        };

        println!("  Original product: {:?}", product);
        println!("  In Cart: {:?}", cart.items[0].product.name);
        println!("  In Order: {:?}", order.items[0].product.name);

        // Verify that they are independent copies
        let ptr_orig = product.name.as_ptr();
        let ptr_cart = cart.items[0].product.name.as_ptr();
        let ptr_order = order.items[0].product.name.as_ptr();

        println!("\n  Memory addresses (String heap):");
        println!("    Original: {:p}", ptr_orig);
        println!("    Cart:     {:p}", ptr_cart);
        println!("    Order:    {:p}", ptr_order);
        println!(
            "    Are they different? {}",
            ptr_orig != ptr_cart && ptr_cart != ptr_order
        );

        println!("  ✅ value_ownership::value_ownership");
    }
}

/*
========================================================================
ID_REFERENCE
========================================================================

    ID REFERENCE (Store only IDs):
    --------------------------------------------
        Entities only store the product ID.
        Products live in a central repository.

        DIAGRAM:
        ┌─────────────────────────────────────────────────────────────┐
        │                                                             │
        │  cart.items[0]       order.items[0]      ProductRepository  │
        │  ┌────────────┐     ┌────────────┐      ┌────────────────┐  │
        │  │ product_id:│     │ product_id:│      │ 1 → Product    │  │
        │  │     1  ────┼─────┼────────────┼─────▶│    id: 1       │  │
        │  │ quantity: 1│     │     1      │      │    name:"Laptop"  │
        │  └────────────┘     └────────────┘      │    price: 899.99  │
        │                                         ├────────────────┤  │
        │  cart.items[1]                          │ 2 → Product    │  │
        │  ┌────────────┐                         │    id: 2       │  │
        │  │ product_id:│                         │    name:"Mouse"│  │
        │  │     2  ────┼────────────────────────▶│    price: 29.99│  │
        │  │ quantity: 2│                         └────────────────┘  │
        │  └────────────┘                                             │
        │                                                             │
        │  → Only one Product in memory, multiple ID references       │
        └─────────────────────────────────────────────────────────────┘

    CHARACTERISTICS:
    --------------------------------------------
        ✓ Advantages: Memory efficient, changes propagate, serializable
        ✗ Disadvantages: Lookup on each access, repo must live longer
*/
#[cfg(test)]
mod id_reference {
    use super::Product;
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    pub struct CartItem {
        pub product_id: u64, // Only the ID, not the object
        pub quantity: u32,
    }

    #[derive(Debug, Clone)]
    pub struct OrderItem {
        pub product_id: u64,
        pub quantity: u32,
    }

    #[derive(Debug)]
    pub struct Cart {
        pub items: Vec<CartItem>,
    }

    #[derive(Debug)]
    pub struct Order {
        pub id: u64,
        pub items: Vec<OrderItem>,
    }

    // Central repository that OWNS the products
    pub struct ProductRepository {
        products: HashMap<u64, Product>,
    }

    impl ProductRepository {
        pub fn new() -> Self {
            ProductRepository {
                products: HashMap::new(),
            }
        }

        pub fn add(&mut self, product: Product) {
            self.products.insert(product.id, product);
        }

        pub fn get(&self, id: u64) -> Option<&Product> {
            self.products.get(&id)
        }

        pub fn update_price(&mut self, id: u64, new_price: f64) {
            if let Some(product) = self.products.get_mut(&id) {
                product.price = new_price;
            }
        }
    }

    #[test]
    pub fn id_reference() {
        let product1 = Product::new(1, "Laptop", 999.99);
        let product2 = Product::new(2, "Mouse", 29.99);

        let mut repo = ProductRepository::new();
        repo.add(product1.clone());
        repo.add(product2.clone());

        // Cart and Order only store IDs
        let cart = Cart {
            items: vec![
                CartItem {
                    product_id: product1.id,
                    quantity: 1,
                },
                CartItem {
                    product_id: product2.id,
                    quantity: 2,
                },
            ],
        };

        let order = Order {
            id: 1,
            items: vec![OrderItem {
                product_id: product1.id,
                quantity: 1,
            }],
        };

        // To access the product, we search in the repo
        println!("  Cart items:");
        for item in &cart.items {
            if let Some(product) = repo.get(item.product_id) {
                println!(
                    "    - {} x{} = ${:.2}",
                    product.name, item.quantity, product.price
                );
            }
        }

        println!("\n  Order items:");
        for item in &order.items {
            if let Some(product) = repo.get(item.product_id) {
                println!(
                    "    - {} x{} = ${:.2}",
                    product.name, item.quantity, product.price
                );
            }
        }

        // Changing price in repo affects everyone
        println!("\n  → Updating Laptop price to $899.99...");
        repo.update_price(1, 899.99);

        println!("  Now in Cart:");
        if let Some(product) = repo.get(1) {
            println!("    - {} = ${:.2} (updated!)", product.name, product.price);
        }

        println!("  ✅ id_reference::id_reference");
    }
}

/*
========================================================================
RC_SHARED
========================================================================

    Rc (Reference Counted) - Single-thread:
    --------------------------------------------
        Multiple owners of the same object in memory.
        Reference counter, frees when it reaches 0.

        DIAGRAM:
        ┌─────────────────────────────────────────────────────────────┐
        │                                                             │
        │  product              cart.items[0]        order.items[0]   │
        │  ┌─────┐              ┌─────┐              ┌─────┐          │
        │  │ Rc  │              │ Rc  │              │ Rc  │          │
        │  │ ptr─┼──────┐       │ ptr─┼──────┐       │ ptr─┼──────┐   │
        │  └─────┘      │       └─────┘      │       └─────┘      │   │
        │               │                    │                    │   │
        │               ▼                    ▼                    ▼   │
        │            ┌─────────────────────────────────────────────┐  │
        │            │              RefCell<Product>               │  │
        │            │  ┌─────────────────────────────────────┐    │  │
        │            │  │ strong_count: 3                     │    │  │
        │            │  │ ┌─────────────────────────────────┐ │    │  │
        │            │  │ │ Product                         │ │    │  │
        │            │  │ │   id: 1                         │ │    │  │
        │            │  │ │   name: "Laptop" ──▶ [heap]     │ │    │  │
        │            │  │ │   price: 899.99                 │ │    │  │
        │            │  │ └─────────────────────────────────┘ │    │  │
        │            │  └─────────────────────────────────────┘    │  │
        │            └─────────────────────────────────────────────┘  │
        │                                                             │
        │  → ONLY ONE object in memory, 3 counted references         │
        │  → RefCell enables interior mutability (runtime borrow check)
        └─────────────────────────────────────────────────────────────┘

    CHARACTERISTICS:
    --------------------------------------------
        ✓ Advantages: One object, multiple owners, changes visible to all
        ✗ Disadvantages: Single-thread only, counter overhead, not serializable

    Rc<RefCell<T>>:
    --------------------------------------------
        Rc only gives you immutable references (&T)
        RefCell enables interior mutability (runtime borrow check) even with
        immutable references (&T). Without this combination, product would be readonly.

    SAFETY:
    --------------------------------------------
        Rc<RefCell<T>> is not multi-thread safe:
        - RefCell is not Sync.
        - Rc is neither Send nor Sync.
*/
#[cfg(test)]
mod rc_shared {
    use super::Product;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug, Clone)]
    pub struct CartItem {
        pub product: Rc<RefCell<Product>>, // Shared ownership + mutability
        pub quantity: u32,
    }

    #[derive(Debug, Clone)]
    pub struct OrderItem {
        pub product: Rc<RefCell<Product>>,
        pub quantity: u32,
    }

    #[derive(Debug)]
    pub struct Cart {
        pub items: Vec<CartItem>,
    }

    #[derive(Debug)]
    pub struct Order {
        pub id: u64,
        pub items: Vec<OrderItem>,
    }

    #[test]
    pub fn rc_shared() {
        // Create product with Rc<RefCell<T>>
        let product = Rc::new(RefCell::new(Product::new(1, "Laptop", 999.99)));

        println!("  Rc strong_count initial: {}", Rc::strong_count(&product));

        // Cart and Order share the SAME product
        let cart = Cart {
            items: vec![CartItem {
                product: Rc::clone(&product), // Increments counter, no data copy
                quantity: 1,
            }],
        };

        println!(
            "  Rc strong_count after Cart: {}",
            Rc::strong_count(&product)
        );

        let order = Order {
            id: 1,
            items: vec![OrderItem {
                product: Rc::clone(&product), // Increments counter
                quantity: 1,
            }],
        };

        println!(
            "  Rc strong_count after Order: {}",
            Rc::strong_count(&product)
        );

        // Verify that it's the SAME object
        println!("\n  Memory addresses:");
        println!("    product:           {:p}", Rc::as_ptr(&product));
        println!(
            "    cart.items[0]:     {:p}",
            Rc::as_ptr(&cart.items[0].product)
        );
        println!(
            "    order.items[0]:    {:p}",
            Rc::as_ptr(&order.items[0].product)
        );
        println!(
            "    Are they the same? {}",
            Rc::ptr_eq(&product, &cart.items[0].product)
        );

        // Modify from anywhere affects everyone
        println!("\n  Original price: ${:.2}", product.borrow().price);
        product.borrow_mut().price = 899.99;
        println!("  Modified price: ${:.2}", product.borrow().price);
        println!(
            "  Seen from Cart: ${:.2}",
            cart.items[0].product.borrow().price
        );
        println!(
            "  Seen from Order: ${:.2}",
            order.items[0].product.borrow().price
        );

        println!("  ✅ rc_shared::rc_shared");
    }
}

/*
========================================================================
ARC_SHARED
========================================================================

    Arc (Atomic Reference Counted) - Multi-thread:
    --------------------------------------------
        Like Rc but thread-safe. Uses atomic operations.

        DIAGRAM:
        ┌─────────────────────────────────────────────────────────────┐
        │                                                             │
        │  Thread 1              Thread 2              Thread 3       │
        │  ┌─────┐              ┌─────┐              ┌─────┐          │
        │  │ Arc │              │ Arc │              │ Arc │          │
        │  │ ptr─┼──────┐       │ ptr─┼──────┐       │ ptr─┼──────┐   │
        │  └─────┘      │       └─────┘      │       └─────┘      │   │
        │               │                    │                    │   │
        │               ▼                    ▼                    ▼   │
        │            ┌─────────────────────────────────────────────┐  │
        │            │              RwLock<Product>                │  │
        │            │  ┌─────────────────────────────────────┐    │  │
        │            │  │ atomic_count: 3                     │    │  │
        │            │  │ ┌─────────────────────────────────┐ │    │  │
        │            │  │ │ Product (protected by RwLock)   │ │    │  │
        │            │  │ │   id: 1                         │ │    │  │
        │            │  │ │   name: "Laptop"                │ │    │  │
        │            │  │ │   price: 849.99                 │ │    │  │
        │            │  │ └─────────────────────────────────┘ │    │  │
        │            │  └─────────────────────────────────────┘    │  │
        │            └─────────────────────────────────────────────┘  │
        │                                                             │
        │  → Thread-safe: multiple threads can access                │
        │  → RwLock: multiple readers OR one writer                  │
        │  → Atomic operations (more overhead than Rc)               │
        └─────────────────────────────────────────────────────────────┘

    CHARACTERISTICS:
    --------------------------------------------
        ✓ Advantages: Thread-safe, one object, multiple owners
        ✗ Disadvantages: Atomic overhead, locks, not serializable

    COMPARISON: Rc<RefCell<T>> vs Arc<RwLock<T>>:
    --------------------------------------------
        1. The cost of "Atomicity" (Arc vs Rc):
           Arc must use atomic CPU instructions (slower).

        2. The cost of "Locking" (RwLock vs RefCell):
           RwLock interacts with the OS (syscalls), RefCell is nearly instant.

        3. Clarity of Intent:
           Rc indicates thread-local object, Arc indicates concurrency.
*/
#[cfg(test)]
mod arc_shared {
    use super::Product;
    use std::sync::{Arc, RwLock};

    #[derive(Debug, Clone)]
    pub struct CartItem {
        pub product: Arc<RwLock<Product>>, // Thread-safe shared ownership
        pub quantity: u32,
    }

    #[derive(Debug, Clone)]
    pub struct OrderItem {
        pub product: Arc<RwLock<Product>>,
        pub quantity: u32,
    }

    #[derive(Debug)]
    pub struct Cart {
        pub items: Vec<CartItem>,
    }

    #[derive(Debug)]
    pub struct Order {
        pub id: u64,
        pub items: Vec<OrderItem>,
    }

    #[test]
    pub fn arc_shared() {
        // Create product with Arc<RwLock<T>>
        let product = Arc::new(RwLock::new(Product::new(1, "Laptop", 999.99)));

        println!(
            "  Arc strong_count initial: {}",
            Arc::strong_count(&product)
        );

        let cart = Cart {
            items: vec![CartItem {
                product: Arc::clone(&product),
                quantity: 1,
            }],
        };

        let order = Order {
            id: 1,
            items: vec![OrderItem {
                product: Arc::clone(&product),
                quantity: 1,
            }],
        };

        println!(
            "  Arc strong_count after Cart and Order: {}",
            Arc::strong_count(&product)
        );

        // Simulate access from multiple threads
        let product_for_thread = Arc::clone(&product);

        let handle = std::thread::spawn(move || {
            // Modify price from another thread
            let mut p = product_for_thread.write().unwrap();
            p.price = 849.99;
            println!("  [Thread] Price modified to: ${:.2}", p.price);
        });

        handle.join().unwrap();

        // See change from main thread
        println!("  [Main] Price seen: ${:.2}", product.read().unwrap().price);
        println!(
            "  [Cart] Price seen: ${:.2}",
            cart.items[0].product.read().unwrap().price
        );

        println!("  ✅ arc_shared::arc_shared");
    }
}
/*
fsfjks fjskfjskfjksj fskjfks jfskjsd'dfdksdsfks dsfs
*/
/*
========================================================================
LIFETIMES_REF: REFERENCES WITH LIFETIMES (&'a T)
========================================================================

    The most "pure" form of Rust. No reference counters or IDs.
    The compiler guarantees the owner lives longer than the reference.

    ✓ Advantages: Zero-cost, maximum performance.
    ✗ Disadvantages: Very difficult to manage in complex graphs or structures
       that must live for a long time (the owner must outlive everyone).
*/

#[cfg(test)]
mod lifetimes_ref {
    use super::Product;

    pub struct CartItem<'a> {
        pub product: &'a Product,
        pub quantity: u32,
    }

    #[test]
    pub fn lifetimes_ref() {
        let product = Product::new(1, "Laptop", 999.99);

        // The CartItem only lives as long as 'product'
        let item = CartItem {
            product: &product,
            quantity: 1,
        };

        println!(
            "  [Lifetimes] Product: {}, Price: {}",
            item.product.name, item.product.price
        );
        assert_eq!(item.product.id, 1);
    }
}

/*
========================================================================
WEAK_REFERENCES: WEAK REFERENCES (Weak<T>)
========================================================================

    Weak tries to create an Rc::clone or Arc::clone on demand
    while its content (the interior of rc or arc) exists in memory.

    If the original object dies, the weak reference can no longer be
    "upgraded" to Rc/Arc.

    ✓ Advantages:
        Caches: You can have a Weak in a cache. If no one else is using the object
        (the Arc died), the cache shouldn't keep it alive artificially.

        Circular Structures in Threads: If you have two services that need each other
        but run in different threads, one must use Weak so the system can shut down
        correctly without waiting infinitely for the other.

    ✗ Disadvantages: You must "try" to (upgrade) convert it to Rc/Arc before
       using it, since the object might have been freed.
*/

#[cfg(test)]
mod weak_references {
    use super::Product;
    use std::cell::RefCell;
    use std::rc::{Rc, Weak};

    #[test]
    pub fn weak_references() {
        let product = Rc::new(RefCell::new(Product::new(1, "Laptop", 999.99)));

        // Create a weak reference
        let weak_product: Weak<RefCell<Product>> = Rc::downgrade(&product);

        // To use it, we must try to "upgrade" it to Rc
        if let Some(strong_product) = weak_product.upgrade() {
            println!(
                "  [Weak] Product recovered: {}",
                strong_product.borrow().name
            );
        }

        // If the original dies...
        drop(product);

        if weak_product.upgrade().is_none() {
            println!("  [Weak] Product no longer exists (we avoid dangling pointers)");
        }
    }
}

/*
========================================================================
ARENA_ALLOCATION: ARENA ALLOCATION
========================================================================

    Large blocks of memory are reserved where all objects live.
    The Arena owns everything and lends us references.

    ✓ Advantages: Extremely fast, allows using simple references (&T)
       because all objects die at the same time as the "Arena".
    ✗ Disadvantages: You can't easily free individual objects.
*/

#[cfg(test)]
mod arena_allocation {
    // Nota: En producción usarías crates como 'bumpalo' o 'typed-arena'
    use super::Product;
    use std::cell::RefCell;

    pub struct Arena {
        // We use RefCell to allow 'alloc' with &self
        products: RefCell<Vec<Product>>,
    }

    impl Arena {
        // In a real arena, this would return a reference that lives as long as the arena
        // Here, to simplify, we return the ID or just show the concept.
        // But for it to compile with &T, we need the Vec to not move.
        fn alloc(&self, p: Product) {
            self.products.borrow_mut().push(p);
        }

        fn get_all(&self) {
            for p in self.products.borrow().iter() {
                println!("  [Arena] Product: {}", p.name);
            }
        }
    }

    #[test]
    pub fn arena_allocation() {
        let arena = Arena {
            products: RefCell::new(Vec::new()),
        };

        // The arena owns the data
        arena.alloc(Product::new(1, "Laptop", 999.99));
        arena.alloc(Product::new(2, "Mouse", 25.00));

        arena.get_all();
        assert_eq!(arena.products.borrow().len(), 2);
    }
}

/*
========================================================================
ECS_PATTERN: ENTITY COMPONENT SYSTEM
========================================================================

    Data-Oriented (data-focused) rather than object-oriented.

    Architecture where entities are just a number (ID), and data
    (Components) lives in contiguous memory arrays.

    TRADITIONAL (AoS):
        [ ID, Name, Price ] [ ID, Name, Price ] [ ID, Name, Price ]
            ^       ^       ^
            └───────┴───────┴── The CPU loads all this even if it only wants the price.

    ECS (SoA):
        Names: [ Name ] [ Name ] [ Name ]
        Prices: [ Price ] [ Price ] [ Price ]  <-- The CPU flies through this.

    You can process thousands of prices in a single clock cycle
    because they're stuck together in memory.

    ✓ Advantages: Cache-friendly, total decoupling, very scalable.
    ✗ Disadvantages: High learning curve, more complex architecture.
*/

#[cfg(test)]
mod ecs_pattern {

    /*
    ========================================================================
    ECS: HASHMAP IMPLEMENTATION
    ========================================================================

    Structure:
        let mut names: HashMap<u32, String> = HashMap::new();

    Deleting an item from hashmap:

        When you do hashmap.remove(&id), the entry disappears. If you iterate the HashMap,
        that element simply isn't there. You don't have to deal with Option::None like
        in Vec<Option<T>>.

        Internally, a HashMap is an array of "buckets". When you delete something:

        The HashMap marks that space as "empty" or uses a "tombstone" (a mark that
        says "something was here").
        The physical "hole" exists in RAM, but the HashMap manages it for you.
        The performance problem:
        Although the HashMap hides the hole, the data is still scattered. The CPU can't
        predict where the next element is because hashes are random. You jump from one
        memory address to another (this is called Pointer Chasing), which empties
        the CPU cache.
     */
    pub mod ecs_hashmap {
        use std::collections::HashMap;

        #[test]
        pub fn ecs_pattern() {
            // Entities are just IDs
            let entity_id = 100;

            // Components live in separate storages
            let mut names: HashMap<u32, String> = HashMap::new();
            let mut prices: HashMap<u32, f64> = HashMap::new();

            names.insert(entity_id, "Laptop".to_string());
            prices.insert(entity_id, 999.99);

            // We access the data by ID
            if let (Some(name), Some(price)) = (names.get(&entity_id), prices.get(&entity_id)) {
                println!("  [ECS] Entity {}: {} costs ${}", entity_id, name, price);
            }

            assert_eq!(names.len(), 1);
        }
    }

    /*
    ========================================================================
    ECS_CONTIGUOUS: VECTOR IMPLEMENTATION (SoA)
    ========================================================================

        Internal Structure:

            names: Vec<Option<String>>,

        Here components live in Vecs. The entity ID is the index.
        It's the fastest form for the CPU because data is stuck together.

        No Hashing:
        No mathematical function needs to be calculated to find the data. If you want the
        price of entity 5, you go directly to prices[5].

        CPU Prefetching:
        When the CPU sees you're iterating through a Vec<f64>, the hardware "guesses"
         you'll need the next number and brings it from RAM to Cache before you ask for it.

        Data Density:
        In a HashMap, there are empty spaces (buckets) and metadata. In a Vec<f64>, every
         byte of cache is full of useful information.

        Deletion with Holes:
        If you delete an entity, you can leave a None in its place. The CPU simply skips
        that space when iterating.
    */

    pub mod ecs_contiguous {
        // Instead of using object, one single World struct with Vecs for components
        // with data from all the entities
        pub struct World {
            // Each index is an entity. Option allows not all to have everything.
            names: Vec<Option<String>>,
            prices: Vec<Option<f64>>,
        }

        impl World {
            fn new() -> Self {
                Self {
                    names: Vec::new(),
                    prices: Vec::new(),
                }
            }

            fn spawn(&mut self, name: &str, price: f64) -> usize {
                let id = self.names.len();
                self.names.push(Some(name.to_string()));
                self.prices.push(Some(price));
                id
            }
        }

        #[test]
        pub fn ecs_contiguous_demo() {
            let mut world = World::new();
            world.spawn("Laptop", 999.99);
            world.spawn("Mouse", 25.00);
            world.spawn("Keyboard", 50.00);

            // SYSTEM: Apply 10% discount to EVERYTHING
            // The CPU flies here because it traverses a pure f64 array.
            for price_opt in world.prices.iter_mut() {
                if let Some(price) = price_opt {
                    *price *= 0.9;
                }
            }

            println!("  [ECS Contiguous] Prices updated in bulk.");
            assert_eq!(world.prices[0], Some(899.991));
        }
    }

    /*
    ========================================================================
    ECS_SPARSE_SET: SPARSE SET IMPLEMENTATION
    ========================================================================

        Internal Structure:
            dense: vec<T>
                // with contiguous data [T, T, T, T]
            sparse: vec<isize>
                // with indices in dense for each entity
                // [id1 -> idx_dense, id2 -> idx_dense, ...]
                // is a vector with holes (-1) where there's no data e.g. if id = 1,000,000
                // puts -1 in all empty positions up to 1,000,000
            entities: vec<usize>
                // inverse of sparse
                // [idx_dense -> id1, idx_dense -> id2, ...]
                // also has holes

        Ideal for components that are added/removed frequently.
        When you delete an item, you move the last one to the hole, keeping everything
        stuck together.

        Perfect Iteration:
        The iter() method traverses self.dense. No Option, no if let Some, no jumps.
        The CPU can do SIMD (process several numbers at once).

        O(1) Deletion:
        Whether you have 3 or 3 million elements, deletion always costs the same because
        you only swap two positions and do a pop().

        Cache Locality:
        With no holes, every time the CPU brings a cache line from RAM, 100% of it is
        useful data.

        "Weakness" of Sparse Set: memory consumption in the sparse array:

            The cost is low: An isize (or u32) takes only 4 or 8 bytes. Having a sparse
            array of 1 million elements takes about 4MB or 8MB. For a modern PC, that's
            insignificant compared to the speed you gain.

            Speed vs Memory: You prefer spending 8MB of RAM to have instant O(1) access,
            rather than using a HashMap which is slower and "dirties" the CPU cache.

            IDs are usually contiguous: In most game engines or high-performance systems,
            IDs are recycled. If an entity dies, its ID is saved for the next one,
            keeping numbers as low as possible.

        Solution to Weakness:
            If your application handles very scattered IDs (e.g. 1 and 1,000,000),
            using pagination or sharding to divide the sparse into smaller blocks
            can help reduce memory consumption.

     */
    pub mod ecs_sparse_set {
        pub struct SparseSet<T> {
            dense: Vec<T>,        // Data stuck together (Cache-friendly)
            entities: Vec<usize>, // Entity ID at each dense position
            sparse: Vec<isize>,   // Map: Entity_ID -> Position in dense (-1 if doesn't exist)
        }

        impl<T> SparseSet<T> {
            pub fn new() -> Self {
                Self {
                    dense: Vec::new(),
                    entities: Vec::new(),
                    sparse: Vec::new(),
                }
            }

            pub fn get(&self, entity_id: usize) -> Option<&T> {
                let idx = self.sparse.get(entity_id)?;
                if *idx == -1 {
                    None
                } else {
                    self.dense.get(*idx as usize)
                }
            }

            pub fn insert(&mut self, entity_id: usize, data: T) {
                // Ensure space in sparse array
                if entity_id >= self.sparse.len() {
                    self.sparse.resize(entity_id + 1, -1);
                }
                // Store current index
                self.sparse[entity_id] = self.dense.len() as isize;
                self.dense.push(data);
                self.entities.push(entity_id);
            }

            pub fn remove(&mut self, entity_id: usize) {
                let idx = self.sparse[entity_id] as usize;
                let last_idx = self.dense.len() - 1;

                // 1. Move the last element to the hole of what we deleted
                self.dense.swap(idx, last_idx);
                self.entities.swap(idx, last_idx);

                // 2. Update sparse map for the entity we moved
                let moved_entity = self.entities[idx];
                self.sparse[moved_entity] = idx as isize;

                // 3. Clean up
                self.sparse[entity_id] = -1;
                self.dense.pop();
                self.entities.pop();
            }

            pub fn iter(&self) -> impl Iterator<Item = &T> {
                self.dense.iter() // Pure and simple iteration without Options!
            }
        }

        #[test]
        pub fn test_sparse_set() {
            let mut prices = SparseSet::new();
            prices.insert(0, 100.0);
            prices.insert(1, 200.0);
            prices.insert(2, 300.0);

            prices.remove(1); // Delete the one in the middle

            // The dense array now has [100.0, 300.0] stuck together.
            // The CPU doesn't know there was a deletion, it only sees contiguous data.
            for price in prices.iter() {
                println!("  [SparseSet] Price: {}", price);
            }
        }
    }
}
