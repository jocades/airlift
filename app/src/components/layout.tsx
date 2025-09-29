import { SettingsIcon, WifiIcon } from "lucide-react"
import { Button } from "./ui/button"
import { Page } from "@/app"

export function Layout(props: { children: React.ReactNode; setPage: (page: Page) => void }) {
  return (
    <div className="h-screen flex flex-col">
      <div className="grid grid-cols-[auto_1fr] flex-grow">
        <Sidebar setPage={props.setPage} />
        <main className="overflow-x-hidden p-4">{props.children}</main>
      </div>
    </div>
  )
}

const navItems: { title: string; page: Page; icon: any }[] = [
  {
    title: "Discover",
    page: "discover",
    icon: WifiIcon,
  },
  {
    title: "Settings",
    page: "settings",
    icon: SettingsIcon,
  },
]

function Sidebar({ setPage }: { setPage: (page: Page) => void }) {
  return (
    <aside className="sticky top-0 flex flex-col gap-4 overflow-y-auto py-4 px-2 border-r h-full">
      {navItems.map((item) => (
        <Button key={item.page} size="icon" onClick={() => setPage(item.page)}>
          <item.icon />
        </Button>
      ))}
    </aside>
  )
}
