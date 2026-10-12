import init, { Universo } from  "./life/pkg/life.js";

const TAMANO_CELDA = 10; // tamaño de celda en pixeles
const INTERVALO = 100; // 100 ms de espera
const ESPERA_RESIZE = 200; // 200 ms de espera entre cambio de tamaño de ventana

const COLOR_FONDO = "#1a1d23"; // color de fondo = color muerte
const COLOR_VIVA = "#3b4455"; // color de las celular vivas

const canvas = document.querySelector("#fondo"); // elemento canvas en DOM
// El contexto 2d es el objeto con las herramientas para dibujar sobre el
const ctx = canvas.getContext("2d");

let memoria;
let universo;

function crearUniverso() {
    if (universo) {
        universo.free();
    }

    // establecemos cuantas columnas tendrá el juego con funcion techo de la division del ancho de la ventana en px / tamaño de celda
    const columnas = Math.ceil(window.innerWidth / TAMANO_CELDA);
    // establecemos cuantas filas tendrá el juego con funcion techo de la division del alto de la ventana en px / tamaño de celda
    const filas = Math.ceil(window.innerHeight / TAMANO_CELDA);
    // >>> es el operador de desplazamiento logico a la derecha
    // primero convierte el valor a un entero de 32 bits
    // como desplaza 0 posiciones el valor numerico no cambia pero trunca cualquier parte decimal
    // siempre da un no negativo
    const semilla = Date.now() >>> 0;

    // creamos el universo en su estado inicial
    universo = Universo.nuevo(columnas, filas, semilla);

    canvas.width = universo.ancho() * TAMANO_CELDA; // ancho del canvas
    canvas.height = universo.alto() * TAMANO_CELDA; // alto del canvas

    dibujar();

}

function dibujar() {
    const columnas = universo.ancho();
    const filas = universo.alto();
    // sintaxis de Uint8Array
    // new Uint8Array(buffer, puntero, largo)
    const celdas = new Uint8Array(
        memoria.buffer, // crea el bloque del buffer
        universo.puntero_celdas(), // direccion del array lineal de wasm donde vive el array lineal de celdas del universo
        columnas * filas // tamaño del array lineal de bytes
    );

    ctx.fillStyle = COLOR_FONDO; // fillStyle elige el color
    // fillRect pinta un rectangulo relleno medido en pixeles desde la esquina superior izquierda
    // fillRect(x, y, ancho, alto) con x,y coordenadas de origen
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    ctx.fillStyle = COLOR_VIVA;
    for (let fila = 0; fila < filas; fila++) {
        for (let col = 0; col < columnas; col++) {
            if (celdas[fila * columnas + col] === 1) {
                ctx.fillRect(
                    col * TAMANO_CELDA,
                    fila * TAMANO_CELDA,
                    TAMANO_CELDA,
                    TAMANO_CELDA
                );
            }
        }
    }

}

let ultimoAvance = 0;
let temporizador;

function bucle(tiempo) {
    if (tiempo - ultimoAvance >= INTERVALO) {
        universo.avanzar();
        dibujar();
        ultimoAvance = tiempo;
    }
    requestAnimationFrame(bucle);
}

// funcion que el navegador llama cuando se mueve el mouse
function alMoverPuntero(e) {
    const col = Math.floor(e.clientX / TAMANO_CELDA);
    const fila = Math.floor(e.clientY / TAMANO_CELDA);
    universo.despertar(fila, col);
}

// funcion de antirebote al cambiar el tamaño de la ventana
function alCambiarTamano() {
    clearTimeout(temporizador);
    temporizador = setTimeout(crearUniverso, ESPERA_RESIZE);
}

const wasm = await init(); // descarga y prepara el .wasm y devuelve un objeto con lo que Wasm exporta, entre otras cosas, su memory
memoria = wasm.memory; // guardamos la memoria de wasm para poder  llevar a cabo todo el proceso de dibujar
crearUniverso(); // se crea el primer universo

// a la escucha del movimiento del ratón
window.addEventListener("pointermove", alMoverPuntero);
// a la escucha del cambio de tamaño de ventana
window.addEventListener("resize", alCambiarTamano);
requestAnimationFrame(bucle);