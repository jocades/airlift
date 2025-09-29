import { useEffect, useState } from "react"
import { listen } from "@tauri-apps/api/event"
import { Layout } from "./components/layout"
import { Offer } from "./components/offer"

interface Info {
  id: string
  alias: string
  port: number
}

interface Peer {
  info: Info
  ip: string
}

interface Join {
  kind: "join"
  peer: Peer
}

interface Leave {
  kind: "leave"
  id: string
}

type Message = Join | Leave

export type Page = "discover" | "settings"

export default function App() {
  const [page, setPage] = useState<Page>("discover")
  const [peers, setPeers] = useState<Peer[]>([])
  const [open, setOpen] = useState(true)

  useEffect(() => {
    const unlisten = listen<Message>("discover", ({ payload }) => {
      console.log({ payload })

      if (payload.kind === "join") {
        setPeers((prev) => [...prev, payload.peer])
      } else if (payload.kind === "leave") {
        setPeers((prev) => prev.filter((p) => p.info.id !== payload.id))
      }
    })

    return () => {
      unlisten.then((u) => u())
    }
  }, [])

  return (
    <Layout setPage={setPage}>
      {page === "discover" && <Discover peers={peers} />}
      {page === "settings" && <Settings />}
      <Offer open={open} setOpen={setOpen} />
    </Layout>
  )
}

function Discover({ peers }: { peers: Peer[] }) {
  return (
    <div>
      <h1>Discover page</h1>
      <ul>
        {peers.map((p) => (
          <li key={p.info.id}>{p.info.id}</li>
        ))}
      </ul>
    </div>
  )
}

function Settings() {
  return <div>Settings page</div>
}
