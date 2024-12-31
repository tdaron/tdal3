import { createSignal, For } from 'solid-js'
import './App.css'
import { Core,assemble_file } from '../../pkg/tdal3.js'
function App() {
  const [core, setCore] = createSignal(new Core(), {equals: () => false});
  
  const registers = () => core().registers_view();
  const pc = () => core().pc();
  return (
    <>
      <div>
        <p>Current program: </p>
        <textarea id="code" style="width: 500px; height: 200px;" value={[".ORIG x200\nADD R2, R7, #7\nADD R2, R2, #3"]}/>
        <br/>
        <button on:click={() => {
          //@ts-ignore
          let content = document.getElementById("code")?.value.split("\n");
          console.log(content);
          let assembled = assemble_file(content);
          console.log(assembled);
          let newCore = new Core();
          newCore.load_obj(assembled);
          setCore(newCore);
        }}>Assemble !</button>
        <h1>Pc: 0x{pc().toString(16)} </h1>
        <h1>Registers: </h1>
        <For each={Array.from(registers())}>{(val, _) => (
          <span> {val} </span>
        )}
        </For>
        <button on:click={() => {
          core().step()
          setCore(core);
        }} >Step</button>
        <button on:click={() => {
          //@ts-ignore
          window.i = setInterval(() => {
          core().step()
          setCore(core);
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
