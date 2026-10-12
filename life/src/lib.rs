use wasm_bindgen::prelude::*; // prelude es la convención en rust del nombre del modulo con las funciones más utilizadas

const CADA_N_GENERACIONES: u32 = 20;
const NUM_CHISPAS: u32 = 3;

/*
En general el flujo de rust + web assembly será:
lib.rs es creado al iniciar el proyecto, al compilar: 
-> life_bg.wasm es creado y aqui vive el algoritmo escrito en rust pero en formato para wasm, luego se crea 
-> life.js que es el pegamento que carga el .wasm yy expone sus funciones en codigo JS, luego 
-> index.html carga el script de logica, script.js para mostrar en
-> navegador
 */
/// Este struct es el universo en donde sucede todo el juego de la vida, con atributos ancho, alto, celdas, estado_azar y generacion
#[wasm_bindgen] // macro de atributo que marca la funcion para ser exportada y leida por wasm como numeros
pub struct Universo {
    ancho: u32, // columnas
    alto:  u32, // filas
    celdas: Vec<u8>,
    estado_azar: u32,
    generacion: u32
}

// Esta función no se marca con #[wasm_bindgen] porque solo es para funcionamiento interno de rust, JS no las necesita y por ende no las podrá ver
impl Universo {
    // funcion para generar numeros pseudoaleatorios
    fn azar(&mut self) -> u32 {
        let mut x = self.estado_azar;
        // se le hace un XORSHIFT al valor de estado_azar para aleatorizarlo
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.estado_azar = x;
        x // retornamos el numero
    }

    // funcion para encontrar el indice en el vector de una linea, debe retornar usize porque es lo unico que acepta rust para tamaños e indices de listas
    fn indice(&self, fila: u32, col: u32) -> usize {
        (fila * self.ancho + col) as usize
    }

    // Aqui se calcula el total de vecinos que tiene una fila
    // Se usa el truco de establecer el cambio del vecino anterior como alto - 1 y ancho - 1 para no desbordar u32 y la operacion modulo para dar geometria de toroide (no hay bordes "reales" en la cuadricula, todo es continuo)
    fn vecinos_vivos(&self, fila: u32, col: u32) -> u8 {
        let mut suma = 0;
        for delta_f in [self.alto - 1, 0, 1] {
            for delta_c in [self.ancho - 1, 0, 1] {
                if delta_f == 0 && delta_c == 0 {
                    continue; // se salta la iteracion porque estaría contandose a si mismo para el total de casillas vivas
                }
                let f = (fila + delta_f) % self.alto;
                let c = (col + delta_c) % self.ancho;
                suma += self.celdas[self.indice(f,c)];
            }
        }
        suma
    }

}

// macro de atributo que marca la funcion para ser exportada y leida por wasm como numeros, mientras que life.js hace el proceso contrario y toma los numeros de wasm y los transforma a string
// Esta sí la exportamos a wasm para que sea leída por JS
#[wasm_bindgen]
impl Universo {
    pub fn nuevo(ancho: u32, alto: u32, semilla: u32) -> Universo {
        // Se crea un nuevo universo
        let mut universo = Universo {
        // se le asignan sus atributos
        // los siguientes dos atributos pueden abreviarse así porque la variable se llama igual que el campo
        ancho, // equivalente a ancho: ancho
        alto, // equivalente a alto: alto
        celdas: vec![0; (ancho * alto) as usize], // se crea el vector que contiene a toda la cuadricula pero en version de una sola linea
        estado_azar: if semilla == 0 { 4242424242 } else { semilla }, // establecemos la semilla, con un fallback a un numero entero por si la semilla = 0
        generacion: 0 // contador de cuantas generaciones lleva el universo
        };
        
        for i in 0..universo.celdas.len() {
            // llama a la funcion de azar y le aplica modulo, si cae entre 0 y 24 entonces vive (25% probabilidad)
            if universo.azar() %100 < 25 {
                universo.celdas[i] = 1; // si tuvo exito entonces la celda vive
            }
        }

        universo

    }

    // funcion avanzar calcula el nuevo estado del universo
    pub fn avanzar(&mut self) {
        let mut siguiente = self.celdas.clone(); // creamos una copia profunda (independiente a la original) para escribir los nuevos valores de las celdas
        for fila in 0..self.alto {
            for col in 0..self.ancho {
                let i = self.indice(fila, col); // celda actual en el vector linea
                let viva = self.celdas[i] == 1; // estado de la celda actual, viva: true muerta: false
                let n = self.vecinos_vivos(fila, col); // total de vecinos vivos

                // aqui se hace un match acorde a las reglas del juego de la vida
                siguiente[i] = match (viva, n) {
                    (true, 2) | (true, 3) | (false, 3) => 1, // las unicas condiciones que dan vida
                    _ => 0 // patron comodin, el resto de condiciones son de muerte
                };
            }
        }

        self.celdas = siguiente; // actualizamos las celdas del struct universo
        self.generacion += 1; // actualizamos las generaciones de nuestro struct universo

        // cuando el universo llegue a 20 generaciones
        if self.generacion % CADA_N_GENERACIONES == 0 {
            for _ in 0..NUM_CHISPAS {
                // se eligen filas y columnas al azar
                let fila = self.azar() % self.alto;
                let col = self.azar() % self.ancho;
                // se llama a despertar para intentar dar vida
                self.despertar(fila, col);
            }
        }

    }

    // funcion para reanimar el tablero que será llamada cada 20 generaciones del universo
    pub fn despertar (&mut self, fila: u32, col: u32) {
        // trabaja en un area de 5x5, tomando entonces la vecindad de una celda a dos celdas de distancia en horizontal y vertical
        for delta_f in [self.alto - 2, self.alto - 1, 0, 1, 2] {
            for delta_c in [self.ancho - 2, self.ancho - 1, 0, 1, 2] {
                // calculo de fila y columna para identificar la celda, se usa modulo para el efecto de toroide
                let f = (fila + delta_f) % self.alto;
                let c = (col + delta_c) % self.ancho;
                // la celda solo volverá a tener vida si azar() % 100 da un valor entre 0 y 49, si la celda ya estaba viva esto no le afectará
                if self.azar() % 100 < 50 {
                    let i = self.indice(f, c);
                    self.celdas[i] = 1;
                }
            }
        }
    }

    // getters, como solo es lectura se usa referencia inmutable &self
    
    // getter de ancho
    pub fn ancho(&self) -> u32 {
        self.ancho
    }

    // getter de alto
    pub fn alto(&self) -> u32 {
        self.alto
    }

    //getter que devuelve la direccion de memoria donde empieza el primer byte de la lista
    // *const u8 es un tipo que es un puntero de solo lectura y no se puede escribir a través de el porque es undefined behavior
    pub fn puntero_celdas(&self) -> *const u8 {
        self.celdas.as_ptr() // expone la direccion de memoria donde empieza el primer byte de la lista
    }


}