import type React from "react"
import { ThemeToggle } from "./ThemeToggle"

interface LayoutProps {
  children: React.ReactNode
}

const Layout: React.FC<LayoutProps> = ({ children }) => {
  return (
    <div className="min-h-screen bg-background">
      <header className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container flex h-14 items-center justify-between">
          <div className="flex items-center space-x-2">
            <h1 className="text-lg font-semibold">Your App</h1>
          </div>
          <ThemeToggle />
        </div>
      </header>
      <main className="container mx-auto py-6">{children}</main>
    </div>
  )
}

export default Layout
