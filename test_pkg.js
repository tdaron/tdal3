import * as wasm from "./pkg"

await wasm.default(); // Initialize WASM
const core = new wasm.Core();
let registers = core.registers_view()
let memory = core.memory_view()
let code = new Uint16Array(2);
code[0] = 0x0200;
code[1] = 0b0001_010_111_1_00111; // R2 = R7 + 7
core.load_obj(code);
core.step()
core.step()
console.log(registers)
console.log(memory)
