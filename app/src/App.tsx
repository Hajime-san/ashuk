import './App.css'
import { useInternalProcess, useMutationInternalProcess } from './hooks/useInternalProcess'

function App() {
  const { response, error, mutate } = useMutationInternalProcess<string, { message: string }>('command_with_message')

  return (
    <div className="App">
      <h1>Vite + React + Tauri</h1>
      <div className="card">
        <button onClick={() => mutate({ message: 'world ' + Math.random().toFixed(2) })}>
          button
        </button>
        <p>message: {response}</p>
        <p>error: {error?.message}</p>
      </div>
    </div>
  )
}

export default App
