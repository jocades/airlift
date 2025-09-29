import { useEffect, useState } from "react"
import { listen } from "@tauri-apps/api/event"
import { Layout } from "./components/layout"
import { Offer } from "./components/offer"
import { HeartIcon, LaptopMinimalIcon, WifiIcon } from "lucide-react"
import { Badge } from "./components/ui/badge"

interface Info {
  id: string
  alias: string
  port: number
}

interface Peer {
  info: Info
  ip: string
}

type Event =
  | {
      kind: "join"
      peer: Peer
    }
  | {
      kind: "leave"
      id: string
    }

export type Page = "discover" | "settings"

const peerData: Peer[] = [
  {
    info: {
      id: crypto.randomUUID(),
      alias: "Peer 1",
      port: 8000,
    },
    ip: "192.168.8.1",
  },
  {
    info: {
      id: crypto.randomUUID(),
      alias: "Peer 2",
      port: 8001,
    },
    ip: "192.168.8.2",
  },
  {
    info: {
      id: crypto.randomUUID(),
      alias: "Peer 3",
      port: 8002,
    },
    ip: "192.168.8.3",
  },
]

export default function App() {
  const [page, setPage] = useState<Page>("discover")
  const [peers, setPeers] = useState<Peer[]>([])
  const [open, setOpen] = useState(false)

  useEffect(() => {
    const unlisten = listen<Event>("events", ({ payload }) => {
      console.log({ payload })

      switch (payload.kind) {
        case "join":
          setPeers((prev) => [...prev, payload.peer])
          break
        case "leave":
          setPeers((prev) => prev.filter((p) => p.info.id !== payload.id))
          break
        default:
          break
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
    <div className="flex flex-col items-center justify-center">
      <div className="flex items-center gap-x-6 mb-8">
        <WifiIcon className="scale-150" />
        <h1 className="text-4xl font-bold">Discover</h1>
      </div>
      <ul className="flex flex-col items-center w-full gap-y-4">
        {peers.map((p) => (
          <li
            key={p.info.id}
            className="flex w-[80%] max-w-4xl bg-card rounded-xl border shadow-md p-6 items-center justify-between cursor-pointer hover:bg-accent transition-colors"
          >
            <div className="flex gap-x-8 items-center">
              <LaptopMinimalIcon className="scale-150" />
              <div className="flex flex-col gap-y-1">
                <h2 className="font-semibold">{p.info.alias}</h2>
                <Badge variant="secondary" className="font-mono">
                  #{p.info.id.substring(0, 8)}
                </Badge>
              </div>
            </div>
            <HeartIcon />
          </li>
          /* <Card key={p.info.id} className="w-[80%]">
            <CardHeader>
              <CardTitle>
                <ComputerIcon />
                {p.info.alias}
              </CardTitle>
            </CardHeader>
            <CardContent></CardContent>
          </Card> */
        ))}
      </ul>
    </div>
  )
}

function Settings() {
  return <div>Settings page</div>
}
