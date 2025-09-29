import { useEffect, useState } from "react"
import "./App.css"
import { listen } from "@tauri-apps/api/event"

interface Info {
  id: string
  alias: string
  port: number
}

interface Peer {
  info: Info
  ip: string
}

function App() {
  const [peers, setPeers] = useState<Peer[]>([])

  useEffect(() => {
    const unlisten = listen<Peer>("peer-joined", (e) => {
      console.log(e.payload)
      setPeers((prev) => [...prev, e.payload])
    })

    return () => {
      unlisten.then((u) => u())
    }
  }, [])

  return (
    <main className="container">
      <ul>
        {peers.map((p) => (
          <li>{p.info.id}</li>
        ))}
      </ul>
    </main>
  )
}

export default App
