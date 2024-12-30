import { createSignal, For } from 'solid-js'
import './App.css'
import { Core } from '../../pkg/tdal3.js'
function App() {
  const core = new Core();

  const [registers, setRegisters] = createSignal(core.registers_view(), { equals: () => false });
  const [pc, setPc] = createSignal(core.pc())

  const code = new Uint16Array(4);
  code[0] = 0x200;
  code[1] = 0b0001_010_111_1_00111; // R2 = R7 + 7
  code[2] = 0b0001_010_010_1_00011; // R2 = R2 + 3
  code[3] = 0b0000_111_111111110;
  core.load_obj(code);
  return (
    <>
      <div>
        <p>Current program: </p>
        <p>R2 = R7 + 7 </p>
        <p>LOOP R2 = R2 + 3 </p>
        <p>JUMP TO LOOP</p>
        <h1>Pc: 0x{pc().toString(16)} </h1>
        <h1>Registers: </h1>
        <For each={Array.from(registers())}>{(val, _) => (
          <span> {val} </span>
        )}
        </For>
        <button on:click={() => {
          core.step()
          setPc(core.pc())
          setRegisters(registers())
        }} >Step</button>
        <button on:click={() => {
          //@ts-ignore
          window.i = setInterval(() => {
          core.step()
          setPc(core.pc())
          setRegisters(registers())
            
          }, 5)
        }} >Loop</button>
        <button on:click={() => {
          //@ts-ignore
          clearInterval(window.i)            
        }} >stop</button>
      </div>
    </>
  )
}

export default App
