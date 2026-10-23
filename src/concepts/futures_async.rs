#[test]
fn index() {
    future_trait::manual_dispatch();
    future_trait::tokio_dispatch();
    futures_in_stack_and_heap::futures_in_stack_and_heap();
    pin_futures::pin_futures();
    async_block::async_block();
    async_block::async_block_move();
    async_fn::async_fn();
    async_closures::async_fn();
    async_closures::async_fn_mut();
    async_closures::async_fn_once();
    not_unpin_and_async::auto_ref_after_await();
    not_unpin_and_async::auto_ref_before_await();
    not_unpin_and_async::capture_move_with_auto_ref();
    not_unpin_and_async::async_fn_no_move();
}

/*
FUTURE TRAIT:
========================================================================

    Concept:
    --------------------------------------------------
        A Future is a value that will become ready at some point in the future.
        It works asynchronously, it can be suspended and resumed/polled later until it is ready.
        A runtime is required to call poll() each time an .await is called, wraps the future in a Pin once and calls poll() until it is Ready.

        Inside the poll method, there is a context that contains the waker, that is used to notify the runtime when the future can be polled again.
        The future has an state = (await_step, captured variables) which are changed by the poll method.

        el future es pinned para evitar que se mueva de lugar su stack??
        al parecer la utilidad de pin es para cuando se lo hace manual, y poder alterar entre polls / antes de arrancar, la memoria del future, ya sea asignacion.


        sí podés pinneear y luego .await en otro thread, pero tiene que ser Box::pin (heap), no pin! (stack). El pin garantiza que el future no cambie de dirección en memoria; Send garantiza que podés transferir ese Pin<Box<F>> a otro thread

        let fut = async {
            let data = String::from("hola");
            let r = &data;
            async {}.await;
            println!("{r}");
        };
        let pinned = Box::pin(fut);  // heap @ 0xABC — dirección fija
        tokio::spawn(async move {
            pinned.await;  // ✅ pollea en worker thread
        });

        En general no hace falta, porque el tipo es Send. si se quiere compartir datos con el padre se puede usar Arc<T> o Mutex<T>.

    Info
    --------------------------------------------------

     el runtime es quien llama a poll; el usuario sólo usa .await. (ej Tokio)
     cuando se llama a .await, el runtime envuelve una vez el future en un Pin y luego va llamando apoll(), hasta que sea Ready.

     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
        if listo() {
            Poll::Ready(valor)
        } else {
            cx.waker().wake_by_ref(); // Notifica al runtime
            Poll::Pending
        }
    }
    se puede pasar el waker a otros futures para que se notifiquen cuando esté listo.



    Examples:
    --------------------------------------------------

        Async fn
        --------------------------------------------------
            async fn future() -> i32 {
                async {}.await;   // suspends and resumes
                async {}.await;   // suspends and resumes
                123
            }
            let result = future().await;
            assert_eq!(result, 123);

        Async block
        --------------------------------------------------
            let future = async {
                async {}.await;   // suspends and resumes
                async {}.await;   // suspends and resumes
                123
            };
            let result = future.await;
            assert_eq!(result, 123);

        Async closure
        --------------------------------------------------
            let future = async || {
                async {}.await;   // suspends and resumes
                async {}.await;   // suspends and resumes
                123
            };
            let result = future.await;
            assert_eq!(result, 123);

        Struct implementing Future
        --------------------------------------------------
            struct MyFuture {
                value: i32,
            }
            impl Future for MyFuture {
                type Output = i32;
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
                    if self.value == 1 {
                        Poll::Ready(self.value)
                    } else {
                        self.value += 1;
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
            let future = MyFuture { value: 0 };
            let result = future.await;
            assert_eq!(result, 1);

    Trait:
    --------------------------------------------------
        enum Poll<T> {  // Result of the poll method: Ready or Pending
            Ready(T),
            Pending,
        }

        trait Future {
            type Output;
            fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output>;
        }

        - &mut self:
            - exclusive access to the future, no other executor can poll it at the same time
            - mutable access, can change/update the state of the future (step and captured variables)
        - pin:
            - prevents the future from being moved in memory without unsafe.
            - only in the poll method you will use unsafe access, but in the rest of the code
            it wont be allowed to access the fields without unsafe. Not moved by accident.
            So once pinned the future will mantain the same memory address.


        States:
        --------------------------------------------------
        - Ready
        - Pending:
            Notifies the executor that it can be polled again,
            adds it to the wakers list to be polled again
            suspends and resumes when it can be polled again.
            if the waker (_cx.waker().wake_by_ref()) is not notified, it stays suspended forever.

        Timers, IO, files, ... :
        --------------------------------------------------
            an integration with the async/await executor that
            is responsible for notifying the waker of the completion/update of the
            asynchronous operation,  and resumes the future. It can notify status update
            of the asynchronous operation  or if it completed, then the poll decides
            if it is pending or ready.

    Basic implementation:
    --------------------------------------------------

        struct MyFuture {
            // state:
            value: i32,
        }

        impl Future for MyFuture {

            type Output = i32;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
                if some_condition {
                    Poll::Ready(value)
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }

    Poll flow:
    --------------------------------------------------
        ┌─────────────────────────────────────────────────────────────┐
        │ Executor                                                    │
        │    │                                                        │
        │    ├──▶ future.poll(cx)                                    │
        │    │         │                                              │
        │    │         ├──▶ Poll::Ready(T) → Completed, return T    │
        │    │         │                                              │
        │    │         └──▶ Poll::Pending → Wait, poll again        │
        │    │                     │                                  │
        │    │◀───────────────────┘                                  │
        │    │                                                        │
        └─────────────────────────────────────────────────────────────┘
*/
#[cfg(test)]
pub mod future_trait {

    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll, Waker},
    };
    struct NoOpWaker;

    impl std::task::Wake for NoOpWaker {
        fn wake(self: std::sync::Arc<Self>) {
            // No-op waker for testing
        }
    }

    pub struct ValueFuture {
        pub value: i32,
    }
    impl Future for ValueFuture {
        type Output = i32;
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
            if self.value == 1 {
                Poll::Ready(self.value)
            } else {
                self.value += 1;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    /*
    Polling manually:
     */
    #[test]
    pub fn manual_dispatch() {
        let mut future = ValueFuture { value: 0 };
        let mut future = Pin::new(&mut future);

        let waker = Waker::from(std::sync::Arc::new(NoOpWaker));
        let mut cx = Context::from_waker(&waker);

        let result = future.as_mut().poll(&mut cx);
        assert_eq!(result, Poll::Pending);
        let result = future.as_mut().poll(&mut cx);
        assert_eq!(result, Poll::Ready(1));
    }

    /*
    Polling with tokio:
     */
    #[tokio::test]
    pub async fn tokio_dispatch() {
        let mut future = ValueFuture { value: 0 };
        let future_pinned = Pin::new(&mut future);
        let result = future_pinned.await; // tokio will poll
        assert_eq!(result, 1);
    }
}
/*
FUTURES IN STACK AND HEAP:
========================================================================
    Follow the same rules as impl or dyn traits.
    By default, futures are in stack.
    To put them in heap, use Pin<Box<MyFuture>>

    let x: &mut MyFuture = &mut MyFuture { value: 0 };    // stack
    let x: Pin<&mut MyFuture> = Pin::new(x);              // stack
    let x: Pin<Box<MyFuture>> = Box::pin(x);              // heap
*/
#[cfg(test)]
mod futures_in_stack_and_heap {

    use super::future_trait::ValueFuture;
    use std::pin::Pin;

    #[test]
    pub fn futures_in_stack_and_heap() {
        let mut future = ValueFuture { value: 0 };
        let _future_pin: Pin<&mut ValueFuture> = Pin::new(&mut future); // stack
        let _future_box_pin: Pin<Box<ValueFuture>> = Box::pin(future); // heap
    }
}

/*
PIN FUTURES:
========================================================================

    Why Pin Futures?
    --------------------------------------------------
    Structs futures and futures async/await can be self-referential, so they are !Unpin.
    The compiler adds PhantomPinned automatically when it detects this.
    when you pin the future, you protect it from being moved in memory without unsafe.
    The idea is you use unsafe access in the poll method to access the fields, but in the
    rest of the code you can't access the fields without unsafe. So the future
    will mantain the same memory address and the same fields.

    Pin Project:
    --------------------------------------------------
    An improved approach is to use pin_project macro to mark specific fields as pinned.
    So you can access the fields without unsafe in the rest of the code.
    And use unsafe only in the poll method in the !unpin fields.
*/
#[cfg(test)]
pub mod pin_futures {
    use super::future_trait::ValueFuture;
    use std::pin::Pin;

    #[tokio::test]
    pub async fn pin_futures() {
        let mut future = ValueFuture { value: 0 };
        let future_pinned = Pin::new(&mut future); // Pin<&mut ValueFuture>
        let result = future_pinned.await;
        assert_eq!(result, 1);
    }
}

/*
ASYNC BLOCK:
========================================================================

    Async are futures  (syntax sugar: translates to Future struct)
    await is a poll loop until Ready (syntax sugar: translates to poll loop)

        let future: Future<Output = i32> = async {
            // code
            123 // return value
        };
        let result = future.await;

            // Generated code:

            struct ExampleFuture {
                state: State,
            }
            impl Future for ExampleFuture {
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
                    // code
                }
            }

            // loop until Ready:
            while let Poll::Pending = future.poll(cx) {
                // code
            }
            let result = future.poll(cx);
            match result {
                Poll::Ready(v) => v,
                Poll::Pending => unreachable!(),
            }

    Async block States
    --------------------------------------------------
        Each await inside an async block is a different state of the future.

        let x = String::from("hello");
        async { // State 0: before run (captured variables)
            assert_eq!(x, String::from("hello"));    // captured variable
            let await1 = await;  // State 1
            let await2 = await;  // State 2
            let await3 = await;  // State 3
        } // State 4: completed (return value)

            // Generated code:

            enum State {
                State0,
                State1,
                State2,
                State3,
                State4,
            }

            struct ExampleFuture<'a> {
                state: State,

                // captured variables by ref/value
                x: &'a String,
            }
            impl Future for ExampleFuture<'a> {
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
                    // code
                    // advances to the next state
                }
            }

    ASYNC MOVE:
    --------------------------------------------------
        move ownership of the captured variables to the future

        let x = String::from("hello");
        async move {
            assert_eq!(x, String::from("hello"));    // captured variable
            let await1 = await;  // State 1
            let await2 = await;  // State 2
            let await3 = await;  // State 3
        } // State 4: completed (return value)

        in this case x: String is moved to the future.
*/
pub mod async_block {

    #[tokio::test]
    pub async fn async_block() {
        let x = String::from("hello");
        // let future: impl Future<Output = i32>
        let future = async move {
            assert_eq!(x, String::from("hello")); // captured variable
            async {}.await; // State 1
            async {}.await; // State 2
            async {}.await; // State 3
            123
        }; // State 4: completed (return value)
        let result = future.await;
        assert_eq!(result, 123);
    }

    #[tokio::test]
    pub async fn async_block_move() {
        let x = String::from("hello");
        async move {
            assert_eq!(x, String::from("hello")); // captured variable
            async {}.await; // State 1
        }
        .await; // State 2: completed (return value)
    }
}

/*
ASYNC fn:
========================================================================

    Async fn are async blocks without move

        async fn something(): i32 {
            async {}.await; // call to another future, suspends and resumes
            123
        }

    Equivalent to:

        fn something() -> impl Future<Output = i32> {
            async {
                async {}.await; // call to another future, suspends and resumes
                123
            }
        }
*/
#[cfg(test)]
pub mod async_fn {
    #[tokio::test]
    pub async fn async_fn() {
        async fn demo() -> i32 {
            async {}.await;
            123
        }
        let result = demo().await;
        assert_eq!(result, 123);
    }
}

/*
ASYNC CLOSURES:
========================================================================

    // Similar to closures but returning a Future:
    ------------------------------------------------------------

    // Regular function traits:
    fn call(&self) { }            // Fn
    fn call_mut(&mut self) { }    // FnMut
    fn call_once(self) { }        // FnOnce

    // Async function traits:
    fn call(&self) -> impl Future<Output = ...> { ... }        // AsyncFn
    fn call_mut(&mut self) -> impl Future<Output = ...> { ... }    // AsyncFnMut
    fn call_once(self) -> impl Future<Output = ...> { ... }         // AsyncFnOnce


    same as fn, two ways to define it:
    let x = || {
        async {}
    };

    let y = async || {
        println!("hello");
    };

*/
#[cfg(test)]
pub mod async_closures {
    /*
    AsyncFn
     */
    #[tokio::test]
    pub async fn async_fn() {
        let closure_fn = async || {
            // impl AsyncFn() -> i32
            async {}.await;
            123
        };
        let future = closure_fn(); // impl Future<Output = i32>
        let result: i32 = future.await;
        assert_eq!(result, 123);
    }

    /*
    AsyncFnMut
     */
    #[tokio::test]
    pub async fn async_fn_mut() {
        let mut data = 10;

        // ✅ AsyncFnMut - takes &mut self
        let mut async_mut_closure = async || {
            data += 1; // Mutable capture
            async {}.await;
            data
        };

        // First call: &mut self
        let future1 = async_mut_closure(); // Returns impl Future
        let result1 = future1.await;
        assert_eq!(result1, 11);

        // Second call: &mut self (data was modified)
        let future2 = async_mut_closure();
        let result2 = future2.await;
        assert_eq!(result2, 12); // data incremented again
    }

    #[tokio::test]
    pub async fn async_fn_once() {
        let data = 10;

        // ✅ AsyncFnOnce - takes self (consume)
        let async_once_closure = async move || {
            let result = data + 1; // Capture by value (move)
            async {}.await;
            result
        };

        // First call - consumes the closure
        let future = async_once_closure(); // Consumed here
        let result = future.await;
        assert_eq!(result, 11);

        // ❌ Second call does NOT work - already consumed
        // let future2 = async_once_closure();  // Error: value used after move
    }
}

/*
WHEN ASYNC IS !UNPIN:
========================================================================

    !Unpin in async/await:
    --------------------------------------------------
    * References/pointers are created pointing to LOCAL data in the async block
    and at least one await (suspension / resumption) inside the async block.
    local data can be from:
    * creation of local data
        * manual (let x = 10;)
        * captures of parameters/arguments with async move

    When a future is created from an async block, the compiler creates an internal struct,
    and if it detects that it is !Unpin, it adds the phantompinned to mark it as !Unpin.
    Then it is wrapped in a Pin to protect it from movements before calling poll
    or in the suspended state.

*/
#[cfg(test)]
pub mod not_unpin_and_async {
    use std::future::Future;

    /*
    Cases: Arguments and captures in async/await when async is !Unpin:
    --------------------------------------------------
    for captures the same analysis applies as for arguments.
    */

    /*
    Auto ref after await: !unpin
    */
    #[tokio::test]
    pub async fn auto_ref_after_await() {
        fn fn_future() -> impl Future<Output = ()> {
            async {
                let data = 10;
                let ref_data = &data; // self-referential reference inside the future
                async {}.await; // suspends and resumes
                assert_eq!(*ref_data, 10); // Potential UB, because ref_data is no longer 
                // valid if the future moved
            }
        }
        fn_future().await;
    }

    /*
    Auto ref before await: !unpin
     */
    #[tokio::test]
    pub async fn auto_ref_before_await() {
        fn fn_future() -> impl Future<Output = ()> {
            async {
                let data = 10;
                let _ref_data = &data; // self-referential reference inside the future
                async {}.await; // just the fact of suspending/resuming the compiler 
                // marks the UB inconsistency error
            }
        }
        fn_future().await;
    }

    /*
    Captures in async/await with move, and auto-ref:
     */
    #[tokio::test]
    // params or captures: async move same as defining it inside the async
    pub async fn capture_move_with_auto_ref() {
        fn fn_future(data: i32) -> impl Future<Output = ()> {
            async move {
                // data lives here now
                let _ref_data = &data; // self-referential reference inside the future
                async {}.await; // just the fact of suspending/resuming the compiler 
                // marks the UB inconsistency error
            }
        }
        fn_future(10).await;
    }

    /*
    async fn are async blocks without move
     */
    #[tokio::test]
    pub async fn async_fn_no_move() { /* */
    }
}
