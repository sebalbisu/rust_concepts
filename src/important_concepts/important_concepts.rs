/*

OOP Problems:
--------------------------------------------
Problemas wih OOP:
* Datos inflados: metadatos por todos lados.
• Pointer Chasing: telaraña de punteros, peligra seguridad memoria, peor en multihilo.
• Datos en heap: no usa cache.
• Invariante rota: Cualquiera que herede de tu clase puede romper tus reglas internas (invariantes) por descuido o desconocimiento. mediante la modificacion del estado de variables heredadas a estados invalidos.
* El Problema del Diamante (con Herencia Múltiple): cuando hay conflicto de nombre de metodo no sabe a cual usar.
• Clase Base Frágil: cambiar un padre puede perjudicar inesperadamente a algun hijo.
• El problema del Gorila y la Banana: Querías reutilizar una función simple (la banana), pero como está atada a una clase, terminás obligado a importar toda la jerarquía (el gorila y la selva completa).
• Aliasing + Mutation Efectos Secundarios Ocultos: Como en POO los objetos se pasan por referencia por defecto, múltiples partes del código pueden modificar el mismo objeto sin que las demás se enteren, rompiendo el programa.


Strict Typing & Sized:
--------------------------------------------
- Strict Typing: Todos los tipos son conocidos en tiempo de compilacion.
- Sized: Tamaño fijo en memoria y conocido en compilacion.
Todos las variables tienen un tamaño fijo en memoria. Si un tipo no es Sized, se maneja con smart pointers (punteros 8bits que apuntan al heap).

Rust es Strict Typing y Sized.
- Las referencias son Sized porque ocupan 8bits en el stack.
- Los generics <T> se monomorfizan a un tipo concreto en compilacion, son Sized.
- Los tipos no Sized se manejan con smart pointers. Entonces la variables es Sized porque ocupa 8bits en el stack. Ej: String, Vec, HashMap, Box<T>, etc.
- Los tipos dinamicos dyn Traits no son Sized, pero si Box<dyn Traits> 8bits.


Stack y Funciones:
--------------------------------------------
las funciones, sus variables y parametros se guardan en stack, se  calcula el espacio total que usara la funcion en compilacion.
* seguridad: se limpian los datos de la pila al salir de la funcion.
* rapides: esos datos se cachean y se leen en una sola instruccion de cpu
* no require mantenimiento de memoria, ni re-organizacion, busqueda, ni interaccion con otro proceso/cpu
* Sized: requiere usar datos de tamaño fijo en compilacion.
* los variables se guardan en assembler con un offset fijo en el binario segun la longuitud del tipo de dato
heap:
* lo mantiene el cpu/ otro proceso, requiere reorganizacion, para datos con longuitud variable.
* Se usa con Box, Vec, String, Rc, Arc...

todos los metodos, variables o respuestas que se usen dentro d e funciones deben ser sized, si no lo son deben ser encapsuladas en un Box<T> o referencia para usar solo el pointer.


Referencias vs Punteros:
--------------------------------------------

Referencias: (&T, &mut T)
* seguridad de memoria: garantia direccion de memoria apunta a un valor valido.
* no puede ser null
* lifetime dependiente del valor apuntado.
* Prevención de Condiciones de Carrera:&T multiples lectores, &mut T solo un escritor XOR multiple lectores.
* reducir permisos: ej & baja el permiso a solo lectura, &mut saca el permiso de ownership.

Punteros: (*const T, *mut T)
* pueden ser invalidos: apuntar a una direccion de memoria invalida
* nullables
* desferenciar un null da un Segmentation Fault (crash)
* no usa lifetime.
* *const T solo lectura, *mut T multiples lectura y escritura en simultaneo.
* se usa con unsafe { ... }


Herencia de Permisos de Acceso (Read/Write):
--------------------------------------------
• Con Ownership: Si la raíz es mutable, todo lo de adentro y anidados es mutable.
• Con Referencias: Si el puntero es inmutable, todo lo de adentro y anidados es inmutable, si es mutable el interior y anidados es mutable.


Copy y Move: copia o mueve el valor al asignar.
--------------------------------------------
Al asignar una variable a otra, si es Copy se copia el valor, si es Move se transfiere el ownership a la nueva variable, y si es referencia no se transfiere ownership ni se copia el valor.

// Copy: value copy
let x = 1;
let y = x; // copy x value to y, x is still valid

// Move: ownership transfer
let x = String::from("Hello");
let y = x; // move x value to y, x is no longer valid

// References, preserve ownership of any type
let x = String::from("Hello");
let y = &x; // y is a reference to x, x is still valid

Auto-Ref:
--------------------------------------------
Automaticamente entrega una referencia(inm/mut) o el ownership segun se necesite.

t.method() ->  method(&self | &mut self | Self)
.....................
Adapta hacia el parametro (&self | &mut self | Self) definido en el metodo

t == &t
.....................
PartialEq::eq(&self, other: &Rhs) -> bool;


Operator * y Deref: Acceder al valor
--------------------------------------------

- *(&T) -> T
- *(&&T) -> &T
- *T si T:Deref -> Target   // permite leer / escribir dependiendo de los accesos de T.
                            // *(T::Deref(&T) -> &Target) -> *(&Target) -> Target
- *Box<T> -> T

Ej:

let x = Box::new(1);
assert_eq!(*y, 1); // leer ok
*x = 2; // error escribir

let mut y = Box::new(1);
assert_eq!(*y, 1); // leer ok
*y = 2; // escribir ok


Auto-Deref:
--------------------------------------------
Automaticamente llama a deref.
Facilita el uso de valores desde referencias o smart pointers.

    let x:&Target = &T
    .....................

    fn some_fn(&Target) => some_fn(&T)
    .....................

    data.method() -> (***data).method()
    .....................
    data.method() aplica tantos * como haga falta hasta encontrar el metodo en algun valor padre.


Box<T>: Smart Pointer hacia el heap
--------------------------------------------

Deref, DerefMut Traits:
- Lectura: (Deref)
    *box -> T (tipos Copy)
    &*box -> &T (tipos Move)
    let x: &T = &box (auto-deref)
- Escritura: (DerefMut)
    *box = T
- Acceso a metodos de T:
    box.method_t() : gracias a Auto-Deref y Auto-Ref: (&*box).method() -> T.method(&t)
- cambiar donde apunta:
  swap(&mut box1, other: &mut box2) // intercambia los punteros
  replace(&mut box1, box2) // apunta a box2 ahora.

Drop Trait:
- drop automatico

Utilidades de Box<T>:
.....................
- usar punteros en lugar de valores para evitar copias costosas.
- guardar datos en heap, tamaños grandes o longuitud variable
- reducir tamaño de structs
- pasar datos no-sized a funciones/metodos.
- estructuras recursivas del mismo tipo T (elementos unicos, no multiple acceso): ej: nodos, listas enlazadas, arboles, etc. imposible usando T<T<T>...>
- tipos dinamicos: box<dyn Traits> permite usar tipos dinamicos en tiempo de compilacion.
- prevenir Stack Overflow


UnsafeCell<T>
--------------------------------------------

Rc<T>: [Reference Counted]
--------------------------------------------
muchos "ownerships inmutables" de T

Deref Trait:
- leer T y acceder a metodos de T
- no puede escribir en T

Drop Trait:
* Drop automatico: cuando no quedan mas referencias en Rc<T>.

* Mas simple que referencias: no necesita lifetimes, ni drop del ownership.
* Seguro en memoria

version multi-threads: Arc<T> [Atomic Reference Counted]

Utilidades de Rc<T>:
.....................
- Hacer un dato inmutable. (permiso mut no le permite mutar)
- Similar a referencias, pero sin lifetimes, mas simple.
- Patrón Flyweight en videojuegos: lectura eficiente, multiples instancias consumen el mismo dato (ej texturas), ahorra memoria y performance.
- Recursividad, Nodos, Grafos y Árboles: apuntar a un mismo elemento desde multiples puntos, sin usar referencias.
- Rc<Config> lectura de config en multiples partes del codigo solo lectura.

Cell<T> y RefCell<T>: [Cell y Reference Cell]
--------------------------------------------
Mutabilidad interna con múltiples accesos mutables seguros

La variable se declara como inmutable. El control se mueve al tiempo de ejecución. Podés modificar el valor interno o reemplazarlo por completo por otro objeto del mismo tipo

Cell/ Refcell no usan Deref/DerefMut:
 - Cell no necesita referencias, porque copia el valor.
 - RefCell hace chequeos de seguridad de memoria en tiempo de ejecucion.

Lectura:
    cell.get() -> T (tipos Copy)
    refCell.borrow() -> &T

Escritura:
    cell.set(T)
    refCell.replace(T) | refCell.borrow_mut() -> &mut T (solo un borrow mutable por vez)

version multi-threads: Mutex<T>, RwLock<T>

nota: Cell no tiene penalidad de performance.

mutabilidad Exterior (mut) / mutabilidad Interior (Cell/RefCell):
.....................
• Mutabilidad Exterior (mut): El compilador rastrea la variable y te exige que la declares como modificable. Si está dentro de una referencia compartida (&), es intocable.
• Mutabilidad Interior (RefCell): La variable se declara como inmutable. El control se mueve al tiempo de ejecución. Podés modificar el valor interno o reemplazarlo por completo por otro objeto del mismo tipo, incluso si el contenedor está congelado como inmutable por el resto del programa.

Utilidades de Cell/RefCell:
.....................
- Modificar un estado interno (sized Cell o dinamico RefCell) sin acceso de mut.
- Contadores, estados simples, flags,... Cell dentro de un struct (inmutable).
- Caches, estados variables (Vec, HashMap, etc.) RefCell dentro de un struct (inmutable).
- Mocks testing
- Observer pattern: modificar estado interno
- extender traits inmutables (metodos con &self hacer que puedan mutar otros datos extras (Cell/RefCell))
- Rc<RefCell<T>>: Grafos, Árboles DOM, Patrón Observador, elementos mutables.
- Rc<Cell<T>>: Contadores compartidos, Banderas de estado compartidos



Genericos: <T>
--------------------------------------------
Todos los genericos se monomorfizan, son Sized.

Cualquier función o método que use <T> o impl Trait se monomorfiza en tiempo de compilación
El compilador genera una copia física y estática de la función para cada tipo concreto con el que se haya invocado

fn:
.....................
fn hello(t: impl T) { /* ... */ }
// uso:
hello(42);
// resultado monomorfizado: funcion especifica para i32
// fn hello_i32(t: i32) { /* ... */ }

impl:
.....................
impl<T> From<T> for MiWrapper<T> { ... }
// implementacion concreta:
impl From<i32> for MiWrapper<i32> { ... } // monomorfiza para i32
// uso:
MiWrapper::from(true) // monomorfiza para bool

Struct, Traits
.....................
idem

Traits:
--------------------------------------------

Tipos:
..............
* impl Trait<T, ...>, resuelve a tipos concretos en compilacion.
* dyn Traits: tipos dinamicos en runtime. Siempre con referencias o fat pointers (Box<dyn Traits>), para que el puntero sea Sized dentro de la funcion. Para usar como dyn el Trait debe cumplir que sus metodos tengan parametros y retornos Sized y asi poder calcular el stack de la funcion en compilacion.

T vs types
..............
T: varias implementaciones de un trait por tipo
types: solo una implementacion por type

T:
    impl From<i32> for MyType { ... }
    impl From<String> for MyType { ... }

    // uso: el consumidor elige cual usar
    MyType::from(10) // monomorfiza para i32
    MyType::from(String::from("Hello")) // monomorfiza para String

Types:
    pub trait Iterator {
        type Item; // Tipo asociado
        fn next(&mut self) -> Option<Self::Item>;
    }
    impl Iterator for MyType {
        type Item = i32; // fijo a i32
        // ...
    }

beneficios de type:
• ergonomia: evita tener que especificar el T en el consumidor (impl Trait / dyn Trait)
• unicidad de tipo: solo trabaja un tipo
• permite usar dyn Traits al sacar generics



Lifetimes y generics en definiciones:
--------------------------------------------
Structs:
structs MyType<'a, T> // obligatorio
// en impl<..> marca que parametros son genericos y en MyType los especificos y genericos
impl<T> MyType<'static, T> //T  generico
impl<'a> MyType<'a, i32> // 'a generico
impl MyType<'static, i32> // ningun generico

Funciones:
fn(&ref) ~> fn<'a>(&'a ref)  // lifetimes se pueden omitir
fn<'a>(&'a ref1, &'a ref2) -> &'a (necesario)
fn<T>(x: T) // generics siempre van

Traits:
trait MyTrait { fn hello<'a>(&'a ref) } // 'a requiere vivir tanto como dura el metodo
impl  MyTrait  for  User {...}

trait MyTrait<'a> { fn hello<'a>(&'a ref) }  // 'a requiere vivir tanto como el objeto
impl<'a>  MyTrait<'a>  for  User {...}
impl<'a> MyTrait<'a> for User<'a> // comparten el mismo 'a en User y el metodo

Idem para generics
trait MyTrait { fn hello(t: impl Debug) }
trait MyTrait { fn hello<T: Debug>(t: T) } // afecta solo al metodo, la implementacion es capas de manejar al tipo
trait MyTrait<T> { fn hello(t: impl T) }  // asociado al trait

Implementaciones:
--------------------------------------------
de genericas, a implementacion de traits, a tipos concretos:

struct Contenedor<T> {
    valor: T,
}

// 1. Implementación base para CUALQUIER T
impl<T> Contenedor<T> {
    fn nuevo(valor: T) -> Self {
        Self { valor }
    }
}

// 2. Implementación específica SOLO para String
impl Contenedor<String> {
    fn gritar(&self) {
        println!("¡{}!", self.valor.to_uppercase());
    }
}

// 3. Granularidad 1: SOLO si T implementa Clone
impl<T: Clone> Contenedor<T> {
    fn duplicar(&self) -> Self {
        Self { valor: self.valor.clone() }
    }
}

// 4. Granularidad 2: SOLO si T implementa Display
impl<T: std::fmt::Display> Contenedor<T> {
    fn mostrar(&self) {
        println!("{}", self.valor);
    }
}

Closures:
--------------------------------------------
Closures son funciones anonimas que pueden capturar variables del entorno.
Al igual que funciones internamente son structs anonimos que implementan algun trait: Fn, FnMut, FnOnce.

Traits:
Fn: (&) captura variables referencia inmutable
FnMut: (&mut) captura variables por referencia mutable.
FnOnce: (move) captura variables por valor.

Herencia:
FnOnce : FnMut : Fn : fn

fn: cualquier funcion que no sea closure.

for<'a>
.....................
una closure que es parametro de una funcion, donde la closure tiene un parametro que es una referencia:

- si la referencia es externa a la funcion
fn<'a>(cb: F) where F: Fn(&'a ref)

- si la referencia nace dentro de la funcion
fn(cb: F) where F: for<'a> Fn(&'a ref)


Future
--------------------------------------------
Permite ejecutar codigo concurrente o asincrono en un solo hilo, alternando el control de ejecucion entre diferentes tareas.

Future es un trait, el implementador llama a poll() hasta que termina y obtiene el type Output
Un struct tiene que implementar el trait con un solo tipo de dato Output

Implementaciones:
.....................
- future.await: espera a que el future termine y obtiene el type Output
- join!(fut1, fut2, ...): espera a que todos los futures terminen y obtiene el type Output de cada uno
- select!(fut1, fut2, ...): espera a que alguno de los futures termine primero y obtiene el type Output del futuro que termino
- block_on(fut): ejecuta el future y bloquea el hilo hasta que termine (ideal para main() o tests)

Utilidad Futures
.....................

Lifetimes en futures:
.....................
si dentro del future se usan referencias externas (async fn(&'a ref)) entonces el future puede vivir hasta que viva 'a. Entonces fn (&'a ref) -> impl Future... + 'a. En versiones actuales rust elije si agregar + 'static o + 'a.. por ergonomia.

Ergonomia Futures:
.....................
Pin<Box<Future<Output = T> +'a >> es poco legible

pub type FutureRef<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub type FutureStatic<T> = Pin<Box<dyn Future<Output = T> + 'static>>
pub type FutureSend<T> = Pin<Box<dyn Future<Output = T> + 'static + Send>>
futures::future::BoxFuture<'a, T>

dyn Future<Output = T>
.....................
permite usarce con distintos (async fn -> T) siempre y cuando su salidas sean del mismo tipo T

Pin: Pin<Box<Future<Output = T>>>
.....................
(Pin = seguridad obligatoria por si tiene autoreferencias, ej referencias a variables usadas internas)
con Box es posible mover el future a otro lugar de memoria, no hay garantias de que este fijo.
con impl Future no es necesario Pin, porque el future queda fijo en el stack

Basicamente Pin anula del DerefMut, entonces no puede usarce el &mut para mutar el box.

Async: (Future sugar syntax)
--------------------------------------------

Async block:
.....................
async { } -> T
internamente es un struct anonimo que implementa el Future trait:
captura variables: puede capturar variables del entorno, pero no es un closure, es un valor.

Async Function/Method:
.....................
async fn (params...) -> T

fn (params...) -> impl Future<Output = T>

Async Closure:
.....................
async |params...| { ... } -> T

|params... | { ... } -> Future<Output = T>
igual a async fn (..) -> T pero al ser closure puede capturar variables del entorno.
implementan trais AsyncFn, AsyncFnMut, AsyncFnOnce.

la diferencia entre Fn y AsyncFn es porque se ejecutan en contextos diferentes una con una llamada y la otra con un implementador con .await

Async Traits Methods:
.....................
trait { async fn show() -> T }
trait { fn show() -> impl Future<Ouput = T> }


Send y Sync
--------------------------------------------
Seguridad entre hilos.

Concepto:
Send: pasar ownership (entre hilos) => un escritor
Sync: pasar referencias/accesos inmutables (entre hilos) => muchos lectores

Arc, Mutex, RwLock
--------------------------------------------
versiones atomicas de Rc, RefCell y Cell.


Utilidades Multihilo
.....................

Multihilo vs un hilo /  paralelismo vs concurrencia.
.....................

>> Concurrencia (un nucleo):
Multitarea de turnos, context switching: El OS alterna un time-slice de ejecucion para cada hilo en el mismo nucleo. Siempre ejecuta una linea de codigo a la vez.

>> Paralelismo (varios nucleos):
Realiza tareas en paralelo usando mas nucleos, terminando el trabajo mas rapido.
El OS scheduler coloca los nuevos hilos en los nucleos disponibles.
Puede migrar hilos a otros nucleos para optimizar la performance, en ese caso la memoria ram no se mueve, es trasparente para el codigo.
Paralelismo: Ejecuta multiples lineas de codigo a la vez, puede dar lugar a problemas de race conditions, no ocurre en un hilo. Por eso se usan los traits Send y Sync para garantizar la seguridad.

>> App Multi-nucleo:
- multihilo
- paralelismo (+ concurrencia en cada hilo)
- 100% hardware usado
- mas memoria cache L1 usada, mas memoria por nucleo.
- tareas livianas y bloqueantes, dividir tarea bloqueante en paralelo mejora la performance.
- Send / Sync para datos que se pasan a hilos distintos.

>> App Un nucleo:
- un hilo o multihilo
- concurrencia
- tarea bloqueante frena el resto del hilo.

>> Multiples App-un-nucleo en varios nucleos:
- independencia.
- resilencia si cae una app, la otra sigue corriendo.
- mas memoria para reservada para cada programa
- 100% hardware usado
- tarea bloqueante frena el resto del hilo, no puede dividir tareas pesadas en paralelo.

Tecnicas Multi-hilo
.....................
- Thread-per-task / Dedicated Threads:
hilos dedicados a tareas especificas definidas, puede ser de larga duracion o corta duracion.

- Thread Pool & Work-Stealing:
a un conjunto de hilos se le asignan tareas mediante un scheduler en orden de llegada. Ideal trabajo con miles de tareas, despacho por orden de llegada.

Task:
.....................
Unidad de trabajo que se ejecuta en un hilo con implementadores que manejan un pool de hilos para despacho de tareas. Se utilizan con un future porque son concurrentes.

Utilizar multihilo
.....................
>> std::thread::spawn(callback)
crea un hilo nuevo y ejecuta la callback. (handle.join()) bloquea el hilo principal hasta que la callback termine.

>> std::thread::scope({...})
Permite abrir hilos que van a vivir menos que el scope, el scope los espera a que terminen en caso de no esperarlos manualmente con .join():
- permite usar referencias locales en hilos sin usar Arc.
- obtiene un Result<T, E> con .join(), no hay que usar channels/mutex para obtener la respuesta.
- simplicidad codigo
- panics: si un sub-hilo falla, el hilo del scope falla.

>> tokio::spawn(task)
ejecuta una tarea asíncrona dentro del runtime de pool de hilos tokio. (handle.await) espera a que la tarea termine de forma asíncrona.

>> data.par_iter_mut().for_each(|x| { ... })
Rayon: Paralelismo de Datos mediante Iteradores en su pool de hilos, super simple para filtros, maps, etc.

>> Channels:
transmicion de datos entre hilos.
* mpsc: (N -> 1)
* broadcast: (M -> N) todos los consumidores reciben los mensajes que hay en el buffer.
* crossbeam: (M -> N) el consumidor que recibe primero el mensaje se lo queda.
* mpmc: (M -> N) idem crossbeam con async/await. (flume lib)
* watch: (1 -> N), todos los consumidores solo tienen el ultimo mensaje, sin buffer.
* oneshot: (1 -> 1) 1 message y muere.
* scsp: (1 -> 1) version baja latencia para un hilo de mpsc (N -> 1).

>> Sincronizacion:
acceso/bloqueo de recursos compartidos.
* mutex: (1 escritor)
* rwlock: (N lectores XOR 1 escritor)
* semaphore: limita el numero de N accesos simultaneos.
* barrier: espera a que N hilos lleguen a un punto y luego siguen.
* condvar: espera a que una condicion se cumpla, luego se desbloquea.

>> Actores: Actix
@TODO


WebAssembly (WASM)
--------------------------------------------
un .wasm es un modulo wasm, que adentro tiene funciones (puede ser concurrentes), y se crean instancias cuando llega un pedido de ese modulo, cuando termina la peticion se muere la instancia, entre instancias no comparten memoria, ni datos. La creacion de instancias es instantanea, no hay tiempo de espera, y casino consume recursos comparado a un docker que tiene un os.
Es un binario que se ejecuta con un implementador warm.

- compila a nativo
- funciona en client / server
- seguridad sandbox: limites de acceso a recursos.
- portable: a cualquier sistema operativo / browser
- muy poco peso
- tiempo arranque inmediato
- actualmente un solo hilo (+ concurrencia), a futuro multihilo.
- creacion inmediata
- se destruye la instancia cuando termina la peticion.

Edge Computing
.....................
Al tener una huella minima de recursos y maxima portabilidad, puede correr en multiples nodos cercanos al usuario, sin necesidad de un servidor centralizado. Casi se comporta como un archivo estatico. Ideal Tareas independientes que no dependen de una base de datos central pesada son ideales para el Edge con Wasm.

- Criptografía rápida: Generar o verificar firmas digitales y tokens (como JWT) aprovechando el rendimiento casi nativo de Wasm cerca del usuario.
- Transformación de datos en tránsito: Sanitizar, comprimir o modificar payloads de solicitudes (JSON, XML o fragmentos de archivos) antes de que crucen toda la red.
- Enrutamiento y personalización: Aplicar lógica de negocio ligera, como control de acceso geográfico o pruebas A/B, directamente en los nodos de la red global.

Utilidades WASM
.....................

- edge computing
- extensiones, plugins, customizaciones de terceros: corren en entornos seguros, independientes entre si y rapidos, con poca memoria, poco peso.
- entornos seguros
- browser apps

Modules
--------------------------------------------

Similar a como una cocina ordena sus elementos con cajoneras, estantes y subdivisiones.
Son lugares para organizar y agrupar muchos items relacionados(funciones, structs, traits, etc.).
Dentro de un modulo se puede definir sub-modulos.

Declarar sub-modulos:
.....................
- inlined en un archivo de un modulo
- archivo hijo de una carpeta (la carpeta es el modulo porque tiene un mod.rs)
- un archivo en otra ubicacion (se incluye con #[path = "ruta/al/archivo"] mod nombre_modulo;)

Visibilidad de la api:
.....................
permite controlar la visibilidad de los elementos del modulo hacia afuera del modulo.
Por defecto son privados.
Los elementos privados solo son visibles dentro del modulo y sus sub-modulos. La visibilidad se otorga de adentro hacia afuera.

- pub(super): visible dentro del modulo padre
- pub(in <path_ancestro>): visible dentro del modulo ancestro especificado
- pub(crate): visible dentro del proyecto
- pub: visible afuera del proyecto (ej: librerias son accedidos por terceros en otros proyectos)


Result<T, E> y Option<T>
--------------------------------------------
- Result<T, E>: Es un enum con dos variantes Ok(T), Err(E).
- Option<T>: Es un enum con dos variantes Some(T), None.


Error
--------------------------------------------

Manejo de errores:
.....................
- Result<T, E>: Obliga a manejar el error.
- ?: Si es error, retorna ese error.
- panic!: Detiene el programa al instante. Errores irrecuperables.
- unwrap: Retorna el valor o panic.


Definir errores:
.....................
- nativo: Enum implementa traits Display, Debug y Error.
- thiserror: Enum con msg de errores en cada variante. (ideal librerias, declarativo)
- anyhow: anyhow:Result<T:Error>.context(msg) encapsula cualquier error y le agrega un msg. (ideal apps donde maneja multiples errores y mas importante el msg). Tambien puede buscar errores en el stack.


Iterators
--------------------------------------------
Modelo Rust (Loop Fusion):
Ítem 1 ──> [ Método 1 ──> Método 2 ──> Método M ] ──> Resultado 1
Ítem 2 ──> [ Método 1 ──> Método 2 ──> Método M ] ──> Resultado 2

Modelo por separado:
[Ítem 1..N] --(Método 1)--> [Lista Temporal 1] --(Método 2)--> [Lista Temporal 2] ...

Ventajas Loop Fusion:
.....................
- ahorro recursos: no genera listas temporales innecesarias.
- uso cache: cada item transformado se puede trabjar en cache por ser un solo dato, en cambio una lista temporal mas dificil de cachear es mas costosa.
- ahorra costo de generar varios bucles con contadores, solo tiene un bucle.
- short-circuiting methods (find, take, all) permiten terminar el bucle antes de tiempo, ahorrandose correr los bucles enteros.
- son perezosos, no se evalúan hasta que se necesitan, ahorrando recursos.

Trait:
.....................
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

Convenciones de nombres:
.....................
Cada metodo devuelve un tipo de Iterator distinto.
- iter(&self) -> IteratorA
- iter_mut(&mut self) -> IteratorB
- into_iter(self) -> IteratorC


Testing
--------------------------------------------
>> Test unitarios:
testea una unidad de codigo por separado. Un Structs, una funcion, etc, se mockean internamente dependencias internas. Se escribe el test en gral en el mismo archivo del codigo a testear.
se pueden testear apis privadas ademas de las publicas

>> Test de integración:
testea la integración de varias unidades de codigo. Se mockean externamente los servicios. Se escriben los test en la carpeta tests/. Solo testean apis publicas. Se puede testear un modulos, dos modulos interactuando, o todo el proyecto.

>> E2E Tests:
testea desde afuera la aplicacion compilada, paywright, cypress, ...


Estrategias de mocking:
.....................
- internal mocking: impl Trait o Box<dyn Trait> para poder testear.
- external mocking: mock de la APIs/http/db/... se cambia el endpoint por uno local mockeado.

Librerias
.....................
>> Mocking:
- mockall: a partir de un trait, genera un tipo T:Trait mockeable (un struct configurable para tests)
- wiremock: servidor http local, ofrece su endpoint local y configurar endpoints mockeados.

>> Assertions:
- pretty_assertions: mejor output de assertions, colorido y legible, diffs de errores.

>> Property Testing:
- proptest: generación de datos automaticos y corre el test muchas veces.



@TODO:
Mas Smart Pointers:
Cow (Clone-on-Write), ManuallyDrop, UnsafeCell, Ref/RefMut, Pin
*/
