/* @refresh reload */
import { render } from 'solid-js/web'
import './index.css'
import App from './App.tsx'
import init from "../../pkg/tdal3.js"
const root = document.getElementById('root')
init().then(() => {
  render(() => <App />, root!)
})
